mod common;

use ledger_core::ingest::TransactionInput;
use ledgerr_mcp::{GetRawContextRequest, IngestPdfRequest, TurboLedgerService, TurboLedgerTools};

#[test]
fn mcp_01_ingest_pdf_returns_deterministic_tx_ids_from_real_ingest() {
    let workbook_path = common::unique_workbook_path("phase2-contract");
    let service =
        TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
            .unwrap();

    let dir = tempfile::tempdir().unwrap();
    let response = service
        .ingest_pdf(IngestPdfRequest {
            pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
            journal_path: dir.path().join("ledger.beancount"),
            workbook_path: dir.path().join("tax-ledger.xlsx"),
            raw_context_bytes: Some(b"abc".to_vec()),
            extracted_rows: vec![TransactionInput {
                account_id: "WF-BH-CHK".to_string(),
                date: "2023-01-15".to_string(),
                amount: "-42.11".to_string(),
                description: "Coffee Shop".to_string(),
                source_ref: dir.path().join("ctx.rkyv").display().to_string(),
            }],
        })
        .unwrap();

    assert_eq!(response.inserted_count, 1);
    assert_eq!(response.tx_ids.len(), 1);
}

#[test]
fn mcp_05_get_raw_context_returns_stored_rkyv_bytes() {
    let workbook_path = common::unique_workbook_path("phase2-raw-context");
    let service =
        TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
            .unwrap();

    let dir = tempfile::tempdir().unwrap();
    let rkyv = dir.path().join("ctx.rkyv");
    std::fs::write(&rkyv, b"evidence-bytes").unwrap();

    let response = service
        .get_raw_context(GetRawContextRequest { rkyv_ref: rkyv })
        .unwrap();
    assert_eq!(response.bytes, b"evidence-bytes");
}
