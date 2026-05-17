use axiom_crypto::{Hash, PublicKey};
use serde::{Deserialize, Serialize};
use std::{fmt, net::SocketAddr};

/// Stable identifier for a peer: BLAKE3 hash of its public key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PeerId([u8; 32]);

impl PeerId {
    pub fn from_public_key(pk: &PublicKey) -> Self {
        PeerId(*Hash::of(pk.as_bytes()).as_bytes())
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Debug for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PeerId({}…)", &self.to_hex()[..12])
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

/// Current connection metadata for a peer.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: PeerId,
    pub addr: SocketAddr,
    pub public_key: PublicKey,
    pub protocol_version: u32,
    pub chain_id: String,
    /// Current chain height as last reported by the peer.
    pub latest_height: u64,
}

impl PeerInfo {
    pub fn new(pk: PublicKey, addr: SocketAddr, protocol_version: u32, chain_id: String) -> Self {
        PeerInfo {
            id: PeerId::from_public_key(&pk),
            addr,
            public_key: pk,
            protocol_version,
            chain_id,
            latest_height: 0,
        }
    }
}
