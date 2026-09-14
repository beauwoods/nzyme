use std::sync::{Arc, Mutex};
use log::error;
use crate::state::tables::webrtc_table::WebRtcTable;
use crate::wired::ethernet::types::webrtc_conversation::WebRtcConversation;

pub struct WebRtcProcessor {
    table: Arc<Mutex<WebRtcTable>>
}

impl WebRtcProcessor {

    pub fn new(table: Arc<Mutex<WebRtcTable>>) -> Self {
        Self { table }
    }

    pub fn process(&mut self, session: Arc<WebRtcConversation>) {
        match self.table.lock() {
            Ok(table) => table.register_conversation(session),
            Err(e) => error!("Could not acquire WebRTC table mutex: {}", e)
        }
    }

}