//! P2P networking layer for the Axiom Protocol.
//!
//! Provides:
//! - [`Message`]: all protocol wire messages
//! - [`PeerId`]: stable peer identity
//! - [`PeerInfo`]: connection metadata
//! - [`Codec`]: length-delimited bincode framing

pub mod codec;
pub mod message;
pub mod peer;

pub use codec::AxiomCodec;
pub use message::Message;
pub use peer::{PeerId, PeerInfo};
