use crate::types::{ChainId, Epoch, Height};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Compile-time protocol parameters for a specific chain deployment.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Human-readable chain identifier (e.g. "axiom-mainnet-1").
    pub chain_id: ChainId,

    /// Maximum number of transactions per block.
    pub max_txs_per_block: usize,

    /// Maximum serialised block size in bytes.
    pub max_block_bytes: usize,

    /// Target block time.
    #[serde(with = "duration_millis")]
    pub block_time: Duration,

    /// BFT timeout before incrementing the round.
    #[serde(with = "duration_millis")]
    pub consensus_timeout: Duration,

    /// Number of blocks per epoch (used for validator-set rotation).
    pub epoch_length: Height,

    /// Minimum stake required to become a validator (in base units).
    pub min_validator_stake: u128,

    /// Fraction of voting power needed for a quorum (numerator/denominator).
    pub quorum_numerator: u64,
    pub quorum_denominator: u64,

    /// Maximum number of validators in the active set.
    pub max_validators: usize,

    /// Slash fraction for double-signing (basis points, i.e. 1/10000).
    pub slash_fraction_double_sign: u32,

    /// Slash fraction for liveness failures (basis points).
    pub slash_fraction_downtime: u32,

    /// Protocol semantic version.
    pub protocol_version: u32,
}

impl ChainConfig {
    /// Fraction of total voting power required to form a quorum.
    pub fn quorum_threshold(&self, total_power: u64) -> u64 {
        total_power * self.quorum_numerator / self.quorum_denominator
    }

    /// Whether a given height is the first block of a new epoch.
    pub fn is_epoch_boundary(&self, height: Height) -> bool {
        height > 0 && height % self.epoch_length == 0
    }

    pub fn epoch_of(&self, height: Height) -> Epoch {
        height / self.epoch_length
    }
}

impl Default for ChainConfig {
    fn default() -> Self {
        ChainConfig {
            chain_id: "axiom-devnet-1".into(),
            max_txs_per_block: 4096,
            max_block_bytes: 4 * 1024 * 1024, // 4 MiB
            block_time: Duration::from_millis(500),
            consensus_timeout: Duration::from_secs(2),
            epoch_length: 1000,
            min_validator_stake: 1_000_000_000_000, // 1 AXIOM in atto-units
            quorum_numerator: 2,
            quorum_denominator: 3,
            max_validators: 100,
            slash_fraction_double_sign: 500, // 5%
            slash_fraction_downtime: 1,      // 0.01%
            protocol_version: 1,
        }
    }
}

mod duration_millis {
    use serde::{Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_millis() as u64)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }

    use serde::Deserialize;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quorum_threshold_2_3() {
        let cfg = ChainConfig::default();
        assert_eq!(cfg.quorum_threshold(99), 66);
        assert_eq!(cfg.quorum_threshold(100), 66);
    }

    #[test]
    fn epoch_boundary_detection() {
        let cfg = ChainConfig::default();
        assert!(!cfg.is_epoch_boundary(0));
        assert!(!cfg.is_epoch_boundary(999));
        assert!(cfg.is_epoch_boundary(1000));
        assert!(cfg.is_epoch_boundary(2000));
    }

    #[test]
    fn default_config_serialises() {
        let cfg = ChainConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let cfg2: ChainConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.chain_id, cfg2.chain_id);
    }
}
