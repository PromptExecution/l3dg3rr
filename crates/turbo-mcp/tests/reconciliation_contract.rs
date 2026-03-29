use turbo_mcp::{ReconciliationStageRequest, TurboLedgerService};

fn service() -> TurboLedgerService {
    TurboLedgerService::from_manifest_str(
        "[session]\nworkbook_path=\"tax-ledger.xlsx\"\nactive_year=2023\n",
    )
    .expect("manifest")
}

#[test]
fn recon_01_commit_is_blocked_when_postings_are_imbalanced() {
    let svc = service();
    let response = svc
        .commit_reconciliation_stage_tool(ReconciliationStageRequest {
            source_total: "100.00".to_string(),
            extracted_total: "100.00".to_string(),
            posting_amounts: vec!["-100.00".to_string(), "90.00".to_string()],
        })
        .expect("commit stage response");

    assert_eq!(response.stage, "commit");
    assert_eq!(response.status, "blocked");
    assert_eq!(response.blocked_reasons, vec!["imbalance_postings".to_string()]);
    assert_eq!(response.stage_marker, "validate:passed|reconcile:passed|commit:blocked");
    assert_eq!(response.diagnostics[0].key, "posting_balance_mismatch");
    assert_eq!(response.diagnostics[0].message, "posting amounts must net to 0.00");
}

#[test]
fn recon_02_reconcile_fails_with_deterministic_totals_mismatch_diagnostics() {
    let svc = service();
    let response = svc
        .reconcile_reconciliation_stage_tool(ReconciliationStageRequest {
            source_total: "100.00".to_string(),
            extracted_total: "99.00".to_string(),
            posting_amounts: vec!["-50.00".to_string(), "50.00".to_string()],
        })
        .expect("reconcile stage response");

    assert_eq!(response.stage, "reconcile");
    assert_eq!(response.status, "blocked");
    assert_eq!(response.blocked_reasons, vec!["totals_mismatch".to_string()]);
    assert_eq!(response.stage_marker, "validate:passed|reconcile:blocked");
    assert_eq!(response.diagnostics[0].key, "source_extracted_total_mismatch");
    assert_eq!(
        response.diagnostics[0].message,
        "source_total and extracted_total must match"
    );
}

#[test]
fn recon_01_02_commit_is_ready_only_after_validate_and_reconcile_succeed() {
    let svc = service();
    let request = ReconciliationStageRequest {
        source_total: "100.00".to_string(),
        extracted_total: "100.00".to_string(),
        posting_amounts: vec!["-100.00".to_string(), "100.00".to_string()],
    };

    let validate = svc
        .validate_reconciliation_stage_tool(request.clone())
        .expect("validate stage response");
    assert_eq!(validate.stage, "validate");
    assert_eq!(validate.status, "passed");
    assert!(validate.blocked_reasons.is_empty());
    assert_eq!(validate.stage_marker, "validate:passed");

    let reconcile = svc
        .reconcile_reconciliation_stage_tool(request.clone())
        .expect("reconcile stage response");
    assert_eq!(reconcile.stage, "reconcile");
    assert_eq!(reconcile.status, "passed");
    assert!(reconcile.blocked_reasons.is_empty());
    assert_eq!(reconcile.stage_marker, "validate:passed|reconcile:passed");

    let commit = svc
        .commit_reconciliation_stage_tool(request)
        .expect("commit stage response");
    assert_eq!(commit.stage, "commit");
    assert_eq!(commit.status, "ready");
    assert!(commit.blocked_reasons.is_empty());
    assert_eq!(
        commit.stage_marker,
        "validate:passed|reconcile:passed|commit:ready"
    );
}
