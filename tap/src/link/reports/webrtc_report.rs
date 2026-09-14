use std::collections::HashMap;
use serde::Serialize;
use chrono::{DateTime, Utc};
use crate::protocols::parsers::l4_key::L4Key;
use crate::protocols::parsers::rtp_tagger::{RtpDirection, RtpStream};
use crate::wired::ethernet::types::webrtc_conversation::WebRtcConversation;

#[derive(Serialize)]
pub struct WebRTCReport {
    pub conversations: Vec<WebRTCConversationReport>,
}

#[derive(Serialize)]
pub struct WebRTCConversationReport {
    pub negotiation_key: String,
    pub source_address: String,
    pub source_mac: Option<String>,
    pub source_port: u16,
    pub destination_address: String,
    pub destination_port: u16,
    pub has_rtp: bool,
    pub has_dtls: bool,
    pub has_audio: bool,
    pub has_video: bool,
    pub stream_count: u16,
    pub rtp_streams: Vec<RtpStreamReport>,
    pub dtls_app_data_records: u64,
    pub first_seen: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub bytes_rx: u64,
    pub bytes_tx: u64,
}

#[derive(Serialize)]
pub struct RtpStreamReport {
    pub ssrc: u32,
    pub direction: String,
    pub payload_types: Vec<u8>,
    pub packet_count: u64,
    pub byte_count: u64,
    pub media_kind: String,
    pub estimated_lost: u32,
    pub sequence_min: u16,
    pub sequence_max: u16,
    pub timestamp_span: u32,
    pub marker_count: u64,
}

pub fn generate(conversations: &HashMap<L4Key, WebRtcConversation>) -> WebRTCReport {
    let mut out: Vec<WebRTCConversationReport> = Vec::new();

    for c in conversations.values() {
        out.push(WebRTCConversationReport {
            negotiation_key: c.negotiation_key.clone(),
            source_address: c.source_address.to_string(),
            source_mac: c.source_mac.clone(),
            source_port: c.source_port,
            destination_address: c.destination_address.to_string(),
            destination_port: c.destination_port,
            has_rtp: c.has_rtp,
            has_dtls: c.has_dtls,
            has_audio: c.has_audio,
            has_video: c.has_video,
            stream_count: c.stream_count,
            rtp_streams: c.rtp_streams.iter().map(stream_to_report).collect(),
            dtls_app_data_records: c.dtls_app_data_records as u64,
            first_seen: c.first_seen,
            last_activity: c.last_activity,
            bytes_rx: c.bytes_rx,
            bytes_tx: c.bytes_tx,
        });
    }

    WebRTCReport { conversations: out }
}

fn stream_to_report(s: &RtpStream) -> RtpStreamReport {
    let direction = match s.direction {
        RtpDirection::ClientToServer => "CLIENT_TO_SERVER",
        RtpDirection::ServerToClient => "SERVER_TO_CLIENT",
    }.to_string();

    RtpStreamReport {
        ssrc: s.ssrc,
        direction,
        payload_types: s.payload_types.clone(),
        packet_count: s.packet_count as u64,
        byte_count: s.byte_count,
        media_kind: s.media_kind.to_string(),
        estimated_lost: s.estimated_lost,
        sequence_min: s.sequence_min,
        sequence_max: s.sequence_max,
        timestamp_span: s.timestamp_span,
        marker_count: s.marker_count as u64,
    }
}