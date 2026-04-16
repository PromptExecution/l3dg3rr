use ledger_core::ingest::{deterministic_tx_id, IngestedLedger, TransactionInput};

#[test]
fn tx_id_is_deterministic_for_same_input() {
    let tx = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: "2023-taxes/WF--BH-CHK--2023-01--statement.rkyv".to_string(),
    };

    let a = deterministic_tx_id(&tx);
    let b = deterministic_tx_id(&tx);
    assert_eq!(a, b);
}

#[test]
fn reingest_is_idempotent() {
    let tx = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: "2023-taxes/WF--BH-CHK--2023-01--statement.rkyv".to_string(),
    };

    let mut ledger = IngestedLedger::default();
    let first = ledger.ingest(std::slice::from_ref(&tx));
    let second = ledger.ingest(&[tx]);

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 0);
}
