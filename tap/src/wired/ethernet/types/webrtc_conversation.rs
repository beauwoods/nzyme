use std::mem;
use std::net::IpAddr;
use chrono::{DateTime, Utc};
use crate::protocols::parsers::l4_key::L4Key;
use crate::protocols::parsers::rtp_tagger::{RtpMediaKind, RtpStream};
use crate::state::tables::udp_table::UdpConversation;

#[derive(Debug, Clone)]
pub struct WebRtcConversation {
    pub session_key: L4Key,
    pub negotiation_key: String,
    pub source_address: IpAddr,
    pub source_mac: Option<String>,
    pub source_port: u16,
    pub destination_address: IpAddr,
    pub desteination_mac: Option<String>,
    pub destination_port: u16,
    pub has_rtp: bool,
    pub has_dtls: bool,
    pub has_audio: bool,
    pub has_video: bool,
    pub stream_count: u16,
    pub rtp_streams: Vec<RtpStream>,
    pub dtls_app_data_records: usize,
    pub first_seen: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub bytes_rx: u64,
    pub bytes_tx: u64,
}

impl WebRtcConversation {

    pub fn build(conversation: &UdpConversation,
                 negotiation_key: String,
                 rtp_streams: Vec<RtpStream>,
                 dtls_app_data_records: usize) -> WebRtcConversation {

        let has_rtp = !rtp_streams.is_empty();
        let has_dtls = dtls_app_data_records > 0;
        let has_audio = rtp_streams.iter().any(|s| s.media_kind == RtpMediaKind::Audio);
        let has_video = rtp_streams.iter().any(|s| s.media_kind == RtpMediaKind::Video);
        let stream_count = rtp_streams.len() as u16;

        let session_key = L4Key::new(
            conversation.source_address,
            conversation.source_port,
            conversation.destination_address,
            conversation.destination_port,
        );

        WebRtcConversation {
            session_key,
            negotiation_key,
            source_address: conversation.source_address,
            source_mac: conversation.source_mac.clone(),
            source_port: conversation.source_port,
            destination_address: conversation.destination_address,
            desteination_mac: conversation.destination_mac.clone(),
            destination_port: conversation.destination_port,
            has_rtp,
            has_dtls,
            has_audio,
            has_video,
            stream_count,
            rtp_streams,
            dtls_app_data_records,
            first_seen: conversation.start_time,
            last_activity: conversation.most_recent_segment_time,
            bytes_rx: conversation.bytes_count_rx,
            bytes_tx: conversation.bytes_count_tx,
        }
    }

    pub fn estimate_struct_size(&self) -> u32 {
        let mut size = mem::size_of::<Self>() as u32;

        size += self.negotiation_key.len() as u32;

        if let Some(mac) = &self.source_mac {
            size += mac.len() as u32;
        }

        for stream in &self.rtp_streams {
            size += mem::size_of::<RtpStream>() as u32;
            size += (stream.payload_types.len() * mem::size_of::<u8>()) as u32;
        }

        size
    }
}