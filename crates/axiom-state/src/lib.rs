//! Merkle state trie and account ledger for the Axiom Protocol.
//!
//! Provides:
//! - [`Account`]: mutable per-address state (balance, nonce, code)
//! - [`Ledger`]: in-memory state store with Merkle root computation
//! - [`StateTrie`]: sparse Merkle trie over 32-byte keys

pub mod account;
pub mod ledger;
pub mod trie;

pub use account::Account;
pub use ledger::Ledger;
pub use trie::SparseMerkleTrie;
