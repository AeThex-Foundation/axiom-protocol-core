//! BFT consensus engine for the Axiom Protocol.
//!
//! Implements a Tendermint-inspired single-leader PBFT variant:
//!
//! ```text
//! for each height h:
//!   for each round r:
//!     1. PROPOSE  – leader broadcasts a Block
//!     2. PREVOTE  – all validators vote on the proposal
//!     3. PRECOMMIT – validators lock if they saw 2/3+ prevotes
//!     4. COMMIT   – validators finalise if they saw 2/3+ precommits
//! ```
//!
//! Key types:
//! - [`ValidatorSet`]: the active set with voting power
//! - [`Vote`]: a signed prevote or precommit
//! - [`QuorumCert`]: an aggregate of 2/3+ precommits
//! - [`Engine`]: drives the state machine

pub mod engine;
pub mod quorum;
pub mod validator_set;
pub mod vote;

pub use engine::Engine;
pub use quorum::QuorumCert;
pub use validator_set::{Validator, ValidatorSet};
pub use vote::{Vote, VoteType};
