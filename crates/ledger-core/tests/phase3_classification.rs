use ledger_core::classify::{ClassificationEngine, FlagStatus, SampleTransaction};
use ledger_core::ingest::{deterministic_tx_id, TransactionInput};

fn write_rule_file(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let path = dir.path().join("classify.rhai");
    std::fs::write(
        &path,
        r#"
fn classify(tx) {
    let desc = tx["description"];
    let category = if desc.contains("Coffee") { "Meals" } else { "Uncategorized" };
    let confidence = if category == "Meals" { 0.91 } else { 0.45 };
    let review = confidence < 0.80;
    #{
      category: category,
      confidence: confidence,
      review: review,
      reason: "phase3-test"
    }
}
"#,
    )
    .expect("rule file write");
    path
}

#[test]
fn clsf_01_02_03_runtime_rules_classify_and_emit_review_flags() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = write_rule_file(&dir);

    let tx_ok = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-15".to_string(),
        amount: "-42.11".to_string(),
        description: "Coffee Shop".to_string(),
        source_ref: "ctx1.rkyv".to_string(),
    };
    let tx_review = TransactionInput {
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-01-16".to_string(),
        amount: "-99.99".to_string(),
        description: "Unknown Merchant".to_string(),
        source_ref: "ctx2.rkyv".to_string(),
    };

    let mut engine = ClassificationEngine::default();
    let result = engine
        .classify_rows_from_file(&rules, &[tx_ok.clone(), tx_review.clone()], 0.80)
        .expect("classification should succeed");

    assert_eq!(result.classifications.len(), 2);
    let meals = result
        .classifications
        .iter()
        .find(|c| c.tx_id == deterministic_tx_id(&tx_ok))
        .expect("meals classification");
    assert_eq!(meals.category, "Meals");
    assert!(meals.confidence >= 0.80);

    let uncategorized = result
        .classifications
        .iter()
        .find(|c| c.tx_id == deterministic_tx_id(&tx_review))
        .expect("uncategorized classification");
    assert_eq!(uncategorized.category, "Uncategorized");
    assert!(uncategorized.needs_review);

    let open_flags_2023 = engine.query_flags(2023, FlagStatus::Open);
    assert_eq!(open_flags_2023.len(), 1);
    assert_eq!(open_flags_2023[0].tx_id, deterministic_tx_id(&tx_review));
}

#[test]
fn clsf_04_candidate_rule_test_runs_without_persisting_flags() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = write_rule_file(&dir);

    let sample = SampleTransaction {
        tx_id: "sample-tx".to_string(),
        account_id: "WF-BH-CHK".to_string(),
        date: "2023-02-01".to_string(),
        amount: "-3.50".to_string(),
        description: "Coffee Beans".to_string(),
    };

    let engine = ClassificationEngine::default();
    let outcome = engine
        .run_rule_from_file(&rules, &sample)
        .expect("rule run should succeed");

    assert_eq!(outcome.category, "Meals");
    assert!(outcome.confidence >= 0.80);
    assert!(engine.query_flags(2023, FlagStatus::Open).is_empty());
}
