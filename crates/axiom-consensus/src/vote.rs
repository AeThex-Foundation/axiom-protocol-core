use axiom_core::types::{Height, Round, ValidatorId};
use axiom_crypto::{Hash, Keypair, Signature};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoteType {
    Prevote,
    Precommit,
}

/// A validator's signed vote for a block at a specific height/round.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vote {
    pub vote_type: VoteType,
    pub height: Height,
    pub round: Round,
    /// `None` means a nil vote (no proposal seen / timeout).
    pub block_hash: Option<Hash>,
    pub validator: ValidatorId,
    pub signature: Signature,
}

impl Vote {
    /// Bytes that the validator signs.
    pub fn signing_bytes(
        vote_type: VoteType,
        height: Height,
        round: Round,
        block_hash: Option<Hash>,
    ) -> Vec<u8> {
        let type_byte: u8 = match vote_type {
            VoteType::Prevote => 0,
            VoteType::Precommit => 1,
        };
        let mut buf = Vec::with_capacity(1 + 8 + 4 + 33);
        buf.push(type_byte);
        buf.extend_from_slice(&height.to_le_bytes());
        buf.extend_from_slice(&round.to_le_bytes());
        match block_hash {
            Some(h) => {
                buf.push(1);
                buf.extend_from_slice(h.as_bytes());
            }
            None => buf.push(0),
        }
        buf
    }

    pub fn sign(
        vote_type: VoteType,
        height: Height,
        round: Round,
        block_hash: Option<Hash>,
        keypair: &Keypair,
    ) -> Self {
        let bytes = Self::signing_bytes(vote_type, height, round, block_hash);
        Vote {
            vote_type,
            height,
            round,
            block_hash,
            validator: keypair.public_key(),
            signature: keypair.sign(&bytes),
        }
    }

    pub fn verify(&self) -> bool {
        let bytes = Self::signing_bytes(self.vote_type, self.height, self.round, self.block_hash);
        self.validator.verify(&bytes, &self.signature).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_crypto::Keypair;

    #[test]
    fn vote_sign_verify() {
        let kp = Keypair::generate();
        let hash = Hash::of(b"block");
        let v = Vote::sign(VoteType::Prevote, 10, 0, Some(hash), &kp);
        assert!(v.verify());
    }

    #[test]
    fn nil_vote_verifies() {
        let kp = Keypair::generate();
        let v = Vote::sign(VoteType::Precommit, 5, 1, None, &kp);
        assert!(v.verify());
    }
}
