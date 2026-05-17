use axiom_crypto::{Hash, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::fmt;

// ---------- Primitive aliases ----------

/// Chain identifier (e.g. "axiom-mainnet-1").
pub type ChainId = String;

/// Absolute block height (genesis = 0).
pub type Height = u64;

/// BFT consensus round within a height.
pub type Round = u32;

/// Consensus epoch number.
pub type Epoch = u64;

/// Unix timestamp in milliseconds.
pub type Timestamp = u64;

// ---------- Address ----------

/// A 20-byte account address derived from the public key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Address([u8; 20]);

impl Address {
    pub fn from_public_key(pk: &PublicKey) -> Self {
        let hash = Hash::of(pk.as_bytes());
        let mut addr = [0u8; 20];
        addr.copy_from_slice(&hash.as_bytes()[..20]);
        Address(addr)
    }

    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// ---------- ValidatorId ----------

/// Stable validator identity — the Ed25519 public key.
pub type ValidatorId = PublicKey;

// ---------- Transaction ----------

/// A signed state-transition request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub chain_id: ChainId,
    pub nonce: u64,
    pub from: Address,
    pub to: Address,
    pub value: u128,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub data: Vec<u8>,
    pub signature: Signature,
}

/// BLAKE3 hash of a serialised transaction.
pub type TxHash = Hash;

impl Transaction {
    /// Compute the canonical hash of this transaction.
    pub fn hash(&self) -> TxHash {
        let encoded = bincode::serialize(self).expect("tx serialization is infallible");
        Hash::of(&encoded)
    }

    /// The bytes that were signed: everything except the signature field.
    pub fn signing_bytes(&self) -> Vec<u8> {
        bincode::serialize(&TxSigningData {
            chain_id: &self.chain_id,
            nonce: self.nonce,
            from: &self.from,
            to: &self.to,
            value: self.value,
            gas_limit: self.gas_limit,
            gas_price: self.gas_price,
            data: &self.data,
        })
        .expect("signing bytes serialization is infallible")
    }
}

#[derive(Serialize)]
struct TxSigningData<'a> {
    chain_id: &'a str,
    nonce: u64,
    from: &'a Address,
    to: &'a Address,
    value: u128,
    gas_limit: u64,
    gas_price: u64,
    data: &'a [u8],
}

// ---------- BlockHeader ----------

/// Immutable metadata for a block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub chain_id: ChainId,
    pub height: Height,
    pub epoch: Epoch,
    pub round: Round,
    pub timestamp: Timestamp,
    /// Hash of the parent block header.
    pub parent_hash: Hash,
    /// Merkle root of all transactions in this block.
    pub tx_root: Hash,
    /// Root of the application state trie after applying this block.
    pub state_root: Hash,
    /// Aggregated BFT quorum certificate from the previous height.
    pub quorum_cert: Hash,
    /// Proposer validator public key.
    pub proposer: ValidatorId,
}

impl BlockHeader {
    /// Canonical hash — this is what validators sign.
    pub fn hash(&self) -> Hash {
        let encoded = bincode::serialize(self).expect("header serialization is infallible");
        Hash::of(&encoded)
    }
}

// ---------- Block ----------

/// A complete protocol block.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    /// Proposer's signature over the header hash.
    pub proposer_sig: Signature,
}

impl Block {
    pub fn hash(&self) -> Hash {
        self.header.hash()
    }

    pub fn height(&self) -> Height {
        self.header.height
    }

    pub fn is_genesis(&self) -> bool {
        self.header.height == 0
    }

    /// Compute the transaction Merkle root for `txs`.
    pub fn compute_tx_root(txs: &[Transaction]) -> Hash {
        if txs.is_empty() {
            return Hash::ZERO;
        }
        let leaves: Vec<[u8; 32]> = txs.iter().map(|tx| *tx.hash().as_bytes()).collect();
        merkle_root(&leaves)
    }
}

/// Simple binary Merkle tree root via BLAKE3.
fn merkle_root(leaves: &[[u8; 32]]) -> Hash {
    match leaves.len() {
        0 => Hash::ZERO,
        1 => Hash::from(leaves[0]),
        n => {
            let mid = n.next_power_of_two() / 2;
            let left = merkle_root(&leaves[..mid.min(n)]);
            let right = if mid < n { merkle_root(&leaves[mid..]) } else { left };
            Hash::of_many(&[left.as_bytes(), right.as_bytes()])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_crypto::Keypair;

    fn make_address(seed: u8) -> Address {
        let kp = Keypair::from_bytes(&[seed; 32]);
        Address::from_public_key(&kp.public_key())
    }

    #[test]
    fn address_from_public_key_is_deterministic() {
        let a1 = make_address(1);
        let a2 = make_address(1);
        assert_eq!(a1, a2);
    }

    #[test]
    fn address_from_different_keys_differs() {
        assert_ne!(make_address(1), make_address(2));
    }

    #[test]
    fn merkle_root_empty() {
        assert_eq!(Block::compute_tx_root(&[]), Hash::ZERO);
    }
}
