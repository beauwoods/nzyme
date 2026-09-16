use anyhow::{bail, Error};

/// Bluetooth interface config keys are expected in the form `bt-<address>`,
/// where `<address>` is the adapter's own Bluetooth device address (its BD
/// address), colon-separated or not. This mirrors how WiFi Sona interfaces
/// are keyed by `sona-<serial>` (see wireless::dot11::sona::sona_tools) --
/// both exist because the kernel-assigned name for the underlying device
/// (hciN for Bluetooth, an enumeration-order-dependent serial port for Sona)
/// is not guaranteed stable across a USB reset or reboot, but the device's
/// own hardware address is.
///
/// Returns the address in canonical colon-separated uppercase form (matching
/// what BlueZ itself reports via the `Address` property on `org.bluez.AdapterN`),
/// regardless of whether it was written with or without colons in the config.
pub fn extract_bd_address_from_interface_name(interface_name: &str) -> Result<String, Error> {
    let Some(raw) = interface_name.strip_prefix("bt-") else {
        bail!("Invalid Bluetooth interface name (expected a \"bt-<address>\" prefix): [{}]", interface_name);
    };

    let hex_only: String = raw.chars().filter(|c| *c != ':').collect::<String>().to_uppercase();

    if hex_only.len() != 12 || !hex_only.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!(
            "Invalid Bluetooth interface name [{}]: expected 12 hex characters after \"bt-\" \
            (colon-separated or not), like \"bt-08:BE:AC:4D:34:F2\" or \"bt-08BEAC4D34F2\"",
            interface_name
        );
    }

    let canonical = hex_only
        .as_bytes()
        .chunks(2)
        .map(|pair| std::str::from_utf8(pair).unwrap())
        .collect::<Vec<_>>()
        .join(":");

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_colon_separated_address() {
        assert_eq!(
            extract_bd_address_from_interface_name("bt-08:BE:AC:4D:34:F2").unwrap(),
            "08:BE:AC:4D:34:F2"
        );
    }

    #[test]
    fn accepts_lowercase_and_no_colons() {
        assert_eq!(
            extract_bd_address_from_interface_name("bt-08beac4d34f2").unwrap(),
            "08:BE:AC:4D:34:F2"
        );
    }

    #[test]
    fn rejects_missing_prefix() {
        assert!(extract_bd_address_from_interface_name("hci1").is_err());
    }

    #[test]
    fn rejects_wrong_length() {
        assert!(extract_bd_address_from_interface_name("bt-08BEAC4D34").is_err());
    }

    #[test]
    fn rejects_non_hex() {
        assert!(extract_bd_address_from_interface_name("bt-ZZBEAC4D34F2").is_err());
    }
}
