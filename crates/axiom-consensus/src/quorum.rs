use crate::{ValidatorSet, Vote, VoteType};
use axiom_core::types::{Height, Round};
use axiom_crypto::Hash;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Accumulated votes for a single (height, round, block_hash) triple.
#[derive(Debug, Default)]
pub struct VoteAccumulator {
    /// validator hex → vote
    votes: HashMap<String, Vote>,
}

impl VoteAccumulator {
    pub fn add(&mut self, vote: Vote, vset: &ValidatorSet) -> AddResult {
        if !vote.verify() {
            return AddResult::Invalid;
        }
        if !vset.contains(&vote.validator) {
            return AddResult::UnknownValidator;
        }
        let key = vote.validator.to_hex();
        if self.votes.contains_key(&key) {
            return AddResult::Duplicate;
        }
        self.votes.insert(key, vote);
        AddResult::Added
    }

    /// Total voting power of accumulated votes for `block_hash`.
    pub fn power_for(&self, block_hash: Option<Hash>, vset: &ValidatorSet) -> u64 {
        self.votes
            .values()
            .filter(|v| v.block_hash == block_hash)
            .map(|v| vset.voting_power_of(&v.validator))
            .sum()
    }

    pub fn has_quorum(&self, block_hash: Option<Hash>, vset: &ValidatorSet, threshold: u64) -> bool {
        self.power_for(block_hash, vset) >= threshold
    }

    pub fn votes_for(&self, block_hash: Option<Hash>) -> Vec<&Vote> {
        self.votes.values().filter(|v| v.block_hash == block_hash).collect()
    }

    /// All distinct non-nil block hashes that have received at least one vote.
    pub fn candidate_hashes(&self) -> Vec<Hash> {
        let mut seen = std::collections::HashSet::new();
        for v in self.votes.values() {
            if let Some(h) = v.block_hash {
                seen.insert(h);
            }
        }
        seen.into_iter().collect()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddResult {
    Added,
    Duplicate,
    Invalid,
    UnknownValidator,
}

/// A finalised quorum certificate: 2/3+ precommits for one block.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuorumCert {
    pub height: Height,
    pub round: Round,
    pub block_hash: Hash,
    pub votes: Vec<Vote>,
}

impl QuorumCert {
    /// Build a QC from accumulated precommit votes, if quorum is present.
    pub fn try_build(
        acc: &VoteAccumulator,
        height: Height,
        round: Round,
        block_hash: Hash,
        vset: &ValidatorSet,
        threshold: u64,
    ) -> Option<Self> {
        if !acc.has_quorum(Some(block_hash), vset, threshold) {
            return None;
        }
        let votes: Vec<Vote> = acc.votes_for(Some(block_hash)).into_iter().cloned().collect();
        Some(QuorumCert { height, round, block_hash, votes })
    }

    /// Verify every constituent vote and check voting power.
    pub fn verify(&self, vset: &ValidatorSet, threshold: u64) -> bool {
        let mut power = 0u64;
        for vote in &self.votes {
            if vote.vote_type != VoteType::Precommit {
                return false;
            }
            if vote.height != self.height || vote.round != self.round {
                return false;
            }
            if vote.block_hash != Some(self.block_hash) {
                return false;
            }
            if !vote.verify() {
                return false;
            }
            power += vset.voting_power_of(&vote.validator);
        }
        power >= threshold
    }

    /// Canonical hash of this QC (used as `quorum_cert` field in BlockHeader).
    pub fn hash(&self) -> Hash {
        let encoded = bincode::serialize(self).expect("qc serialization is infallible");
        Hash::of(&encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_crypto::Keypair;
    use crate::validator_set::Validator;

    fn make_vset(n: usize) -> (ValidatorSet, Vec<Keypair>) {
        let kps: Vec<Keypair> = (0..n as u8).map(|i| Keypair::from_bytes(&[i + 1; 32])).collect();
        let validators: Vec<Validator> = kps
            .iter()
            .map(|kp| Validator { id: kp.public_key(), voting_power: 100 })
            .collect();
        (ValidatorSet::new(validators), kps)
    }

    #[test]
    fn quorum_requires_2_3() {
        let (vset, kps) = make_vset(4);
        // 4 validators × 100 power = 400 total; threshold = 267 (⌊400 × 2/3⌋)
        let threshold = vset.total_power() * 2 / 3;
        let block_hash = Hash::of(b"test_block");
        let height = 1;
        let round = 0;

        let mut acc = VoteAccumulator::default();
        // Add 2 votes (200/400 power) — not enough
        for kp in kps.iter().take(2) {
            let v = Vote::sign(VoteType::Precommit, height, round, Some(block_hash), kp);
            acc.add(v, &vset);
        }
        assert!(!acc.has_quorum(Some(block_hash), &vset, threshold));

        // Add a 3rd vote — now 300/400 ≥ 267
        let v = Vote::sign(VoteType::Precommit, height, round, Some(block_hash), &kps[2]);
        acc.add(v, &vset);
        assert!(acc.has_quorum(Some(block_hash), &vset, threshold));
    }

    #[test]
    fn duplicate_vote_is_ignored() {
        let (vset, kps) = make_vset(2);
        let block_hash = Hash::of(b"b");
        let mut acc = VoteAccumulator::default();
        let v = Vote::sign(VoteType::Prevote, 1, 0, Some(block_hash), &kps[0]);
        assert_eq!(acc.add(v.clone(), &vset), AddResult::Added);
        assert_eq!(acc.add(v, &vset), AddResult::Duplicate);
    }
}
