use crate::{account::Account, trie::SparseMerkleTrie};
use axiom_core::types::{Address, Transaction};
use axiom_crypto::Hash;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use tracing::debug;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("account not found: {0}")]
    AccountNotFound(Address),

    #[error("nonce mismatch for {address}: expected {expected}, got {actual}")]
    NonceMismatch { address: Address, expected: u64, actual: u64 },

    #[error("insufficient balance for {address}: have {have}, need {need}")]
    InsufficientBalance { address: Address, have: u128, need: u128 },

    #[error("serialization error: {0}")]
    Serialization(String),
}

/// In-memory account ledger backed by a sparse Merkle trie.
///
/// Supports optimistic apply/revert via snapshot checkpoints.
#[derive(Default)]
pub struct Ledger {
    inner: Arc<RwLock<LedgerInner>>,
}

#[derive(Default)]
struct LedgerInner {
    accounts: HashMap<Address, Account>,
    trie: SparseMerkleTrie,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger::default()
    }

    /// Get or create an account.
    pub fn get_or_create(&self, address: Address) -> Account {
        self.inner
            .read()
            .accounts
            .get(&address)
            .cloned()
            .unwrap_or_else(|| Account::new(address))
    }

    pub fn get(&self, address: &Address) -> Option<Account> {
        self.inner.read().accounts.get(address).cloned()
    }

    pub fn set(&self, account: Account) {
        let mut inner = self.inner.write();
        let key = *account.address.as_bytes();
        let mut key32 = [0u8; 32];
        key32[..20].copy_from_slice(&key);
        let value = account_leaf_hash(&account);
        inner.trie.insert(key32, *value.as_bytes());
        inner.accounts.insert(account.address, account);
    }

    /// Apply a transfer transaction, returning an error if it is invalid.
    pub fn apply_tx(&self, tx: &Transaction) -> Result<(), LedgerError> {
        let mut sender = self.get_or_create(tx.from);
        let fee = tx.gas_limit as u128 * tx.gas_price as u128;
        let total_debit = tx.value.saturating_add(fee);

        if sender.nonce != tx.nonce {
            return Err(LedgerError::NonceMismatch {
                address: tx.from,
                expected: sender.nonce,
                actual: tx.nonce,
            });
        }
        if sender.balance < total_debit {
            return Err(LedgerError::InsufficientBalance {
                address: tx.from,
                have: sender.balance,
                need: total_debit,
            });
        }

        sender.balance -= total_debit;
        sender.nonce += 1;

        let mut recipient = self.get_or_create(tx.to);
        recipient.balance = recipient.balance.saturating_add(tx.value);

        debug!(
            from = %tx.from,
            to = %tx.to,
            value = tx.value,
            "applied tx"
        );

        self.set(sender);
        self.set(recipient);
        Ok(())
    }

    /// Current Merkle root of all account state.
    pub fn state_root(&self) -> Hash {
        self.inner.read().trie.root()
    }

    pub fn account_count(&self) -> usize {
        self.inner.read().accounts.len()
    }

    /// Snapshot the current state for potential revert.
    pub fn snapshot(&self) -> LedgerSnapshot {
        let inner = self.inner.read();
        LedgerSnapshot {
            accounts: inner.accounts.clone(),
            trie: inner.trie.clone(),
        }
    }

    pub fn restore(&self, snapshot: LedgerSnapshot) {
        let mut inner = self.inner.write();
        inner.accounts = snapshot.accounts;
        inner.trie = snapshot.trie;
    }
}

/// Opaque checkpoint for ledger revert.
pub struct LedgerSnapshot {
    accounts: HashMap<Address, Account>,
    trie: SparseMerkleTrie,
}

fn account_leaf_hash(account: &Account) -> Hash {
    let encoded =
        bincode::serialize(account).unwrap_or_else(|_| b"invalid".to_vec());
    Hash::of(&encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_core::types::Transaction;
    use axiom_crypto::Keypair;

    fn addr(seed: u8) -> Address {
        Address::from_public_key(&Keypair::from_bytes(&[seed; 32]).public_key())
    }

    fn make_tx(from: Address, to: Address, value: u128, nonce: u64) -> Transaction {
        let kp = Keypair::from_bytes(&[1u8; 32]);
        let dummy_sig = kp.sign(b"dummy");
        Transaction {
            chain_id: "test".into(),
            nonce,
            from,
            to,
            value,
            gas_limit: 21000,
            gas_price: 0,
            data: vec![],
            signature: dummy_sig,
        }
    }

    #[test]
    fn transfer_updates_balances() {
        let ledger = Ledger::new();
        let alice = addr(1);
        let bob = addr(2);

        let mut alice_acc = Account::new(alice);
        alice_acc.balance = 1_000_000;
        ledger.set(alice_acc);

        let tx = make_tx(alice, bob, 500_000, 0);
        ledger.apply_tx(&tx).unwrap();

        assert_eq!(ledger.get(&alice).unwrap().balance, 500_000);
        assert_eq!(ledger.get(&bob).unwrap().balance, 500_000);
    }

    #[test]
    fn insufficient_balance_fails() {
        let ledger = Ledger::new();
        let alice = addr(1);
        let bob = addr(2);
        let tx = make_tx(alice, bob, 1, 0);
        assert!(ledger.apply_tx(&tx).is_err());
    }

    #[test]
    fn snapshot_restore() {
        let ledger = Ledger::new();
        let a = addr(1);
        let mut acc = Account::new(a);
        acc.balance = 100;
        ledger.set(acc);

        let snap = ledger.snapshot();
        let mut acc2 = Account::new(a);
        acc2.balance = 999;
        ledger.set(acc2);
        assert_eq!(ledger.get(&a).unwrap().balance, 999);

        ledger.restore(snap);
        assert_eq!(ledger.get(&a).unwrap().balance, 100);
    }

    #[test]
    fn state_root_changes() {
        let ledger = Ledger::new();
        let r1 = ledger.state_root();
        let mut acc = Account::new(addr(5));
        acc.balance = 42;
        ledger.set(acc);
        assert_ne!(ledger.state_root(), r1);
    }
}
