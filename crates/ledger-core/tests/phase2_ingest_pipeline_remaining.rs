use calamine::Reader;
use ledger_core::ingest::{IngestedLedger, TransactionInput};

#[test]
fn ing_01_ingest_writes_journal_and_tx_sheet_projection() {
    let dir = tempfile::tempdir().unwrap();
    let journal = dir.path().join("ledger.beancount");
    let workbook = dir.path().join("tax-ledger.xlsx");

    let row = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: dir.path().join("ctx.rkyv").display().to_string(),
    };

    let mut ledger = IngestedLedger::default();
    let inserted = ledger
        .ingest_to_journal_and_workbook(&[row], &journal, &workbook)
        .unwrap();

    assert_eq!(inserted.len(), 1);

    let wb = calamine::open_workbook_auto(&workbook).unwrap();
    assert!(wb.sheet_names().iter().any(|s| s == "TX.WF-BH-CHK"));
}

#[test]
fn ing_02_reingest_has_no_duplicate_journal_or_tx_rows() {
    let dir = tempfile::tempdir().unwrap();
    let journal = dir.path().join("ledger.beancount");
    let workbook = dir.path().join("tax-ledger.xlsx");

    let row = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: dir.path().join("ctx.rkyv").display().to_string(),
    };

    let mut ledger = IngestedLedger::default();
    let first = ledger
        .ingest_to_journal_and_workbook(&[row.clone()], &journal, &workbook)
        .unwrap();
    let second = ledger
        .ingest_to_journal_and_workbook(&[row], &journal, &workbook)
        .unwrap();

    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 0);

    let content = std::fs::read_to_string(journal).unwrap();
    assert_eq!(content.matches("txid:").count(), 1);
}

#[test]
fn ing_03_ing_04_source_ref_is_persisted_and_attached_to_tx() {
    let dir = tempfile::tempdir().unwrap();
    let journal = dir.path().join("ledger.beancount");
    let workbook = dir.path().join("tax-ledger.xlsx");
    let source_ref = dir.path().join("evidence.rkyv").display().to_string();

    let row = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: source_ref.clone(),
    };

    std::fs::write(&source_ref, b"proof").unwrap();

    let mut ledger = IngestedLedger::default();
    let inserted = ledger
        .ingest_to_journal_and_workbook(&[row], &journal, &workbook)
        .unwrap();

    assert_eq!(inserted.len(), 1);
    assert_eq!(inserted[0].source_ref, source_ref);
}
