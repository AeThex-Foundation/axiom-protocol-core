use crate::{
    quorum::{AddResult, QuorumCert, VoteAccumulator},
    ValidatorSet, Vote, VoteType,
};
use axiom_core::{
    config::ChainConfig,
    types::{Block, Height, Round},
};
use axiom_crypto::Hash;
use parking_lot::Mutex;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Consensus phase within a single round.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Propose,
    Prevote,
    Precommit,
    Commit,
}

/// Per-height round state.
#[derive(Debug)]
struct RoundState {
    round: Round,
    phase: Phase,
    proposal: Option<Block>,
    prevotes: VoteAccumulator,
    precommits: VoteAccumulator,
    locked_hash: Option<Hash>,
    locked_round: Option<Round>,
}

impl RoundState {
    fn new(round: Round) -> Self {
        RoundState {
            round,
            phase: Phase::Propose,
            proposal: None,
            prevotes: VoteAccumulator::default(),
            precommits: VoteAccumulator::default(),
            locked_hash: None,
            locked_round: None,
        }
    }
}

/// Consensus events that the Engine can emit.
#[derive(Debug)]
pub enum ConsensusEvent {
    /// Engine has committed a block at a height.
    Committed { height: Height, block: Block, qc: QuorumCert },
    /// Engine needs the application to create a proposal.
    NeedProposal { height: Height, round: Round },
    /// Engine timed out and advanced to a new round.
    RoundTimeout { height: Height, round: Round },
    /// Equivocation detected.
    Equivocation { validator: String, height: Height },
}

/// The core consensus state machine.
///
/// Thread-safe: all mutable state is behind a `Mutex`.
pub struct Engine {
    config: ChainConfig,
    validator_set: ValidatorSet,
    inner: Mutex<EngineInner>,
}

struct EngineInner {
    height: Height,
    round_state: RoundState,
    committed: Vec<(Height, Hash)>,
}

impl Engine {
    pub fn new(config: ChainConfig, validator_set: ValidatorSet, start_height: Height) -> Self {
        let inner = EngineInner {
            height: start_height,
            round_state: RoundState::new(0),
            committed: Vec::new(),
        };
        Engine { config, validator_set, inner: Mutex::new(inner) }
    }

    pub fn height(&self) -> Height {
        self.inner.lock().height
    }

    pub fn round(&self) -> Round {
        self.inner.lock().round_state.round
    }

    /// Set a received block proposal for the current round.
    pub fn receive_proposal(&self, block: Block) -> Option<ConsensusEvent> {
        let mut inner = self.inner.lock();
        let height = inner.height;
        let round = inner.round_state.round;
        let phase = inner.round_state.phase;

        if phase != Phase::Propose {
            debug!("ignoring late proposal in phase {:?}", phase);
            return None;
        }
        if block.height() != height {
            warn!("proposal height mismatch: {} vs {}", block.height(), height);
            return None;
        }

        info!(height, round, hash = %block.hash(), "received proposal");
        inner.round_state.proposal = Some(block);
        inner.round_state.phase = Phase::Prevote;
        None
    }

    /// Process an incoming vote; returns an event if a threshold was crossed.
    pub fn receive_vote(&self, vote: Vote) -> Option<ConsensusEvent> {
        let mut inner = self.inner.lock();
        let threshold = self.config.quorum_threshold(self.validator_set.total_power());
        let height = inner.height;
        let round = inner.round_state.round;

        if vote.height != height || vote.round != round {
            debug!("ignoring vote for h={} r={}", vote.height, vote.round);
            return None;
        }

        let rs = &mut inner.round_state;

        match vote.vote_type {
            VoteType::Prevote => {
                if rs.add_prevote(vote, &self.validator_set) == AddResult::Added {
                    if let Some(hash) = rs.prevote_quorum_hash(&self.validator_set, threshold) {
                        info!(height, round, %hash, "prevote quorum reached");
                        rs.locked_hash = Some(hash);
                        rs.locked_round = Some(round);
                        rs.phase = Phase::Precommit;
                    }
                }
            }
            VoteType::Precommit => {
                let block_hash = vote.block_hash;
                if rs.add_precommit(vote, &self.validator_set) == AddResult::Added {
                    if let Some(hash) = block_hash {
                        if rs.precommits.has_quorum(Some(hash), &self.validator_set, threshold) {
                            let qc = QuorumCert::try_build(
                                &rs.precommits,
                                height,
                                round,
                                hash,
                                &self.validator_set,
                                threshold,
                            )?;
                            let block = rs.proposal.clone()?;
                            info!(height, %hash, "precommit quorum — committing block");
                            inner.committed.push((height, hash));
                            inner.height += 1;
                            inner.round_state = RoundState::new(0);
                            return Some(ConsensusEvent::Committed { height, block, qc });
                        }
                    }
                }
            }
        }
        None
    }

