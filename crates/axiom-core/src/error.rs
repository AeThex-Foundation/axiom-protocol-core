use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    // Block errors
    #[error("invalid block at height {height}: {reason}")]
    InvalidBlock { height: u64, reason: String },

    #[error("block hash mismatch: expected {expected}, got {actual}")]
    BlockHashMismatch { expected: String, actual: String },

    #[error("block height out of order: expected {expected}, got {actual}")]
    HeightMismatch { expected: u64, actual: u64 },

    // Transaction errors
    #[error("invalid transaction {tx_hash}: {reason}")]
    InvalidTransaction { tx_hash: String, reason: String },

    #[error("transaction already known: {0}")]
    DuplicateTransaction(String),

    #[error("insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u128, need: u128 },

    #[error("nonce mismatch: expected {expected}, got {actual}")]
    NonceMismatch { expected: u64, actual: u64 },

    // Consensus errors
    #[error("unknown validator: {0}")]
    UnknownValidator(String),

    #[error("insufficient voting power: {power} / {threshold}")]
    InsufficientVotingPower { power: u64, threshold: u64 },

    #[error("equivocation detected for validator {0}")]
    Equivocation(String),

    // Crypto errors
    #[error("crypto error: {0}")]
    Crypto(#[from] axiom_crypto::CryptoError),

    // State errors
    #[error("state root mismatch")]
    StateRootMismatch,

    #[error("account not found: {0}")]
    AccountNotFound(String),

    // Generic
    #[error("codec error: {0}")]
    Codec(String),

    #[error("internal error: {0}")]
    Internal(String),
}
