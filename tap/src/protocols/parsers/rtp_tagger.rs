const STUN_MAGIC_COOKIE: [u8; 4] = [0x21, 0x12, 0xA4, 0x42];
const RTP_STREAM_MIN_PACKETS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtpDirection {
    ClientToServer,
    ServerToClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtpMediaKind {
    Audio,
    Video,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpStream {
    pub ssrc: u32,
    pub direction: RtpDirection,
    pub payload_types: Vec<u8>,
    pub packet_count: usize,
    pub byte_count: u64,
    // Suspected type of media in the stream
    pub media_kind: RtpMediaKind,
    // Best effort estimate of lost packets from sequence number gaps.
    pub estimated_lost: u32,
    pub sequence_min: u16,
    pub sequence_max: u16,
    pub timestamp_span: u32,
    // Packets with marker bit set (video frame boundaries)
    pub marker_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct RtpPacket {
    direction: RtpDirection,
    payload_type: u8,
    sequence: u16,
    timestamp: u32,
    ssrc: u32,
    marker: bool,
    payload_len: usize,
}

pub fn tag(c2s: &[Vec<u8>], s2c: &[Vec<u8>]) -> Option<Vec<RtpStream>> {
    let mut packets: Vec<RtpPacket> = Vec::new();
    for d in c2s {
        if let Some(p) = parse_rtp_packet(d, RtpDirection::ClientToServer) {
            packets.push(p);
        }
    }
    for d in s2c {
        if let Some(p) = parse_rtp_packet(d, RtpDirection::ServerToClient) {
            packets.push(p);
        }
    }

    if packets.len() < RTP_STREAM_MIN_PACKETS {
        return None;
    }

    // Group by SSRC. Eeach SSRC is one media stream.
    let mut ssrcs: Vec<u32> = Vec::new();
    for p in &packets {
        if !ssrcs.contains(&p.ssrc) {
            ssrcs.push(p.ssrc);
        }
    }

    let mut streams: Vec<RtpStream> = Vec::new();
    for ssrc in ssrcs {
        let mut group: Vec<RtpPacket> =
            packets.iter().filter(|p| p.ssrc == ssrc).copied().collect();

        if group.len() < RTP_STREAM_MIN_PACKETS || !sequences_advance(&group) {
            continue;
        }

        group.sort_by_key(|p| p.sequence);

        let packet_count = group.len();
        let byte_count: u64 = group.iter().map(|p| p.payload_len as u64).sum();

        let mut payload_types: Vec<u8> = Vec::new();
        let mut marker_count = 0usize;
        for p in &group {
            if !payload_types.contains(&p.payload_type) {
                payload_types.push(p.payload_type);
            }
            if p.marker {
                marker_count += 1;
            }
        }

        let sequence_min = group.first().map(|p| p.sequence).unwrap_or(0);
        let sequence_max = group.last().map(|p| p.sequence).unwrap_or(0);

        // Loss estimate: expected count over the sequence span compared to what we observed.
        let span = sequence_max.wrapping_sub(sequence_min) as u32;
        let estimated_lost = span.saturating_add(1).saturating_sub(packet_count as u32);

        let ts_min = group.iter().map(|p| p.timestamp).min().unwrap_or(0);
        let ts_max = group.iter().map(|p| p.timestamp).max().unwrap_or(0);
        let timestamp_span = ts_max.wrapping_sub(ts_min);

        let mean_payload = if packet_count > 0 {
            byte_count / packet_count as u64
        } else {
            0
        };
        let media_kind = classify_media(mean_payload, marker_count, packet_count);

        streams.push(RtpStream {
            ssrc,
            direction: group[0].direction,
            payload_types,
            packet_count,
            byte_count,
            media_kind,
            estimated_lost,
            sequence_min,
            sequence_max,
            timestamp_span,
            marker_count,
        });
    }

    if streams.is_empty() {
        None
    } else {
        Some(streams)
    }
}


pub fn dominant_direction(streams: &[RtpStream]) -> Option<RtpDirection> {
    streams.iter().max_by_key(|s| s.byte_count).map(|s| s.direction)
}

fn classify_media(mean_payload: u64, marker_count: usize, packet_count: usize) -> RtpMediaKind {
    let marker_ratio = if packet_count > 0 {
        marker_count as f32 / packet_count as f32
    } else {
        0.0
    };

    if marker_ratio >= 0.05 {
        RtpMediaKind::Video
    } else if marker_count == 0 {
        // No frame markers at all. That's audio.
        RtpMediaKind::Audio
    } else if mean_payload >= 600 {
        // A few markers and large packets. Likely video.
        RtpMediaKind::Video
    } else {
        RtpMediaKind::Audio
    }
}

fn parse_rtp_packet(buf: &[u8], direction: RtpDirection) -> Option<RtpPacket> {
    if buf.len() < 12 {
        return None;
    }

    if buf.len() >= 8 && buf[4..8] == STUN_MAGIC_COOKIE {
        return None;
    }

    let first = buf[0];
    if (20..=63).contains(&first) {
        return None;
    }

    if (first & 0xC0) >> 6 != 2 {
        return None;
    }

    let marker = (buf[1] & 0x80) != 0;
    let payload_type = buf[1] & 0x7F;

    if (72..=76).contains(&payload_type) {
        return None;
    }

    let sequence = u16::from_be_bytes([buf[2], buf[3]]);
    let timestamp = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let ssrc = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);

    Some(RtpPacket {
        direction,
        payload_type,
        sequence,
        timestamp,
        ssrc,
        marker,
        payload_len: buf.len().saturating_sub(12),
    })
}

fn sequences_advance(group: &[RtpPacket]) -> bool {
    if group.len() < 2 {
        return false;
    }
    let mut advancing = 0usize;
    for w in group.windows(2) {
        let delta = w[1].sequence.wrapping_sub(w[0].sequence);
        if (1..=100).contains(&delta) {
            advancing += 1;
        }
    }
    advancing * 2 >= group.len() - 1
}