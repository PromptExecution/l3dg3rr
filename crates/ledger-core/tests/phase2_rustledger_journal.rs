use ledger_core::ingest::{IngestedLedger, TransactionInput};
use ledger_core::journal::JournalTransaction;

#[test]
fn beancount_entry_is_deterministic_and_contains_source_metadata() {
    let tx = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: "2023-taxes/WF--BH-CHK--2023-01--statement.rkyv".to_string(),
    };

    let journal_tx = JournalTransaction::from_input(&tx);
    let entry = journal_tx.to_beancount_entry();

    assert!(entry.contains("2023-01-15 * \"Imported\" \"Coffee Shop\""));
    assert!(entry.contains("txid: \""));
    assert!(entry.contains("source_ref: \"2023-taxes/WF--BH-CHK--2023-01--statement.rkyv\""));
    assert!(entry.contains("Assets:Bank:WF-BH-CHK -42.11 USD"));
    assert!(entry.contains("Equity:Suspense:Imported 42.11 USD"));
}

#[test]
fn ingest_to_journal_is_replay_safe() {
    let dir = tempfile::tempdir().unwrap();
    let journal_path = dir.path().join("ledger.beancount");

    let tx = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: "2023-taxes/WF--BH-CHK--2023-01--statement.rkyv".to_string(),
    };

    let mut ledger = IngestedLedger::default();

    let first = ledger
        .ingest_to_journal(std::slice::from_ref(&tx), &journal_path)
        .unwrap();
    let second = ledger.ingest_to_journal(&[tx], &journal_path).unwrap();

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 0);

    let contents = std::fs::read_to_string(journal_path).unwrap();
    let entry_count = contents.matches("* \"Imported\"").count();
    assert_eq!(entry_count, 1);
}
