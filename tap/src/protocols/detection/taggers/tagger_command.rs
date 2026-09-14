use std::net::IpAddr;
use crate::state::tables::udp_table::UdpConversation;

// A command a tagger can emit for the parent table to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaggerCommand {
    /*
     * The tagger has determined that the source and destination fields are swapped. This is
     * unlikely in TCP connections, but can easily happen in UDP conversations. The tagger
     * can determine and correct this mismatch because many tagged protocols provide reliable
     * client/server semantics.
     *
     * This command should only be issued in very high confidence situations.
     */
    OrientClientTo { address: IpAddr, port: u16 },

    /*
     * The tagger has determined that the detected protocol may carry additional, later-phase
     * protocols inside the flow. This command instructs the parent table to start filling
     * the lazy recent-window buffer for the flow.
     *
     * The recent-window buffer allows us to detect protocols after the default buffer of
     * the initially received bytes has filled up.
     */
    CaptureRecent
}

impl TaggerCommand {
    pub fn apply(&self, conv: &mut UdpConversation) {
        match *self {
            TaggerCommand::OrientClientTo { address, port } => {
                orient_client_to(conv, address, port);
            },
            TaggerCommand::CaptureRecent => {
                conv.capture_recent = true;
            }
        }
    }
}

// >> fn orient_express << @kroepke
fn orient_client_to(conv: &mut UdpConversation, address: IpAddr, port: u16) {
    let matches_source = conv.source_address == address && conv.source_port == port;
    let matches_dest = conv.destination_address == address && conv.destination_port == port;

    if matches_source {
        // Already oriented correctly. No-op.
        return;
    }

    if !matches_dest {
        // Endpoint is not part of this conversation.
        return;
    }

    std::mem::swap(&mut conv.source_address, &mut conv.destination_address);
    std::mem::swap(&mut conv.source_port, &mut conv.destination_port);
    std::mem::swap(&mut conv.source_mac, &mut conv.destination_mac);
    std::mem::swap(&mut conv.bytes_count_rx, &mut conv.bytes_count_tx);
    std::mem::swap(&mut conv.bytes_count_rx_incremental, &mut conv.bytes_count_tx_incremental);
    std::mem::swap(&mut conv.datagrams_client_to_server, &mut conv.datagrams_server_to_client);
}