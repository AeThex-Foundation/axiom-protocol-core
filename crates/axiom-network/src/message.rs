use crate::peer::PeerInfo;
use axiom_consensus::{QuorumCert, Vote};
use axiom_core::types::{Block, Height, Transaction};
use serde::{Deserialize, Serialize};

/// All messages exchanged between Axiom nodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    // --- Handshake ---
    /// Initial greeting; peers exchange identity and chain metadata.
    Hello(PeerInfo),
    /// Acknowledgement to Hello.
    HelloAck(PeerInfo),

    // --- Block propagation ---
    /// Broadcast a newly produced block proposal.
    ProposeBlock(Box<Block>),
    /// Response to a block request.
    Block(Box<Block>),
    /// Request a specific block by height.
    GetBlock { height: Height },

    // --- Consensus ---
    /// Prevote or precommit.
    Vote(Vote),
    /// Quorum certificate from a finalised height.
    QuorumCert(QuorumCert),

    // --- Transaction mempool ---
    /// Gossip a single pending transaction.
    Transaction(Transaction),
    /// Announce transaction hashes you have (pull-based gossip).
    HaveTxs { hashes: Vec<[u8; 32]> },
    /// Request full transactions by hash.
    GetTxs { hashes: Vec<[u8; 32]> },

    // --- State sync ---
    /// Request the peer's current height and state root.
    StatusRequest,
    /// Reply with height and state root.
    StatusResponse { height: Height, state_root: [u8; 32] },

    // --- Peer discovery ---
    /// Request a list of known peers.
    GetPeers,
    /// Reply with known peers.
    Peers(Vec<PeerInfo>),

    // --- Liveness ---
    Ping { nonce: u64 },
    Pong { nonce: u64 },
}

impl Message {
    pub fn kind(&self) -> &'static str {
        match self {
            Message::Hello(_) => "Hello",
            Message::HelloAck(_) => "HelloAck",
            Message::ProposeBlock(_) => "ProposeBlock",
            Message::Block(_) => "Block",
            Message::GetBlock { .. } => "GetBlock",
            Message::Vote(_) => "Vote",
            Message::QuorumCert(_) => "QuorumCert",
            Message::Transaction(_) => "Transaction",
            Message::HaveTxs { .. } => "HaveTxs",
            Message::GetTxs { .. } => "GetTxs",
            Message::StatusRequest => "StatusRequest",
            Message::StatusResponse { .. } => "StatusResponse",
            Message::GetPeers => "GetPeers",
            Message::Peers(_) => "Peers",
            Message::Ping { .. } => "Ping",
            Message::Pong { .. } => "Pong",
        }
    }
}
