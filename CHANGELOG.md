# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).  
This project adheres to [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
- `axiom-crypto`: `Hash` (BLAKE3 32-byte digest), `Keypair`/`PublicKey`/`Signature` (Ed25519), `ZeroizeOnDrop` secret material
- `axiom-core`: `Block`, `BlockHeader`, `Transaction`, `Address`, `ChainConfig`, `ProtocolError`
- `axiom-consensus`: `ValidatorSet`, `Vote` (prevote/precommit), `VoteAccumulator`, `QuorumCert`, `Engine` state machine
- `axiom-state`: `Account`, `Ledger` with snapshot/restore, `SparseMerkleTrie`
- `axiom-network`: `Message` enum, `PeerId`, `PeerInfo`, `AxiomCodec` (length-prefixed bincode)
- Integration test suite: consensus happy path, round timeout, consecutive heights, state roundtrip
- GitHub Actions CI: fmt, clippy, test (Linux + macOS), security audit, docs
- GitHub Actions release workflow: multi-target builds + GitHub Release

---

## [0.1.0] — unreleased

Initial implementation of the Axiom Protocol core library.
