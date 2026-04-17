mod common;

use calamine::Reader;
use ledger_core::ingest::TransactionInput;
use ledger_core::workbook::REQUIRED_SHEETS;
use ledgerr_mcp::{
    ClassifyTransactionRequest, ExportCpaWorkbookRequest, GetScheduleSummaryRequest,
    IngestPdfRequest, ScheduleKindRequest, TurboLedgerService, TurboLedgerTools,
};

fn service() -> TurboLedgerService {
    let workbook_path = common::unique_workbook_path("phase5-cpa");
    TurboLedgerService::from_manifest_str(&format!(
        "{}\n[accounts.WF-BH-CHK]\ninstitution=\"Wise\"\ntype=\"checking\"\ncurrency=\"USD\"\n",
        common::manifest_for_workbook(&workbook_path, 2023)
    ))
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

fn cell_text<T>(range: &calamine::Range<T>, row: usize, col: usize) -> Option<String>
where
    T: calamine::CellType + ToString,
{
    range.get((row, col)).map(ToString::to_string)
}

#[test]
fn wb_01_02_03_export_cpa_workbook_honors_canonical_contract_and_materializes_contents() {
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
        tx_id: low_conf_tx.clone(),
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

    let mut wb = calamine::open_workbook_auto(workbook_path).expect("workbook");
    for required in REQUIRED_SHEETS {
        assert!(
            wb.sheet_names().iter().any(|sheet| sheet == required),
            "missing required sheet `{required}`"
        );
    }
    assert!(wb.sheet_names().iter().any(|s| s == "TX.WF-BH-CHK"));

    let meta = wb.worksheet_range("META.config").expect("META.config");
    assert_eq!(
        cell_text(&meta, 1, 0),
        Some(svc.workbook_path().display().to_string())
    );
    assert_eq!(cell_text(&meta, 1, 1), Some("2023".to_string()));

    let registry = wb.worksheet_range("ACCT.registry").expect("ACCT.registry");
    assert_eq!(cell_text(&registry, 1, 0), Some("WF-BH-CHK".to_string()));
    assert_eq!(cell_text(&registry, 1, 1), Some("Wise".to_string()));
    assert_eq!(cell_text(&registry, 1, 2), Some("checking".to_string()));
    assert_eq!(cell_text(&registry, 1, 3), Some("USD".to_string()));

    let tx_sheet = wb.worksheet_range("TX.WF-BH-CHK").expect("TX sheet");
    assert_eq!(cell_text(&tx_sheet, 1, 3), Some("Office Depot".to_string()));
    assert_eq!(
        cell_text(&tx_sheet, 1, 4),
        Some("OfficeSupplies".to_string())
    );

    let flags_open = wb.worksheet_range("FLAGS.open").expect("FLAGS.open");
    assert_eq!(cell_text(&flags_open, 1, 0), Some(low_conf_tx.clone()));

    let sched_c = wb.worksheet_range("SCHED.C").expect("SCHED.C");
    assert_eq!(
        cell_text(&sched_c, 1, 0),
        Some("OfficeSupplies".to_string())
    );

    let fbar = wb.worksheet_range("FBAR.accounts").expect("FBAR.accounts");
    assert_eq!(cell_text(&fbar, 1, 0), Some("WF-BH-CHK".to_string()));

    let audit = wb.worksheet_range("AUDIT.log").expect("AUDIT.log");
    assert_eq!(cell_text(&audit, 1, 1), Some("agent".to_string()));
    assert_eq!(cell_text(&audit, 1, 3), Some("category".to_string()));
    assert_eq!(cell_text(&audit, 1, 5), Some("OfficeSupplies".to_string()));
    assert!(
        audit.height() >= 4,
        "expected multiple audit log entries for two classifications, got {} rows",
        audit.height()
    );
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
