use calamine::Reader;
use ledger_core::ingest::TransactionInput;
use turbo_mcp::{
    ClassifyTransactionRequest, ExportCpaWorkbookRequest, GetScheduleSummaryRequest, IngestPdfRequest,
    ScheduleKindRequest, TurboLedgerService, TurboLedgerTools,
};

fn service() -> TurboLedgerService {
    TurboLedgerService::from_manifest_str(
        "[session]\nworkbook_path=\"tax-ledger.xlsx\"\nactive_year=2023\n",
    )
    .expect("manifest")
}

fn ingest(svc: &TurboLedgerService, description: &str, amount: &str, date: &str) -> String {
    let dir = tempfile::tempdir().expect("tempdir");
    let source_ref = dir.path().join(format!("{description}.rkyv"));
    let ingest = svc
        .ingest_pdf(IngestPdfRequest {
            pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
            journal_path: dir.path().join("ledger.beancount"),
            workbook_path: dir.path().join("tax-ledger.xlsx"),
            raw_context_bytes: Some(b"ctx".to_vec()),
            extracted_rows: vec![TransactionInput {
                account_id: "WF-BH-CHK".to_string(),
                date: date.to_string(),
                amount: amount.to_string(),
                description: description.to_string(),
                source_ref: source_ref.display().to_string(),
            }],
        })
        .expect("ingest");
    ingest.tx_ids[0].clone()
}

#[test]
fn wb_01_02_03_export_cpa_workbook_materializes_tx_and_flag_sheets() {
    let svc = service();
    let tx_id = ingest(&svc, "Office Depot", "-120.00", "2023-04-10");
    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id,
        category: "OfficeSupplies".to_string(),
        confidence: "0.90".to_string(),
        note: None,
        actor: "agent".to_string(),
    })
    .expect("classify");

    let low_conf_tx = ingest(&svc, "Unknown Vendor", "-44.00", "2023-04-11");
    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id: low_conf_tx,
        category: "Uncategorized".to_string(),
        confidence: "0.40".to_string(),
        note: None,
        actor: "agent".to_string(),
    })
    .expect("classify");

    let dir = tempfile::tempdir().expect("tempdir");
    let workbook_path = dir.path().join("cpa.xlsx");
    svc.export_cpa_workbook(ExportCpaWorkbookRequest {
        workbook_path: workbook_path.clone(),
    })
    .expect("export workbook");

    let wb = calamine::open_workbook_auto(workbook_path).expect("workbook");
    assert!(wb.sheet_names().iter().any(|s| s == "TX.WF-BH-CHK"));
    assert!(wb.sheet_names().iter().any(|s| s == "CAT.taxonomy"));
    assert!(wb.sheet_names().iter().any(|s| s == "FLAGS.open"));
    assert!(wb.sheet_names().iter().any(|s| s == "FLAGS.resolved"));
}

#[test]
fn tax_01_02_03_04_and_mcp_04_schedule_summary_are_available_by_year() {
    let svc = service();
    let c_tx = ingest(&svc, "Office Depot", "-120.00", "2023-04-10");
    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id: c_tx,
        category: "OfficeSupplies".to_string(),
        confidence: "0.90".to_string(),
        note: None,
        actor: "agent".to_string(),
    })
    .expect("classify C");

    let d_tx = ingest(&svc, "Coinbase Trade", "-250.00", "2023-06-01");
    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id: d_tx,
        category: "Crypto".to_string(),
        confidence: "0.92".to_string(),
        note: None,
        actor: "agent".to_string(),
    })
    .expect("classify D");

    let e_tx = ingest(&svc, "Rental Repair", "-80.00", "2023-03-07");
    svc.classify_transaction(ClassifyTransactionRequest {
        tx_id: e_tx,
        category: "RentRepair".to_string(),
        confidence: "0.93".to_string(),
        note: None,
        actor: "agent".to_string(),
    })
    .expect("classify E");

    let sched_c = svc
        .get_schedule_summary(GetScheduleSummaryRequest {
            year: 2023,
            schedule: ScheduleKindRequest::ScheduleC,
        })
        .expect("schedule C");
    assert!(sched_c.total < 0.0);

    let sched_d = svc
        .get_schedule_summary(GetScheduleSummaryRequest {
            year: 2023,
            schedule: ScheduleKindRequest::ScheduleD,
        })
        .expect("schedule D");
    assert!(sched_d.total < 0.0);

    let sched_e = svc
        .get_schedule_summary(GetScheduleSummaryRequest {
            year: 2023,
            schedule: ScheduleKindRequest::ScheduleE,
        })
        .expect("schedule E");
    assert!(sched_e.total < 0.0);

    let fbar = svc
        .get_schedule_summary(GetScheduleSummaryRequest {
            year: 2023,
            schedule: ScheduleKindRequest::Fbar,
        })
        .expect("fbar");
    assert!(!fbar.lines.is_empty());
}

