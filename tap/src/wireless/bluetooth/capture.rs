use std::collections::HashMap;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use anyhow::{bail, Context, Error};
use chrono::Utc;
use dbus::arg;
use dbus::arg::{RefArg, Variant};
use dbus::blocking::{Connection};
use dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
use log::{debug, error, info, warn};
use crate::wireless::bluetooth::bluetooth_device_advertisement::BluetoothDeviceAdvertisement;
use crate::configuration::BluetoothInterface;
use crate::messagebus::bus::Bus;
use crate::messagebus::channel_names::BluetoothChannelName;
use crate::metrics::Metrics;
use crate::to_pipeline;

pub struct Capture {
    pub metrics: Arc<Mutex<Metrics>>,
    pub bus: Arc<Bus>,
    pub configuration: BluetoothInterface,
    // The stable config key ("bt-<address>"), used as the metrics registry
    // key -- NOT the same as the resolved hci device name, which can (and
    // does) change across the life of this capture thread. Metrics are
    // registered once under this name in main.rs; using the transient hci
    // name here instead would silently break every metrics update after the
    // first hci-index change.
    pub interface_name: String
}

const DBUS_INTERFACE: &str = "org.bluez.Adapter1";

// Cap on backoff between consecutive failed cycles (adapter not found, or a
// D-Bus/HCI call failed). Growing this rather than retrying at a flat 1s
// avoids hammering an adapter that is mid-reset or genuinely gone with fresh
// D-Bus/HCI traffic every second, which real-world testing on cheap USB
// Bluetooth dongles found made a marginal adapter's recovery worse, not
// better.
const MAX_BACKOFF_SECONDS: u64 = 60;

impl Capture {

    /// `bd_address` is the adapter's own Bluetooth device address (stable
    /// hardware identity, extracted from the "bt-<address>" interface name
    /// by bluetooth_tools::extract_bd_address_from_interface_name). This is
    /// re-resolved to a current hci device name at the top of every loop
    /// iteration via `resolve_hci_by_address`, rather than trusting a single
    /// hci name captured once at thread start: BlueZ/the kernel do not
    /// guarantee a USB Bluetooth adapter keeps the same hciN across a reset,
    /// and a fixed name silently starts referring to a different (or no)
    /// physical adapter once that happens. This mirrors how Sona WiFi
    /// interfaces are already resolved by their own stable serial number on
    /// every reconnect (see wireless::dot11::sona::capture::Capture::run).
    pub fn run(&mut self, interface_name: &str, bd_address: &str) {
        info!("Starting Bluetooth capture for adapter [{}] (interface [{}]).", bd_address, interface_name);

        let mut consecutive_failures: u32 = 0;
        let mut last_known_hci: Option<String> = None;

        loop {
            let device_name = match Self::resolve_hci_by_address(bd_address) {
                Ok(name) => {
                    if last_known_hci.as_deref() != Some(name.as_str()) {
                        info!("Bluetooth adapter [{}] is currently [{}].", bd_address, name);
                        last_known_hci = Some(name.clone());
                    }
                    name
                },
                Err(e) => {
                    error!("Could not find Bluetooth adapter with address [{}]: {}", bd_address, e);
                    consecutive_failures += 1;
                    sleep(Self::backoff(consecutive_failures));
                    continue;
                }
            };

            let result = catch_unwind(|| {
                if self.configuration.bt_classic_enabled {
                    match self.discover_devices(&device_name, "bredr") {
                        Ok(devices) => self.discovered_devices_to_pipeline(devices),
                        Err(e) => {
                            error!("Could not discover Bluetooth Classic devices on [{}]: {}", device_name, e);
                        }
                    }
                }

                if self.configuration.bt_le_enabled {
                    match self.discover_devices(&device_name, "le") {
                        Ok(devices) => self.discovered_devices_to_pipeline(devices),
                        Err(e) => {
                            error!("Could not discover Bluetooth LE devices on [{}]: {}", device_name, e);
                        }
                    }
                }
            });

            if let Err(e) = result {
                consecutive_failures += 1;
                match e.downcast_ref::<&str>() { Some(s) => {
                    error!("Could not discover Bluetooth devices: {}", s);
                } _ => { match e.downcast_ref::<String>() { Some(s) => {
                    error!("Could not discover Bluetooth devices: {}", s);
                } _ => {
                    error!("Could not discover Bluetooth devices. Panicked with an unknown type.");
                }}}}
            } else {
                consecutive_failures = 0;
            }

            /*
             * The discovery methods sleep during discovery, but we add another sleep to make sure
             * we don't empty spin in case of errors early in the discovery methods. Backs off on
             * repeated consecutive failures instead of a flat 1s -- see MAX_BACKOFF_SECONDS.
             */
            sleep(Self::backoff(consecutive_failures))
        }
    }

