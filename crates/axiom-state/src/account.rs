use axiom_core::types::Address;
use serde::{Deserialize, Serialize};

/// On-chain account state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    /// Token balance in base (atto) units.
    pub balance: u128,
    /// Monotonically increasing nonce to prevent replay attacks.
    pub nonce: u64,
    /// Keccak256 hash of the contract code, or zero for EOAs.
    pub code_hash: [u8; 32],
    /// Storage root (Merkle root of key→value pairs), or zero.
    pub storage_root: [u8; 32],
}

impl Account {
    pub fn new(address: Address) -> Self {
        Account {
            address,
            balance: 0,
            nonce: 0,
            code_hash: [0u8; 32],
            storage_root: [0u8; 32],
        }
    }

    pub fn is_eoa(&self) -> bool {
        self.code_hash == [0u8; 32]
    }

    pub fn credit(&mut self, amount: u128) {
        self.balance = self.balance.saturating_add(amount);
    }

    pub fn debit(&mut self, amount: u128) -> Result<(), InsufficientFunds> {
        if self.balance < amount {
            return Err(InsufficientFunds {
                have: self.balance,
                need: amount,
            });
        }
        self.balance -= amount;
        Ok(())
    }

    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }
}

#[derive(Debug, thiserror::Error)]
#[error("insufficient funds: have {have}, need {need}")]
pub struct InsufficientFunds {
    pub have: u128,
    pub need: u128,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axiom_crypto::Keypair;
    use axiom_core::types::Address;

    fn addr() -> Address {
        Address::from_public_key(&Keypair::from_bytes(&[42; 32]).public_key())
    }

    #[test]
    fn credit_debit() {
        let mut acc = Account::new(addr());
        acc.credit(1000);
        assert_eq!(acc.balance, 1000);
        acc.debit(400).unwrap();
        assert_eq!(acc.balance, 600);
        assert!(acc.debit(700).is_err());
    }

    #[test]
    fn nonce_increments() {
        let mut acc = Account::new(addr());
        assert_eq!(acc.nonce, 0);
        acc.increment_nonce();
        assert_eq!(acc.nonce, 1);
    }
}
