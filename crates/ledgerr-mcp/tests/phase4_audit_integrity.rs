mod common;

use ledger_core::ingest::TransactionInput;
use ledgerr_mcp::{
    ClassifyTransactionRequest, FlagStatusRequest, IngestPdfRequest, QueryAuditLogRequest,
    QueryFlagsRequest, ReconcileExcelClassificationRequest, TurboLedgerService, TurboLedgerTools,
};

fn service() -> TurboLedgerService {
    let workbook_path = common::unique_workbook_path("phase4-audit");
    TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
        .expect("manifest")
}

fn ingest_one(svc: &TurboLedgerService, description: &str, amount: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let source_ref = dir.path().join("ctx.rkyv");

    let ingest = svc
        .ingest_pdf(IngestPdfRequest {
            pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
            journal_path: dir.path().join("ledger.beancount"),
            workbook_path: dir.path().join("tax-ledger.xlsx"),
            raw_context_bytes: Some(b"ctx".to_vec()),
            extracted_rows: vec![TransactionInput {
                account_id: "WF-BH-CHK".to_string(),
                date: "2023-01-15".to_string(),
                amount: amount.to_string(),
                description: description.to_string(),
                source_ref: source_ref.display().to_string(),
            }],
        })
        .expect("ingest");
    ingest.tx_ids[0].clone()
}

#[test]
fn aud_01_mcp_02_classify_transaction_records_append_only_audit_entries() {
    let svc = service();
    let tx_id = ingest_one(&svc, "Coffee Shop", "-42.11");

    let first = svc
        .classify_transaction(ClassifyTransactionRequest {
            tx_id: tx_id.clone(),
            category: "Meals".to_string(),
            confidence: "0.91".to_string(),
            note: Some("initial classify".to_string()),
            actor: "agent".to_string(),
        })
        .expect("first classify");
    assert!(!first.audit_entries.is_empty());

    let second = svc
        .classify_transaction(ClassifyTransactionRequest {
            tx_id,
            category: "Meals".to_string(),
            confidence: "0.93".to_string(),
            note: Some("confidence update".to_string()),
            actor: "agent".to_string(),
        })
        .expect("second classify");

    let audit = svc
        .query_audit_log(QueryAuditLogRequest)
        .expect("audit query");

    assert!(audit.entries.len() >= first.audit_entries.len() + second.audit_entries.len());
}

#[test]
fn aud_02_excel_reconcile_path_writes_matching_audit_records() {
    let svc = service();
    let tx_id = ingest_one(&svc, "Stationery", "-15.00");

    svc.reconcile_excel_classification(ReconcileExcelClassificationRequest {
        tx_id,
        category: "OfficeSupplies".to_string(),
        confidence: "0.88".to_string(),
        actor: "excel-user".to_string(),
        note: Some("edited in workbook".to_string()),
    })
    .expect("reconcile");

    let audit = svc
        .query_audit_log(QueryAuditLogRequest)
        .expect("audit query");
    assert!(audit
        .entries
        .iter()
        .any(|e| e.actor == "excel-user" && e.note.as_deref() == Some("edited in workbook")));
}

#[test]
fn aud_03_decimal_safe_amount_and_confidence_validation_rejects_invalid_values() {
    let svc = service();
    let tx_id = ingest_one(&svc, "Bad Decimal", "-9.99");

    let err = svc
        .classify_transaction(ClassifyTransactionRequest {
            tx_id,
            category: "Misc".to_string(),
            confidence: "not-a-decimal".to_string(),
            note: None,
            actor: "agent".to_string(),
        })
        .expect_err("invalid decimal should fail");

    assert!(err.to_string().contains("invalid input"));
}

#[test]
fn aud_04_invariant_checks_detect_schema_or_txid_violations() {
    let svc = service();
    let _tx_id = ingest_one(&svc, "Coffee Shop", "-42.11");

    let err = svc
        .classify_transaction(ClassifyTransactionRequest {
            tx_id: "not-a-real-txid".to_string(),
            category: "".to_string(),
            confidence: "0.9".to_string(),
            note: None,
            actor: "agent".to_string(),
        })
        .expect_err("invalid invariants should fail");

    assert!(err.to_string().contains("invalid input"));
}

#[test]
fn phase4_keeps_phase3_open_flag_query_behavior() {
    let svc = service();
    let tx_id = ingest_one(&svc, "Fallback Merchant", "-101.00");

    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id,
        category: "Uncategorized".to_string(),
        confidence: "0.40".to_string(),
        note: None,
        actor: "agent".to_string(),
    })
    .expect("classify");

    let flags = svc
        .query_flags(QueryFlagsRequest {
            year: 2023,
            status: FlagStatusRequest::Open,
        })
        .expect("flag query");

    assert!(!flags.flags.is_empty());
}
