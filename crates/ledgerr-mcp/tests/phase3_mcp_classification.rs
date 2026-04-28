mod common;

use ledger_core::ingest::TransactionInput;
use ledgerr_mcp::{
    ClassifyIngestedRequest, FlagStatusRequest, IngestPdfRequest, QueryFlagsRequest,
    RunRhaiRuleRequest, SampleTxRequest, TurboLedgerService, TurboLedgerTools,
};

fn service() -> TurboLedgerService {
    let workbook_path = common::unique_workbook_path("phase3-classification");
    TurboLedgerService::from_manifest_str(&common::manifest_for_workbook(&workbook_path, 2023))
        .expect("manifest")
}

fn write_rule_file(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let path = dir.path().join("classify.rhai");
    std::fs::write(
        &path,
        r#"
fn classify(tx) {
    let desc = tx["description"];
    let category = if desc.contains("Coffee") { "Meals" } else { "Uncategorized" };
    let confidence = if category == "Meals" { 0.92 } else { 0.33 };
    let review = confidence < 0.80;
    #{
      category: category,
      confidence: confidence,
      review: review,
      reason: "phase3-mcp"
    }
}
"#,
    )
    .expect("write rule");
    path
}

#[test]
fn mcp_07_run_rhai_rule_validates_candidate_rule_on_sample_tx() {
    let svc = service();
    let dir = tempfile::tempdir().expect("tempdir");
    let rule_file = write_rule_file(&dir);

    let response = svc
        .run_rhai_rule(RunRhaiRuleRequest {
            rule_file,
            sample_tx: SampleTxRequest {
                tx_id: "sample-1".to_string(),
                account_id: "WF-BH-CHK".to_string(),
                date: "2023-01-03".to_string(),
                amount: "-11.00".to_string(),
                description: "Coffee Cart".to_string(),
            },
        })
        .expect("rule test should succeed");

    assert_eq!(response.category, "Meals");
    assert!(response.confidence >= 0.80);
}

#[test]
fn mcp_03_query_flags_returns_review_queue_by_year_and_status() {
    let svc = service();
    let dir = tempfile::tempdir().expect("tempdir");
    let journal_path = dir.path().join("ledger.beancount");
    let workbook_path = dir.path().join("tax-ledger.xlsx");
    let source_ref = dir.path().join("ctx.rkyv");

    let ingest = svc
        .ingest_pdf(IngestPdfRequest {
            pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
            journal_path,
            workbook_path,
            ontology_path: None,
            raw_context_bytes: Some(b"ctx".to_vec()),
            extracted_rows: vec![TransactionInput {
                account_id: "WF-BH-CHK".to_string(),
                date: "2023-01-15".to_string(),
                amount: "-99.99".to_string(),
                description: "Unknown Merchant".to_string(),
                source_ref: source_ref.display().to_string(),
            }],
        })
        .expect("ingest");
    assert_eq!(ingest.inserted_count, 1);

    let rule_file = write_rule_file(&dir);
    let classify = svc
        .classify_ingested(ClassifyIngestedRequest {
            rule_file,
            review_threshold: 0.80,
        })
        .expect("classify");
    assert_eq!(classify.classifications.len(), 1);

    let open = svc
        .query_flags(QueryFlagsRequest {
            year: 2023,
            status: FlagStatusRequest::Open,
        })
        .expect("query open flags");

    assert_eq!(open.flags.len(), 1);
    assert_eq!(open.flags[0].status, FlagStatusRequest::Open);
}
