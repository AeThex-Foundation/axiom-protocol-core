use axiom_core::types::ValidatorId;
use serde::{Deserialize, Serialize};

/// A single validator with its current voting power.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub id: ValidatorId,
    pub voting_power: u64,
}

/// The ordered, weighted validator set for one epoch.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ValidatorSet {
    validators: Vec<Validator>,
    total_power: u64,
}

impl ValidatorSet {
    pub fn new(mut validators: Vec<Validator>) -> Self {
        validators.sort_by(|a, b| b.voting_power.cmp(&a.voting_power));
        let total_power = validators.iter().map(|v| v.voting_power).sum();
        ValidatorSet { validators, total_power }
    }

    pub fn is_empty(&self) -> bool {
        self.validators.is_empty()
    }

    pub fn len(&self) -> usize {
        self.validators.len()
    }

    pub fn total_power(&self) -> u64 {
        self.total_power
    }

    pub fn get(&self, id: &ValidatorId) -> Option<&Validator> {
        self.validators.iter().find(|v| &v.id == id)
    }

    pub fn contains(&self, id: &ValidatorId) -> bool {
        self.get(id).is_some()
    }

    pub fn voting_power_of(&self, id: &ValidatorId) -> u64 {
        self.get(id).map(|v| v.voting_power).unwrap_or(0)
    }

    /// Deterministic leader for (height, round): weighted round-robin.
    pub fn proposer(&self, height: u64, round: u32) -> Option<&Validator> {
        if self.validators.is_empty() {
            return None;
        }
        let idx = ((height + round as u64) % self.validators.len() as u64) as usize;
        Some(&self.validators[idx])
    }

    pub fn iter(&self) -> impl Iterator<Item = &Validator> {
        self.validators.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_crypto::Keypair;

    fn make_vset(n: usize) -> ValidatorSet {
        let validators: Vec<Validator> = (0..n as u8)
            .map(|i| Validator {
                id: Keypair::from_bytes(&[i + 1; 32]).public_key(),
                voting_power: 100,
            })
            .collect();
        ValidatorSet::new(validators)
    }

    #[test]
    fn total_power() {
        let vs = make_vset(4);
        assert_eq!(vs.total_power(), 400);
    }

    #[test]
    fn proposer_rotates() {
        let vs = make_vset(3);
        let p0 = vs.proposer(0, 0).unwrap().id;
        let p1 = vs.proposer(1, 0).unwrap().id;
        let p2 = vs.proposer(2, 0).unwrap().id;
        // Should rotate through the set
        assert_ne!(p0, p1);
        assert_ne!(p1, p2);
    }
}
