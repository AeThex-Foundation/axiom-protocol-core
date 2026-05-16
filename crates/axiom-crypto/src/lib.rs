//! Cryptographic primitives for the Axiom Protocol.
//!
//! This crate provides:
//! - [`Hash`]: 32-byte BLAKE3 digest with hex/base58 encoding helpers
//! - [`Keypair`] / [`PublicKey`] / [`Signature`]: Ed25519 signing
//! - [`Hasher`]: incremental BLAKE3 hashing

pub mod hash;
pub mod signature;

pub use hash::{Hash, Hasher};
pub use signature::{Keypair, PublicKey, Signature};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("invalid public key bytes")]
    InvalidPublicKey,
    #[error("invalid signature bytes")]
    InvalidSignature,
    #[error("signature verification failed")]
    VerificationFailed,
    #[error("hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),
}
