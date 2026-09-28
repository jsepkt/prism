use prism_core::{Block, Transaction};
use serde::{Deserialize, Serialize};

/// Wire messages exchanged between Prism peer nodes over TCP
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum P2pMessage {
    /// Initial peer greeting
    Handshake {
        node_id: String,
        p2p_port: u16,
        current_height: u64,
    },
    /// Acknowledgment of handshake
    HandshakeAck {
        node_id: String,
        current_height: u64,
    },
    /// Gossip a newly produced or verified block
    BroadcastBlock {
        block: Block,
    },
    /// Gossip a pending transaction to the network mempool
    BroadcastTransaction {
        transaction: Transaction,
    },
    /// Request blocks to catch up after joining the network
    GetBlocksRequest {
        from_height: u64,
        max_count: u32,
    },
    /// Response containing chain blocks
    GetBlocksResponse {
        blocks: Vec<Block>,
    },
    /// Keep-alive ping
    Ping,
    /// Keep-alive pong
    Pong,
}
