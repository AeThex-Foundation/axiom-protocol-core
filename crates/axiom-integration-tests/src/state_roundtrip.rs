use axiom_core::types::{Address, Transaction};
use axiom_crypto::Keypair;
use axiom_state::{Account, Ledger};

fn addr(seed: u8) -> Address {
    Address::from_public_key(&Keypair::from_bytes(&[seed; 32]).public_key())
}

fn dummy_tx(from: Address, to: Address, value: u128, nonce: u64) -> Transaction {
    let kp = Keypair::from_bytes(&[1u8; 32]);
    Transaction {
        chain_id: "axiom-devnet-1".into(),
        nonce,
        from,
        to,
        value,
        gas_limit: 21_000,
        gas_price: 0,
        data: vec![],
        signature: kp.sign(b"dummy"),
    }
}

#[test]
fn multi_transfer_state_root() {
    let ledger = Ledger::new();
    let genesis = addr(0);
    let alice = addr(1);
    let bob = addr(2);

    let mut gen_acc = Account::new(genesis);
    gen_acc.balance = 1_000_000_000;
    ledger.set(gen_acc);

    ledger.apply_tx(&dummy_tx(genesis, alice, 100_000, 0)).unwrap();
    ledger.apply_tx(&dummy_tx(genesis, bob, 200_000, 1)).unwrap();

    let root_after_setup = ledger.state_root();

    ledger.apply_tx(&dummy_tx(alice, bob, 50_000, 0)).unwrap();

    assert_ne!(ledger.state_root(), root_after_setup);
    assert_eq!(ledger.get(&alice).unwrap().balance, 50_000);
    assert_eq!(ledger.get(&bob).unwrap().balance, 250_000);
}

#[test]
fn ledger_snapshot_revert_on_bad_tx() {
    let ledger = Ledger::new();
    let alice = addr(1);
    let bob = addr(2);

    let mut acc = Account::new(alice);
    acc.balance = 1000;
    ledger.set(acc);

    let snap = ledger.snapshot();
    // This transaction should fail (too little balance)
    let bad_tx = dummy_tx(alice, bob, 5000, 0);
    assert!(ledger.apply_tx(&bad_tx).is_err());
    // Even after failure, internal state was not modified (error returned early)
    assert_eq!(ledger.get(&alice).unwrap().balance, 1000);
    drop(snap);
}
