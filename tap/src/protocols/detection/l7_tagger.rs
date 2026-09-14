use std::collections::{HashMap, HashSet};
use std::panic;
use std::sync::{Arc, Mutex, MutexGuard};
use log::{debug, error, info};
use strum_macros::Display;
use crate::protocols::detection::l7_tagger::L7Tag::{HTTP, Unencrypted, SOCKS, SSH, RTSP, STUN, TURN, RTP, DTLS};
use crate::state::tables::tcp_table::TcpSession;
use crate::protocols::parsers::l4_key::L4Key;
use crate::helpers::timer::{record_timer, Timer};
use crate::messagebus::bus::Bus;
use crate::messagebus::channel_names::WiredChannelName;
use crate::metrics::Metrics;
use crate::protocols::detection::taggers::network_protocols::{http_tagger, rtsp_tagger, socks_tagger, ssh_tagger};
use crate::protocols::detection::taggers::tagger_command::TaggerCommand;
use crate::protocols::parsers::{dtls_tagger, rtp_tagger, stun_tagger};
use crate::protocols::parsers::rtp_tagger::RtpStream;
use crate::state::tables::udp_table::UdpConversation;
use crate::to_pipeline;
use crate::wired::ethernet::types::webrtc_conversation::WebRtcConversation;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Display, PartialEq, Clone, Hash, Eq)]
pub enum L7Tag {
    Unencrypted,
    HTTP,
    SMTP,
    IMAP,
    POP3,
    FTP,
    Telnet,
    TLS,
    DNS,
    SSH,
    RTSP,
    SOCKS,
    DHCP4,
    NTP,
    STUN,
    TURN,
    RTP,
    DTLS,
    WebRTC
}

pub fn tag_tcp_sessions(sessions: &mut MutexGuard<HashMap<L4Key, TcpSession>>,
                        bus: Arc<Bus>,
                        metrics: Arc<Mutex<Metrics>>) {
    for session in sessions.values_mut() {
        let mut client_to_server_data: Vec<u8> = vec![];
        let mut server_to_client_data: Vec<u8> = vec![];

        for segment_data in session.segments_client_to_server.values() {
            client_to_server_data.extend(segment_data);
        }

        for segment_data in session.segments_server_to_client.values() {
            server_to_client_data.extend(segment_data);
        }

        /*
         * The taggers should always be written extremely defensively and never throw any panics.
         * However, to increase reliability, we catch and log any panics that may still occur
         * because nobody is perfect.
         */
        let result = panic::catch_unwind(|| tag_all_tcp(
            &client_to_server_data,
            &server_to_client_data,
            session,
            &bus,
            &metrics
        ));

        match result {
            Ok(tags) => {
                // We overwrite the tags because they may have changed with more segments coming in.
                session.tags = tags
            },
            Err(e) => {
                match e.downcast_ref::<&str>() { Some(s) => {
                    error!("Could not tag TCP session {:?}: {}", session.session_key, s);
                } _ => { match e.downcast_ref::<String>() { Some(s) => {
                    error!("Could not tag TCP session {:?}: {}", session.session_key, s);
                } _ => {
                    error!("Could not tag TCP session {:?}. Panicked with an unknown type.",
                        session.session_key);
                }}}}
            }
        }
    }
}


pub fn tag_udp_sessions(conversations: &mut MutexGuard<HashMap<L4Key, UdpConversation>>,
                        bus: Arc<Bus>,
                        metrics: Arc<Mutex<Metrics>>) {
    for conversation in conversations.values_mut() {
        let mut client_to_server_data: Vec<u8> = vec![];
        let mut server_to_client_data: Vec<u8> = vec![];

        for segment_data in &conversation.datagrams_client_to_server {
            client_to_server_data.extend(segment_data);
        }
        for segment_data in &conversation.datagrams_server_to_client {
            server_to_client_data.extend(segment_data);
        }


        /*
         * The taggers should always be written extremely defensively and never throw any panics.
         * However, to increase reliability, we catch and log any panics that may still occur
         * because nobody is perfect.
         */
        let result = panic::catch_unwind(|| tag_all_udp(
            &client_to_server_data,
            &server_to_client_data,
            conversation,
            &bus,
            &metrics
        ));

        match result {
            Ok((tags, commands)) => {
                // UDP tags are extended because there is tagging already fed by initial datagrams.
                conversation.tags.extend(tags);

                // Apply any commands issued by the taggers.
                for command in commands {
                    command.apply(conversation);
                }

                if conversation.tags.contains(&STUN)
                    && (conversation.tags.contains(&RTP) || conversation.tags.contains(&DTLS)) {
                    // We can safely assume the flow is WebRTC when STUN combined with RTP or DTLS.
                    conversation.tags.insert(L7Tag::WebRTC);
                }
            },
            Err(e) => {
                match e.downcast_ref::<&str>() { Some(s) => {
                    error!("Could not tag UDP session: {}", s);
                } _ => { match e.downcast_ref::<String>() { Some(s) => {
                    error!("Could not tag UDP session: {}", s);
                } _ => {
                    error!("Could not tag UDP session. Panicked with an unknown type.");
                }}}}
            }
        }
    }
}

