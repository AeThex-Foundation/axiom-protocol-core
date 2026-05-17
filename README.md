# Axiom Protocol Core

**Axiom** is a modular, Byzantine-fault-tolerant (BFT) consensus protocol written in Rust.  
It provides the foundational building blocks for building high-performance, secure distributed ledgers.

[![CI](https://github.com/aethex-foundation/axiom-protocol-core/actions/workflows/ci.yml/badge.svg)](https://github.com/aethex-foundation/axiom-protocol-core/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust: 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

---

## Architecture

```
axiom-protocol-core/
├── crates/
│   ├── axiom-crypto      — Hash (BLAKE3), Ed25519 keypairs & signatures
│   ├── axiom-core        — Block, Transaction, Address, ChainConfig
│   ├── axiom-consensus   — BFT engine: ValidatorSet, Vote, QuorumCert, Engine
│   ├── axiom-state       — Account ledger + sparse Merkle trie
│   └── axiom-network     — P2P wire protocol, length-framed codec
└── tests/integration/    — Cross-crate integration tests
```

Each crate has a single, focused responsibility and depends only on lower-level crates.

### Consensus overview

Axiom consensus is a Tendermint-inspired single-leader PBFT variant operating in rounds:

```
for each height h:
  for each round r:
    1. PROPOSE    — elected leader broadcasts a Block
    2. PREVOTE    — all validators sign a prevote (or nil)
    3. PRECOMMIT  — validators lock and precommit on 2/3+ prevotes
    4. COMMIT     — block is finalised on 2/3+ precommits → QuorumCert
```

Safety relies on Ed25519-signed votes; liveness relies on advancing rounds on timeout.

---

## Crate overview

| Crate | Purpose |
|---|---|
| `axiom-crypto` | `Hash` (BLAKE3 32-byte digest), `Keypair` / `PublicKey` / `Signature` (Ed25519) |
| `axiom-core` | `Block`, `BlockHeader`, `Transaction`, `Address`, `ChainConfig` |
| `axiom-consensus` | `ValidatorSet`, `Vote`, `VoteAccumulator`, `QuorumCert`, `Engine` |
| `axiom-state` | `Account`, `Ledger` (snapshot/restore), `SparseMerkleTrie` |
| `axiom-network` | `Message` enum, `PeerId`, `PeerInfo`, `AxiomCodec` (length-prefixed bincode) |

---

## Getting started

### Prerequisites

- Rust 1.75+ (`rustup update stable`)

### Build

```bash
git clone https://github.com/aethex-foundation/axiom-protocol-core
cd axiom-protocol-core
cargo build --release
```

### Test

```bash
# Unit tests for all crates
cargo test --all --lib

# Integration tests
cargo test --test '*'

# All tests with output
cargo test -- --nocapture
```

### Lint & format

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Configuration

`ChainConfig` (in `axiom-core`) controls all protocol parameters:

```rust
use axiom_core::ChainConfig;

let cfg = ChainConfig {
    chain_id: "axiom-mainnet-1".into(),
    max_txs_per_block: 4096,
    block_time: Duration::from_millis(500),
    epoch_length: 1000,
    min_validator_stake: 1_000_000_000_000,
    quorum_numerator: 2,
    quorum_denominator: 3,
    ..Default::default()
};
```

---

## Example: running the consensus engine

```rust
use std::sync::Arc;
use axiom_consensus::{Engine, ValidatorSet, Validator, Vote, VoteType};
use axiom_core::ChainConfig;
use axiom_crypto::Keypair;

// Build a 4-validator set
let keypairs: Vec<_> = (0..4u8).map(|i| Keypair::from_bytes(&[i+1; 32])).collect();
let validators: Vec<_> = keypairs.iter()
    .map(|kp| Validator { id: kp.public_key(), voting_power: 100 })
    .collect();
let vset = ValidatorSet::new(validators);

let engine = Arc::new(Engine::new(ChainConfig::default(), vset, 1));

// Receive a proposal (produced by your application layer)
// engine.receive_proposal(block);

// Feed votes from the network
// engine.receive_vote(vote);

// On timeout:
// engine.timeout();
```

---

## Security

- All votes are Ed25519-signed and verified before counting.
- `Keypair` secret bytes are `ZeroizeOnDrop` — memory is zeroed on drop.
- Double-signing (equivocation) is detectable via the `VoteAccumulator`.
- Please report vulnerabilities privately to **security@aethex.foundation**.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).  
Issues and pull requests are welcome.  
All contributors are expected to follow the [Contributor Covenant](https://www.contributor-covenant.org/).

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).
