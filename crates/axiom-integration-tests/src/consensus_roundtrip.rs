use axiom_consensus::{engine::ConsensusEvent, validator_set::Validator, Engine, ValidatorSet};
use axiom_core::{
    config::ChainConfig,
    types::{Block, BlockHeader},
};
use axiom_crypto::{Hash, Keypair};
use axiom_consensus::{Vote, VoteType};
use std::sync::Arc;

fn make_validator_set(n: usize) -> (ValidatorSet, Vec<Keypair>) {
    let kps: Vec<Keypair> = (0..n as u8).map(|i| Keypair::from_bytes(&[i + 1; 32])).collect();
    let validators: Vec<Validator> = kps
        .iter()
        .map(|kp| Validator { id: kp.public_key(), voting_power: 100 })
        .collect();
    (ValidatorSet::new(validators), kps)
}

fn make_block(height: u64, proposer: &Keypair) -> Block {
    let header = BlockHeader {
        chain_id: "axiom-devnet-1".into(),
        height,
        epoch: 0,
        round: 0,
        timestamp: 1_700_000_000_000,
        parent_hash: Hash::ZERO,
        tx_root: Hash::ZERO,
        state_root: Hash::ZERO,
        quorum_cert: Hash::ZERO,
        proposer: proposer.public_key(),
    };
    let sig = proposer.sign(header.hash().as_bytes());
    Block { header, transactions: vec![], proposer_sig: sig }
}

/// Full happy-path: 4 validators, 3 prevotes + 3 precommits → commit.
#[test]
fn happy_path_4_validators() {
    let (vset, kps) = make_validator_set(4);
    let engine = Arc::new(Engine::new(ChainConfig::default(), vset, 1));

    let block = make_block(1, &kps[0]);
    let block_hash = block.hash();

    engine.receive_proposal(block.clone());

    for kp in kps.iter().take(3) {
        let v = Vote::sign(VoteType::Prevote, 1, 0, Some(block_hash), kp);
        engine.receive_vote(v);
    }

    let mut last = None;
    for kp in kps.iter().take(3) {
        let v = Vote::sign(VoteType::Precommit, 1, 0, Some(block_hash), kp);
        last = engine.receive_vote(v);
    }

    assert!(
        matches!(last, Some(ConsensusEvent::Committed { height: 1, .. })),
        "expected commit, got {:?}",
        last
    );
    assert_eq!(engine.height(), 2, "engine should advance to next height");
}

/// Timeout advances the round.
#[test]
fn timeout_advances_round() {
    let (vset, _kps) = make_validator_set(3);
    let engine = Engine::new(ChainConfig::default(), vset, 1);

    assert_eq!(engine.round(), 0);
    engine.timeout();
    assert_eq!(engine.round(), 1);
    engine.timeout();
    assert_eq!(engine.round(), 2);
}

/// Two independent heights commit correctly.
#[test]
fn two_consecutive_heights() {
    let (vset, kps) = make_validator_set(3);
    let engine = Arc::new(Engine::new(ChainConfig::default(), vset, 1));

    for height in 1u64..=2 {
        let block = make_block(height, &kps[0]);
        let bh = block.hash();
        engine.receive_proposal(block);

        for kp in &kps {
            engine.receive_vote(Vote::sign(VoteType::Prevote, height, 0, Some(bh), kp));
        }
        let mut commit = None;
        for kp in &kps {
            let result = engine.receive_vote(Vote::sign(VoteType::Precommit, height, 0, Some(bh), kp));
            if result.is_some() {
                commit = result;
            }
        }
        assert!(
            matches!(commit, Some(ConsensusEvent::Committed { .. })),
            "expected commit at height {height}"
        );
    }
    assert_eq!(engine.height(), 3);
}