fn tag_all_tcp(client_to_server: &[u8],
               server_to_client: &[u8],
               session: &TcpSession,
               bus: &Arc<Bus>,
               metrics: &Arc<Mutex<Metrics>>) -> HashSet<L7Tag> {
    let mut tags = HashSet::new();

    // HTTP.
    let mut http_timer_untagged = Timer::new();
    let mut http_timer_tagged = Timer::new();
    if http_tagger::tag(client_to_server, server_to_client).is_some() {
        http_timer_tagged.stop();
        record_timer(
            http_timer_tagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.http.tagged",
            metrics
        );

        tags.extend([HTTP, Unencrypted]);
    } else {
        http_timer_untagged.stop();
        record_timer(
            http_timer_untagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.http.untagged",
            metrics
        );
    }

    // SOCKS.
    let mut socks_timer_untagged = Timer::new();
    let mut socks_timer_tagged = Timer::new();
    if let Some(socks) = socks_tagger::tag(client_to_server, server_to_client, session) {
        socks_timer_tagged.stop();
        record_timer(
            socks_timer_tagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.socks.tagged",
            metrics
        );

        let len = socks.estimate_struct_size();
        to_pipeline!(
            WiredChannelName::SocksPipeline,
            bus.socks_pipeline.sender,
            Arc::new(socks),
            len
        );

        tags.extend([SOCKS]);
    } else {
        socks_timer_untagged.stop();
        record_timer(
            socks_timer_untagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.socks.untagged",
            metrics
        );
    }
    
    // SSH.
    let mut ssh_timer_untagged = Timer::new();
    let mut ssh_timer_tagged = Timer::new();
    if let Some(ssh) = ssh_tagger::tag(client_to_server, server_to_client, session) {
        ssh_timer_tagged.stop();
        record_timer(
            ssh_timer_tagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.ssh.tagged",
            metrics
        );

        let len = ssh.estimate_struct_size();
        to_pipeline!(
            WiredChannelName::SshPipeline,
            bus.ssh_pipeline.sender,
            Arc::new(ssh),
            len
        );
        
        tags.extend([SSH]);
    } else {
        ssh_timer_untagged.stop();
        record_timer(
            ssh_timer_untagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.ssh.untagged",
            metrics
        );
    }

    // RTSP.
    let mut rtsp_timer_untagged = Timer::new();
    let mut rtsp_timer_tagged = Timer::new();
    if let Some(rtsp) = rtsp_tagger::tag(client_to_server, server_to_client, session) {
        rtsp_timer_tagged.stop();
        record_timer(
            rtsp_timer_tagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.rtsp.tagged",
            metrics
        );

        let len = rtsp.estimate_struct_size();
        to_pipeline!(
            WiredChannelName::RtspPipeline,
            bus.rtsp_pipeline.sender,
            Arc::new(rtsp),
            len
        );

        tags.extend([RTSP, Unencrypted]);
    } else {
        rtsp_timer_untagged.stop();
        record_timer(
            rtsp_timer_untagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.rtsp.untagged",
            metrics
        );
    }

    // STUN / TURN.
    let mut stun_timer_untagged = Timer::new();
    let mut stun_timer_tagged = Timer::new();
    if let Some(stun) = stun_tagger::tag_tcp(client_to_server, server_to_client, session) {
        stun_timer_tagged.stop();
        record_timer(
            stun_timer_tagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.stun.tagged",
            metrics
        );

        tags.extend([STUN]);
        if stun.is_turn {
            tags.extend([TURN]);
        }

        let len = stun.estimate_struct_size();
        to_pipeline!(
            WiredChannelName::StunPipeline,
            bus.stun_pipeline.sender,
            Arc::new(stun),
            len
        );
    } else {
        stun_timer_untagged.stop();
        record_timer(
            stun_timer_untagged.elapsed_microseconds(),
            "tables.tcp.timer.sessions.tagging.stun.untagged",
            metrics
        );
    }

    tags
}

