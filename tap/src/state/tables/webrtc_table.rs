use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{Duration, Utc};
use log::{error};
use crate::helpers::timer::{record_timer, Timer};
use crate::link::leaderlink::Leaderlink;
use crate::link::reports::webrtc_report;
use crate::metrics::Metrics;
use crate::protocols::parsers::l4_key::L4Key;
use crate::wired::ethernet::types::webrtc_conversation::WebRtcConversation;

const STALE_AFTER_MINUTES: i64 = 1;

pub struct WebRtcTable {
    leaderlink: Arc<Mutex<Leaderlink>>,
    metrics: Arc<Mutex<Metrics>>,
    conversations: Mutex<HashMap<L4Key, WebRtcConversation>>,
}

impl WebRtcTable {

    pub fn new(leaderlink: Arc<Mutex<Leaderlink>>, metrics: Arc<Mutex<Metrics>>) -> Self {
        Self {
            leaderlink,
            metrics,
            conversations: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_conversation(&self, incoming: Arc<WebRtcConversation>) {
        let webrtc = (*incoming).clone();
        let key = webrtc.session_key.clone();

        match self.conversations.lock() {
            Ok(mut conversations) => {
                match conversations.get_mut(&key) {
                    Some(existing) => merge_into(existing, &webrtc),
                    None => { conversations.insert(key, webrtc); }
                }
            }
            Err(e) => error!("Could not acquire WebRTC conversations table mutex: {}", e),
        }
    }

    pub fn process_report(&self) {
        let cutoff = Utc::now() - Duration::minutes(STALE_AFTER_MINUTES);
        let mut timer = Timer::new();

        let conversations = match self.conversations.lock() {
            Ok(mut c) => {
                c.retain(|_, w| w.last_activity >= cutoff);
                c.clone()
            }
            Err(e) => {
                error!("Could not acquire WebRTC conversations table mutex for report: {}", e);
                return;
            }
        };

        if conversations.is_empty() {
            return;
        }

        let report = match serde_json::to_string(&webrtc_report::generate(&conversations)) {
            Ok(json) => json,
            Err(e) => {
                error!("Could not serialize WebRTC report: {}", e);
                return;
            }
        };

        timer.stop();
        record_timer(
            timer.elapsed_microseconds(),
            "tables.webrtc.timer.report_generation",
            &self.metrics
        );

        match self.leaderlink.lock() {
            Ok(link) => {
                if let Err(e) = link.send_report("webrtc/conversations", report) {
                    error!("Could not submit WebRTC report: {}", e);
                }
            }
            Err(e) => error!("Could not acquire leader link lock for WebRTC report: {}", e),
        }
    }

    pub fn calculate_metrics(&self) {
        let size: i128 = match self.conversations.lock() {
            Ok(c) => c.len() as i128,
            Err(e) => { error!("Could not acquire WebRTC table mutex for metrics: {}", e); -1 }
        };
        match self.metrics.lock() {
            Ok(mut m) => m.set_gauge("tables.webrtc.conversations.size", size),
            Err(e) => error!("Could not acquire metrics mutex: {}", e),
        }
    }
}

fn merge_into(existing: &mut WebRtcConversation, incoming: &WebRtcConversation) {
    existing.has_rtp   |= incoming.has_rtp;
    existing.has_dtls  |= incoming.has_dtls;
    existing.has_audio |= incoming.has_audio;
    existing.has_video |= incoming.has_video;

    for incoming_stream in &incoming.rtp_streams {
        match existing.rtp_streams.iter_mut().find(|s| s.ssrc == incoming_stream.ssrc) {
            Some(existing_stream) => {
                *existing_stream = incoming_stream.clone();
            }
            None => {
                existing.rtp_streams.push(incoming_stream.clone());
            }
        }
    }

    existing.stream_count = existing.rtp_streams.len() as u16;
    existing.dtls_app_data_records = incoming.dtls_app_data_records;
    existing.source_address = incoming.source_address;
    existing.source_mac = incoming.source_mac.clone();
    existing.source_port = incoming.source_port;
    existing.destination_address = incoming.destination_address;
    existing.destination_port = incoming.destination_port;
    existing.bytes_rx = incoming.bytes_rx;
    existing.bytes_tx = incoming.bytes_tx;
    
    if incoming.first_seen < existing.first_seen {
        existing.first_seen = incoming.first_seen;
    }

    if incoming.last_activity > existing.last_activity {
        existing.last_activity = incoming.last_activity;
    }
}