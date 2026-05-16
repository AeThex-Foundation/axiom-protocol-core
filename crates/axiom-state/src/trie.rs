use axiom_crypto::Hash;
use std::collections::BTreeMap;

/// A sparse Merkle trie over 32-byte keys, using BLAKE3 for node hashing.
///
/// Leaf value is a 32-byte hash of the serialised account state.
/// Interior nodes hash as `H(left || right)`.
/// Missing subtrees hash as `Hash::ZERO`.
#[derive(Clone, Debug, Default)]
pub struct SparseMerkleTrie {
    leaves: BTreeMap<[u8; 32], [u8; 32]>,
}

impl SparseMerkleTrie {
    pub fn new() -> Self {
        SparseMerkleTrie::default()
    }

    pub fn insert(&mut self, key: [u8; 32], value: [u8; 32]) {
        self.leaves.insert(key, value);
    }

    pub fn get(&self, key: &[u8; 32]) -> Option<&[u8; 32]> {
        self.leaves.get(key)
    }

    pub fn remove(&mut self, key: &[u8; 32]) {
        self.leaves.remove(key);
    }

    pub fn len(&self) -> usize {
        self.leaves.len()
    }

    pub fn is_empty(&self) -> bool {
        self.leaves.is_empty()
    }

    /// Compute the Merkle root of all leaves.
    ///
    /// Leaves are sorted by key, then combined pairwise up the tree.
    pub fn root(&self) -> Hash {
        if self.leaves.is_empty() {
            return Hash::ZERO;
        }
        let leaves: Vec<[u8; 32]> = self.leaves.values().copied().collect();
        merkle_reduce(&leaves)
    }
}

fn merkle_reduce(nodes: &[[u8; 32]]) -> Hash {
    match nodes.len() {
        0 => Hash::ZERO,
        1 => Hash::from(nodes[0]),
        n => {
            let mid = n.next_power_of_two() / 2;
            let left = merkle_reduce(&nodes[..mid.min(n)]);
            let right = if mid < n {
                merkle_reduce(&nodes[mid..])
            } else {
                Hash::ZERO
            };
            Hash::of_many(&[left.as_bytes(), right.as_bytes()])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kv(k: u8, v: u8) -> ([u8; 32], [u8; 32]) {
        ([k; 32], [v; 32])
    }

    #[test]
    fn empty_root_is_zero() {
        assert_eq!(SparseMerkleTrie::new().root(), Hash::ZERO);
    }

    #[test]
    fn single_leaf_root() {
        let mut t = SparseMerkleTrie::new();
        let (k, v) = kv(1, 2);
        t.insert(k, v);
        assert_eq!(t.root(), Hash::from(v));
    }

    #[test]
    fn root_changes_on_insert() {
        let mut t = SparseMerkleTrie::new();
        t.insert(kv(1, 1).0, kv(1, 1).1);
        let r1 = t.root();
        t.insert(kv(2, 2).0, kv(2, 2).1);
        let r2 = t.root();
        assert_ne!(r1, r2);
    }

    #[test]
    fn deterministic_root() {
        let mut t1 = SparseMerkleTrie::new();
        let mut t2 = SparseMerkleTrie::new();
        for i in 0..8u8 {
            t1.insert([i; 32], [i * 2; 32]);
            t2.insert([i; 32], [i * 2; 32]);
        }
        assert_eq!(t1.root(), t2.root());
    }
}
