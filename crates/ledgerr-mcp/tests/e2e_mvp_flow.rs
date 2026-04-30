mod common;

use ledger_core::ingest::TransactionInput;
use ledgerr_mcp::{
    ClassifyTransactionRequest, FlagStatusRequest, GetScheduleSummaryRequest, IngestPdfRequest,
    QueryAuditLogRequest, QueryFlagsRequest, ScheduleKindRequest, TurboLedgerService,
    TurboLedgerTools,
};

fn service() -> TurboLedgerService {
    let workbook_path = common::unique_workbook_path("e2e-mvp");
    TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
        .expect("manifest")
}

#[test]
fn rel_03_e2e_mvp_flow_ingest_classify_audit_schedule() {
    let svc = service();
    let tmp = tempfile::tempdir().expect("tmp");
    let source_ref = tmp.path().join("ctx.rkyv");

    let ingest = svc
        .ingest_pdf(IngestPdfRequest {
            pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
            journal_path: tmp.path().join("ledger.beancount"),
            workbook_path: tmp.path().join("tax-ledger.xlsx"),
            ontology_path: None,
            raw_context_bytes: Some(b"ctx".to_vec()),
            extracted_rows: vec![TransactionInput {
                account_id: "WF-BH-CHK".to_string(),
                date: "2023-05-03".to_string(),
                amount: "-210.00".to_string(),
                description: "Office Depot".to_string(),
                source_ref: source_ref.display().to_string(),
            }],
        })
        .expect("ingest");
    assert_eq!(ingest.inserted_count, 1);
    let tx_id = ingest.tx_ids[0].clone();

    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id,
        category: "OfficeSupplies".to_string(),
        confidence: "0.93".to_string(),
        note: Some("e2e classify".to_string()),
        actor: "e2e".to_string(),
    })
    .expect("classify");

    let audit = svc.query_audit_log(QueryAuditLogRequest).expect("audit");
    assert!(!audit.entries.is_empty());

    let flags = svc
        .query_flags(QueryFlagsRequest {
            year: 2023,
            status: FlagStatusRequest::Open,
        })
        .expect("flags");
    assert!(flags.flags.is_empty());

    let schedule = svc
        .get_schedule_summary(GetScheduleSummaryRequest {
            year: 2023,
            schedule: ScheduleKindRequest::ScheduleC,
        })
        .expect("schedule c");
    assert!(schedule.total < 0.0);
}
