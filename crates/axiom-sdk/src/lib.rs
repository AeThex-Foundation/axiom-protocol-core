//! # Axiom SDK
//!
//! The official Rust SDK for the [Axiom Protocol](https://aethex.tech).
//!
//! This crate re-exports everything you need to build on or interact with
//! the Axiom Protocol in a single dependency:
//!
//! ```toml
//! [dependencies]
//! axiom-sdk = "0.1"
//! ```
//!
//! ## Quick start
//!
//! ```rust
//! use axiom_sdk::prelude::*;
//!
//! // Generate a keypair
//! let keypair = Keypair::generate();
//! let address = Address::from_public_key(&keypair.public_key());
//!
//! // Build a validator set and spin up a consensus engine
//! let validators = vec![
//!     Validator { id: keypair.public_key(), voting_power: 100 },
//! ];
//! let vset = ValidatorSet::new(validators);
//! let engine = Engine::new(ChainConfig::default(), vset, 1);
//! ```
//!
//! ## Modules
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`crypto`] | Hashing, keypairs, signatures |
//! | [`core`] | Block, transaction, address, chain config |
//! | [`consensus`] | Validator set, votes, quorum certs, engine |
//! | [`state`] | Account ledger, Merkle trie |
//! | [`network`] | P2P messages, peer identity, wire codec |

pub use axiom_crypto as crypto;
pub use axiom_core as core;
pub use axiom_consensus as consensus;
pub use axiom_state as state;
pub use axiom_network as network;

/// Convenient single-import prelude.
///
/// ```rust
/// use axiom_sdk::prelude::*;
/// ```
pub mod prelude {
    // Crypto
    pub use axiom_crypto::{Hash, Hasher, Keypair, PublicKey, Signature};

    // Core types
    pub use axiom_core::{
        ChainConfig,
        ProtocolError,
        types::{
            Address, Block, BlockHeader, ChainId, Epoch, Height,
            Round, Timestamp, Transaction, TxHash, ValidatorId,
        },
    };

    // Consensus
    pub use axiom_consensus::{
        Engine, QuorumCert, ValidatorSet,
        validator_set::Validator,
        Vote, VoteType,
    };

    // State
    pub use axiom_state::{Account, Ledger, SparseMerkleTrie};

    // Network
    pub use axiom_network::{AxiomCodec, Message, PeerId, PeerInfo};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn prelude_keypair_and_address() {
        let kp = Keypair::generate();
        let addr = Address::from_public_key(&kp.public_key());
        let sig = kp.sign(b"axiom");
        assert!(kp.public_key().verify(b"axiom", &sig).is_ok());
        let _ = addr;
    }

    #[test]
    fn prelude_ledger_transfer() {
        let ledger = Ledger::new();
        let kp = Keypair::generate();
        let addr = Address::from_public_key(&kp.public_key());
        let mut acc = Account::new(addr);
        acc.balance = 1000;
        ledger.set(acc);
        assert_eq!(ledger.get(&addr).unwrap().balance, 1000);
    }

    #[test]
    fn prelude_consensus_engine() {
        let kp = Keypair::generate();
        let validators = vec![Validator { id: kp.public_key(), voting_power: 100 }];
        let vset = ValidatorSet::new(validators);
        let engine = Engine::new(ChainConfig::default(), vset, 1);
        assert_eq!(engine.height(), 1);
    }
}