    /// Advance to the next round (called on timeout).
    pub fn timeout(&self) -> ConsensusEvent {
        let mut inner = self.inner.lock();
        let height = inner.height;
        let old_round = inner.round_state.round;
        let new_round = old_round + 1;
        inner.round_state = RoundState::new(new_round);
        warn!(height, old_round, new_round, "consensus timeout — advancing round");
        ConsensusEvent::RoundTimeout { height, round: old_round }
    }

    pub fn committed_blocks(&self) -> Vec<(Height, Hash)> {
        self.inner.lock().committed.clone()
    }
}

impl RoundState {
    fn add_prevote(&mut self, vote: Vote, vset: &ValidatorSet) -> AddResult {
        self.prevotes.add(vote, vset)
    }

    fn add_precommit(&mut self, vote: Vote, vset: &ValidatorSet) -> AddResult {
        self.precommits.add(vote, vset)
    }

    fn prevote_quorum_hash(&self, vset: &ValidatorSet, threshold: u64) -> Option<Hash> {
        for h in self.prevotes.candidate_hashes() {
            if self.prevotes.has_quorum(Some(h), vset, threshold) {
                return Some(h);
            }
        }
        None
    }
}

// Expose a shared Engine handle.
pub type SharedEngine = Arc<Engine>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator_set::Validator;
    use axiom_core::{
        config::ChainConfig,
        types::{Block, BlockHeader},
    };
    use axiom_crypto::{Hash, Keypair};

    fn test_setup(n: usize) -> (SharedEngine, ValidatorSet, Vec<Keypair>) {
        let kps: Vec<Keypair> = (0..n as u8).map(|i| Keypair::from_bytes(&[i + 1; 32])).collect();
        let validators: Vec<Validator> = kps
            .iter()
            .map(|kp| Validator { id: kp.public_key(), voting_power: 100 })
            .collect();
        let vset = ValidatorSet::new(validators);
        let engine = Arc::new(Engine::new(ChainConfig::default(), vset.clone(), 1));
        (engine, vset, kps)
    }

    fn make_block(height: u64, proposer: &Keypair) -> Block {
        let header = BlockHeader {
            chain_id: "axiom-devnet-1".into(),
            height,
            epoch: 0,
            round: 0,
            timestamp: 0,
            parent_hash: Hash::ZERO,
            tx_root: Hash::ZERO,
            state_root: Hash::ZERO,
            quorum_cert: Hash::ZERO,
            proposer: proposer.public_key(),
        };
        let sig = proposer.sign(header.hash().as_bytes());
        Block { header, transactions: vec![], proposer_sig: sig }
    }

    #[test]
    fn full_consensus_round() {
        let (engine, _vset, kps) = test_setup(4);
        let block = make_block(1, &kps[0]);
        let block_hash = block.hash();

        engine.receive_proposal(block.clone());

        // 3 prevotes (quorum)
        for kp in kps.iter().take(3) {
            let v = Vote::sign(VoteType::Prevote, 1, 0, Some(block_hash), kp);
            engine.receive_vote(v);
        }

        // 3 precommits
        let mut result = None;
        for kp in kps.iter().take(3) {
            let v = Vote::sign(VoteType::Precommit, 1, 0, Some(block_hash), kp);
            result = engine.receive_vote(v);
        }

        assert!(matches!(result, Some(ConsensusEvent::Committed { height: 1, .. })));
        assert_eq!(engine.height(), 2);
    }
}