    fn backoff(consecutive_failures: u32) -> Duration {
        if consecutive_failures == 0 {
            Duration::from_secs(1)
        } else {
            // Shift amount capped at 6 (1 << 6 = 64s) so this can never overflow.
            let secs = 1u64 << consecutive_failures.min(6);
            Duration::from_secs(secs.min(MAX_BACKOFF_SECONDS))
        }
    }

    /// Looks up the current hci device name (e.g. "hci2") for the adapter
    /// with the given BD address, by asking BlueZ's own ObjectManager for
    /// every object it currently knows about and matching on each adapter's
    /// `Address` property. Does not cache anything -- always reflects
    /// BlueZ's live state at the moment of the call.
    fn resolve_hci_by_address(bd_address: &str) -> Result<String, Error> {
        let conn = Connection::new_system().context("Could not establish connection to D-Bus")?;
        let obj_manager = conn.with_proxy("org.bluez", "/", Duration::from_secs(5));

        #[allow(clippy::complexity)]
        let (objects, ): (HashMap<dbus::Path<'static>, HashMap<String, HashMap<String, Variant<Box<dyn RefArg>>>>>, ) =
            obj_manager.method_call("org.freedesktop.DBus.ObjectManager", "GetManagedObjects", ())
                .context("Could not fetch managed objects from D-Bus")?;

        for (path, interfaces) in objects {
            let Some(props) = interfaces.get(DBUS_INTERFACE) else { continue };
            let Some(address) = props.get("Address").and_then(|v| v.as_str()) else { continue };

            if address.eq_ignore_ascii_case(bd_address) {
                if let Some(hci_name) = path.to_string().strip_prefix("/org/bluez/").map(String::from) {
                    return Ok(hci_name);
                }
            }
        }

        bail!("No adapter with this address is currently known to BlueZ")
    }

    pub fn discover_devices(&self, device_name: &str, transport: &str)
        -> Result<HashMap<String, BluetoothDeviceAdvertisement>, Error> {

        /*
         * Using a Map to avoid unlikely case of multiple reports per discovery cycle from
         * same device.
         */
        let mut discovered: HashMap<String, BluetoothDeviceAdvertisement> = HashMap::new();

        // Connect do D-Bus.
        let conn = Connection::new_system().context("Could not establish connection to D-Bus")?;

        // Obtain bluez reference.
        let adapter = conn.with_proxy(
            "org.bluez",
            format!("/org/bluez/{}", device_name),
            Duration::from_secs(self.configuration.dbus_method_call_timeout_seconds as u64)
        );

        // Set the adapter to not discoverable and not pairable.
        adapter.set(DBUS_INTERFACE, "Discoverable", false)
            .context("Could not set Discoverable=false on device")?;
        adapter.set(DBUS_INTERFACE, "Pairable", false)
            .context("Could not set Pairable=false on device")?;

        // Set the discovery filter to set transport to selected method.
        let mut filter = arg::PropMap::new();
        filter.insert("Transport".to_string(), Variant(Box::new(transport.to_string())));
        adapter.method_call::<(), _, _, _>(DBUS_INTERFACE, "SetDiscoveryFilter", (filter,))
            .context("Could not set discovery filter")?;

        // Start bluetooth discovery.
        adapter.method_call::<(), _, _, _>(DBUS_INTERFACE, "StartDiscovery", ())
            .context("Could not start discovery")?;

        // Sleep to allow discovery.
        sleep(Duration::from_secs(self.configuration.discovery_period_seconds as u64));

        // Access the object manager to list all devices
        let obj_manager_path = "/";
        let obj_manager = conn.with_proxy(
            "org.bluez",
            obj_manager_path,
            Duration::from_secs(self.configuration.dbus_method_call_timeout_seconds as u64)
        );
        #[allow(clippy::complexity)]
        let (devices, ): (HashMap<dbus::Path<'static>, HashMap<String, HashMap<String, Variant<Box<dyn RefArg>>>>>, ) =
            obj_manager.method_call("org.freedesktop.DBus.ObjectManager", "GetManagedObjects", ())
                .context("Could not fetch devices from object manager")?;

        // Iterate over all discovered devices.
        for (path, interfaces) in devices {
            if path.to_string().starts_with(&format!("/org/bluez/{}/dev_", device_name)) {
                // Only discovered bluetooth devices.
                if let Some(props) = interfaces.get("org.bluez.Device1") {
                    /*
                     * Is this device connected to the Bluetooth adapter we are listening on?
                     * Notify in log because this will negatively impact discovery. This is most
                     * like a bluetooth device connected to the machine this tap runs on. The user
                     * should either disconnect (and forget) the device or use another bluetooth
                     * adapter that has no devices connected.
                     */
                    if let Some(true) = Self::parse_optional_bool_prop(props, "Connected") {
                        warn!("Bluetooth adapter [{}] has a connected device. This will \
                        negatively impact discovery. Disconnected and un-pair all bluetooth \
                        devices from this machine. Device: {:?}", device_name, props)
                    }

                    // Mandatory fields.
                    let mac = Self::parse_mandatory_string_prop(props, "Address");
                    let alias = Self::parse_mandatory_string_prop(props, "Alias");

                    // Optional fields.
                    let rssi = Self::parse_optional_i16_prop(props, "RSSI");
                    let tx_power = Self::parse_optional_i16_prop(props, "TxPower");
                    let name = Self::parse_optional_string_prop(props, "Name");
                    let class = Self::parse_optional_u32_prop(props, "Class");
                    let appearance = Self::parse_optional_u32_prop(props, "Appearance");
                    let modalias = Self::parse_optional_string_prop(props, "Modalias");
                    let uuids = Self::parse_optional_string_vector(props, "UUIDs");
                    let service_data = Self::parse_optional_string_vector(props, "ServiceData");

                    // Manufacturer data incl. company ID.
                    let (company_id, manufacturer_data) = if let Some(v) = props.get("ManufacturerData") {
                        Self::parse_manufacturer_data(v)
                    } else {
                        (None, None)
                    };
                    
                    discovered.insert(mac.clone(), BluetoothDeviceAdvertisement {
                        mac,
                        name,
                        rssi,
                        company_id,
                        alias,
                        class,
                        appearance,
                        modalias,
                        tx_power,
                        manufacturer_data,
                        uuids,
                        service_data,
                        device: device_name.to_string(),
                        transport: transport.to_string(),
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        // Stop discovery
        adapter.method_call::<(), _, _, _>(DBUS_INTERFACE, "StopDiscovery", ())
            .context("Could not stop discovery")?;

        Ok(discovered)
    }

    fn discovered_devices_to_pipeline(&self,
                                      devices: HashMap<String, BluetoothDeviceAdvertisement>) {
        for device in devices.values() {
            to_pipeline!(
                BluetoothChannelName::BluetoothDevicesPipeline,
                self.bus.bluetooth_device_pipeline.sender,
                Arc::new(device.clone()),
                1024
            );

            match self.metrics.lock() {
                Ok(mut metrics) => {
                    metrics.increment_processed_bytes_total(device.estimate_struct_size());
                    metrics.update_capture(&self.interface_name, true, 0, 0, true);
                },
                Err(e) => error!("Could not acquire metrics mutex: {}", e)
            }
        }
    }

    fn parse_mandatory_string_prop(props: &HashMap<String, Variant<Box<dyn RefArg>>>, name: &str)
        -> String {

        Self::parse_optional_string_prop(props, name).unwrap_or_else(|| {
            warn!("Invalid Bluetooth advertisement, not containing [{}]: {:?}", name, props);
            "Invalid".to_string()
        })
    }

    fn parse_optional_string_prop(props: &HashMap<String, Variant<Box<dyn RefArg>>>, name: &str)
                                  -> Option<String> {
        props.get(name).and_then(|v|
        v.as_str()
            .map(|val| val.to_string())
            .or_else(|| {
                warn!("Invalid Bluetooth advertisement, [{}] not a string: {:?}", name, props);
                None
            })
        )
    }

    fn parse_optional_i16_prop(props: &HashMap<String, Variant<Box<dyn RefArg>>>, name: &str)
        -> Option<i16> {
        props.get(name).and_then(|v| {
            match v.as_i64() {
                Some(val) if val <= i16::MAX as i64 => Some(val as i16),
                Some(_) => {
                    warn!("Invalid Bluetooth advertisement, [{}] value out of range for i64: {:?}", name, props);
                    None
                },
                None => {
                    warn!("Invalid Bluetooth advertisement, [{}] not u64 or not present: {:?}", name, props);
                    None
                }
            }
        })
    }

    fn parse_optional_u32_prop(props: &HashMap<String, Variant<Box<dyn RefArg>>>, name: &str)
        -> Option<u32> {
        props.get(name).and_then(|v| {
            match v.as_u64() {
                Some(val) if val <= u32::MAX as u64 => Some(val as u32),
                Some(_) => {
                    warn!("Invalid Bluetooth advertisement, [{}] value out of range for u32: {:?}", name, props);
                    None
                },
                None => {
                    warn!("Invalid Bluetooth advertisement, [{}] not u64 or not present: {:?}", name, props);
                    None
                }
            }
        })
    }

    fn parse_optional_bool_prop(props: &HashMap<String, Variant<Box<dyn RefArg>>>, name: &str)
        -> Option<bool> {

        props.get(name).and_then(|v| {
            v.0.as_any().downcast_ref::<bool>().copied().or_else(|| {
                warn!("Invalid Bluetooth advertisement, [{}] not bool: {:?}", name, props);
                None
            })
        })
    }

    fn parse_optional_string_vector(props: &HashMap<String, Variant<Box<dyn RefArg>>>, name: &str)
        -> Option<Vec<String>> {

        if let Some(v) = props.get(name) {
            match v.0.as_iter() {
                Some(iter) => {
                    let mut data = Vec::new();
                    for val in iter {
                        match val.as_str() {
                            Some(str) => {
                                data.push(str.to_string())
                            },
                            None => {
                                // Ignore non-string elements.
                                debug!("Invalid Bluetooth advertisement, [{}] includes \
                                element that is not a string: {:?}", name, props);
                            }
                        }
                    }

                    if data.is_empty() {
                        None
                    } else {
                        Some(data)
                    }
                },
                None => None
            }
        } else {
            None
        }
    }

    fn parse_manufacturer_data(manufacturer_data_var: &Variant<Box<dyn RefArg>>)
        -> (Option<u16>, Option<Vec<u8>>) {

        let company_id = match manufacturer_data_var.0.as_iter() { Some(mut iter) => {
            match iter.nth(0) {
                Some(key) => key.as_u64().map(|val| val as u16),
                None => None
            }
        } _ => {
            None
        }};

        let data = manufacturer_data_var.0.as_iter().and_then(|mut iter| {
            iter.nth(1)?.as_iter().and_then(|mut iter| {
                iter.nth(0)?.as_iter().map(|iter| {
                    iter.filter_map(|val| val.as_u64().map(|v| v as u8)).collect::<Vec<u8>>()
                })
            })
        });

        (company_id, data)
    }

}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::log_monitor::LogMonitor;
    use crate::metrics::{CaptureType, Metrics};

    // Regression test for the metrics registry key mismatch fixed alongside
    // the BD-address-resolution work: a capture's metrics must be updated
    // under the *stable* config key ("bt-<address>", `Capture::interface_name`
    // in this module), never under the transient hci device name that BlueZ
    // resolves it to -- that name can (and does) change over the capture
    // thread's lifetime, and Metrics::update_capture() silently no-ops (just
    // an error!() log line, no panic) when given a key it doesn't recognize.
    // That silence is exactly what let this bug run in production for a full
    // day before being noticed -- so this test asserts on the *data*
    // (received count), not just "did it not crash".
    //
    // This can't invoke Capture::discovered_devices_to_pipeline() directly
    // without a live D-Bus connection and a fully-constructed Bus/Configuration
    // (not available in a unit test), so it tests the same underlying
    // mechanism discovered_devices_to_pipeline relies on: Metrics keys
    // captures by whatever string it's given, and only self.interface_name
    // is guaranteed stable -- exactly like bluetooth_tools' own tests
    // exercise the address-parsing helper in isolation rather than a live
    // capture loop.
    fn new_metrics() -> Metrics {
        Metrics::new(Arc::new(LogMonitor::default()))
    }

    #[test]
    fn update_under_the_stable_interface_name_is_recorded() {
        let mut metrics = new_metrics();
        let interface_name = "bt-08:BE:AC:4D:34:F2";
        metrics.register_new_capture(interface_name, CaptureType::Bluetooth);

        metrics.update_capture(interface_name, true, 0, 0, true);

        assert_eq!(metrics.test_capture_received(interface_name), Some(1));
    }

    #[test]
    fn update_under_a_transient_hci_name_silently_does_not_update_anything() {
        // Reproduces the exact bug: the capture was registered under the
        // stable key, but an update arrives keyed by whatever hci name BlueZ
        // happened to resolve the adapter to at that moment.
        let mut metrics = new_metrics();
        let interface_name = "bt-08:BE:AC:4D:34:F2";
        metrics.register_new_capture(interface_name, CaptureType::Bluetooth);

        metrics.update_capture("hci2", true, 0, 0, true);

        // Before the fix, this was the silent failure mode: no panic, no
        // test failure from a crash -- just a metric that never moves.
        assert_eq!(metrics.test_capture_received(interface_name), Some(0));
        assert_eq!(metrics.test_capture_received("hci2"), None);
    }

    #[test]
    fn stable_interface_name_keeps_working_across_hci_name_drift() {
        // Mirrors a real reset cycle: the adapter is registered once under
        // its stable address-derived key, then the OS-assigned hci name
        // drifts across several resolutions over the capture thread's
        // lifetime (hci2 -> hci1 -> hci3, as seen in production hci-index
        // reassignment). Every one of those transient names must fail to
        // match; only the stable key may ever succeed.
        let mut metrics = new_metrics();
        let interface_name = "bt-08:BE:AC:4D:34:F2";
        metrics.register_new_capture(interface_name, CaptureType::Bluetooth);

        for drifted_name in ["hci2", "hci1", "hci3"] {
            metrics.update_capture(drifted_name, true, 0, 0, true);
        }
        assert_eq!(metrics.test_capture_received(interface_name), Some(0));

        metrics.update_capture(interface_name, true, 0, 0, true);
        assert_eq!(metrics.test_capture_received(interface_name), Some(1));
    }
}