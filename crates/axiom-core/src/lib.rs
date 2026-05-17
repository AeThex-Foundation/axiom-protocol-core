//! Core types and traits for the Axiom Protocol.
//!
//! This crate defines the fundamental data structures shared across all
//! Axiom sub-systems: blocks, transactions, validators, and the protocol
//! configuration.

pub mod config;
pub mod error;
pub mod types;

pub use config::ChainConfig;
pub use error::ProtocolError;
pub use types::{
    Address, Block, BlockHeader, ChainId, Epoch, Height, Round, Timestamp,
    Transaction, TxHash, ValidatorId,
};