fn tag_all_udp(client_to_server: &[u8],
               server_to_client: &[u8],
               conversation: &UdpConversation,
               bus: &Arc<Bus>,
               metrics: &Arc<Mutex<Metrics>>) -> (HashSet<L7Tag>, Vec<TaggerCommand>) {
    let mut tags = HashSet::new();
    let mut commands = Vec::new();

    let mut stun_timer_untagged = Timer::new();
    let mut stun_timer_tagged = Timer::new();
    let mut stun_negotiation_key: Option<String> = None; // Needed in WebRTC later.
    if let Some((stun, stun_commands, stun_neg_key)) =
        stun_tagger::tag_udp(client_to_server, server_to_client, conversation) {

        stun_timer_tagged.stop();
        record_timer(stun_timer_tagged.elapsed_microseconds(),
                     "tables.udp.timer.sessions.tagging.stun.tagged", metrics);

        tags.extend([STUN]);
        if stun.is_turn { tags.extend([TURN]); }
        commands.extend(stun_commands);

        let len = stun.estimate_struct_size();
        to_pipeline!(WiredChannelName::StunPipeline, bus.stun_pipeline.sender, Arc::new(stun), len);

        stun_negotiation_key = stun_neg_key;
    } else {
        stun_timer_untagged.stop();
        record_timer(stun_timer_untagged.elapsed_microseconds(),
                     "tables.udp.timer.sessions.tagging.stun.untagged", metrics);
    }

    // Taggers on conversations with recent buffer enabled.
    if conversation.capture_recent {
        let recent_c2s: Vec<Vec<u8>> =
            conversation.recent_client_to_server.iter().cloned().collect();
        let recent_s2c: Vec<Vec<u8>> =
            conversation.recent_server_to_client.iter().cloned().collect();

        // RTP. (Currently only used in WebRTC below, not in its own pipeline)
        let mut rtp_timer = Timer::new();
        let rtp_streams: Vec<RtpStream> = match rtp_tagger::tag(&recent_c2s, &recent_s2c) {
            Some(streams) => {
                rtp_timer.stop();
                record_timer(rtp_timer.elapsed_microseconds(),
                             "tables.udp.timer.sessions.tagging.rtp.tagged", metrics);
                tags.insert(RTP);

                if let Some(dir) = rtp_tagger::dominant_direction(&streams) {
                    let (address, port) = match dir {
                        rtp_tagger::RtpDirection::ClientToServer =>
                            (conversation.source_address, conversation.source_port),
                        rtp_tagger::RtpDirection::ServerToClient =>
                            (conversation.destination_address, conversation.destination_port),
                    };
                    commands.push(TaggerCommand::OrientClientTo { address, port });
                }

                streams
            }
            None => {
                rtp_timer.stop();
                record_timer(rtp_timer.elapsed_microseconds(),
                             "tables.udp.timer.sessions.tagging.rtp.untagged", metrics);
                Vec::new()
            }
        };

        // DTLS. (Currently only used in WebRTC below, not in its own pipeline)
        let mut dtls_timer = Timer::new();
        let dtls_result = dtls_tagger::tag(&recent_c2s, &recent_s2c);
        if dtls_result.is_some() {
            dtls_timer.stop();
            record_timer(dtls_timer.elapsed_microseconds(),
                         "tables.udp.timer.sessions.tagging.dtls.tagged", metrics);
            tags.insert(DTLS);
        } else {
            dtls_timer.stop();
            record_timer(dtls_timer.elapsed_microseconds(),
                         "tables.udp.timer.sessions.tagging.dtls.untagged", metrics);
        }

        // WebRTC.
        let has_rtp = !rtp_streams.is_empty();
        let has_dtls = dtls_result.is_some();
        if tags.contains(&STUN) && (has_rtp || has_dtls) {
            match &stun_negotiation_key {
                Some(key) => {
                    let webrtc = WebRtcConversation::build(
                        conversation,
                        key.clone(),
                        rtp_streams,
                        dtls_result.map(|d| d.app_data_records).unwrap_or(0),
                    );
                    let len = webrtc.estimate_struct_size();
                    to_pipeline!(WiredChannelName::WebRtcPipeline, bus.webrtc_pipeline.sender, Arc::new(webrtc), len);
                }
                None => {
                    debug!("WebRTC media/data on a STUN flow with no negotiation key; skipping.");
                }
            }
        }
    }

    (tags, commands)
}