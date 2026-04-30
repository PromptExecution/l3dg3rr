use std::path::{Path, PathBuf};

use ledger_core::classify::{ClassificationEngine, SampleTransaction};
use ledger_core::rule_registry::{RuleRegistry, SemanticRuleSelector};

fn rules_dir() -> PathBuf {
    let manifest =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo test");
    Path::new(&manifest)
        .parent()
        .expect("crates parent")
        .parent()
        .expect("workspace root")
        .join("rules")
}

fn sample(description: &str, account_id: &str, amount: &str) -> SampleTransaction {
    SampleTransaction {
        tx_id: format!("tx-{description}"),
        account_id: account_id.to_string(),
        date: "2024-03-15".to_string(),
        amount: amount.to_string(),
        description: description.to_string(),
    }
}

fn file_names(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .expect("rule path has utf8 filename")
                .to_string()
        })
        .collect()
}

#[test]
fn load_from_dir_loads_transaction_rules_only() {
    let registry = RuleRegistry::load_from_dir(&rules_dir()).expect("rules directory loads");
    let names = file_names(registry.rule_paths());

    assert!(registry.rule_count() >= 3);
    assert_eq!(registry.candidate_count(), 0);
    assert!(names.contains(&"classify_foreign_income.rhai".to_string()));
    assert!(names.contains(&"classify_self_employment.rhai".to_string()));
    assert!(names.contains(&"classify_fallback.rhai".to_string()));
    assert!(
        !names.contains(&"classify_document_shape.rhai".to_string()),
        "document-shape rules use classify_document(), not transaction classify()"
    );
}

#[test]
fn select_rules_deterministic_prefers_keyword_matches_and_appends_fallback() {
    let registry = RuleRegistry::load_from_dir(&rules_dir()).expect("rules directory loads");
    let tx = sample(
        "Wire transfer from DE employer salary",
        "HSBC--BH-CHK--2024-03",
        "4250.00",
    );

    let selected = registry.select_rules_deterministic(&tx);
    let names = file_names(&selected);

    assert_eq!(
        names.first().map(String::as_str),
        Some("classify_foreign_income.rhai")
    );
    assert_eq!(
        names.last().map(String::as_str),
        Some("classify_fallback.rhai")
    );
}

#[test]
fn classify_waterfall_returns_first_non_unclassified_result() {
    let registry = RuleRegistry::load_from_dir(&rules_dir()).expect("rules directory loads");
    let mut engine = ClassificationEngine::default();
    let tx = sample(
        "Client invoice #INV-042 consulting services",
        "WF--BH-CHK--2024-02",
        "3500.00",
    );

    let outcome = registry
        .classify_waterfall(&mut engine, &tx)
        .expect("waterfall classifies");

    assert_eq!(outcome.category, "SelfEmployment");
    assert!(outcome.confidence > 0.0);
    assert_ne!(outcome.reason, "no rule produced a classification");
}

#[test]
fn classify_waterfall_preserves_fallback_unclassified_reason() {
    let registry = RuleRegistry::load_from_dir(&rules_dir()).expect("rules directory loads");
    let mut engine = ClassificationEngine::default();
    let tx = sample("Mystery payment", "WF--BH-CHK--2024-02", "99.00");

    let outcome = registry
        .classify_waterfall(&mut engine, &tx)
        .expect("waterfall reaches fallback");

    assert_eq!(outcome.category, "Unclassified");
    assert_eq!(outcome.confidence, 0.0);
    assert!(outcome.needs_review);
    assert!(outcome.reason.contains("Mystery payment"));
}

#[test]
fn semantic_candidate_ids_are_stable_after_rebuild() {
    let mut first = RuleRegistry::load_from_dir(&rules_dir()).expect("rules directory loads");
    first
        .build_embedding_index()
        .expect("local lexical index builds");
    let mut second = RuleRegistry::load_from_dir(&rules_dir()).expect("rules directory loads");
    second
        .build_embedding_index()
        .expect("local lexical index builds");

    assert_eq!(
        first.semantic_candidate_ids(),
        second.semantic_candidate_ids()
    );
    assert_eq!(first.semantic_candidate_ids().len(), first.rule_count());
}

#[test]
fn semantic_selector_preserves_deterministic_fallback() {
    let mut registry = RuleRegistry::load_from_dir(&rules_dir()).expect("rules directory loads");
    registry
        .build_embedding_index()
        .expect("local lexical index builds");
    let tx = sample("Mystery payment", "WF--BH-CHK--2024-02", "99.00");

    let semantic = registry.select_rules_semantic(&tx, 5);
    let deterministic = registry.select_rules_deterministic(&tx);

    assert_eq!(file_names(&semantic), file_names(&deterministic));
}

#[test]
fn rule_registry_waterfall_remains_authoritative_with_semantic_index() {
    let mut registry = RuleRegistry::load_from_dir(&rules_dir()).expect("rules directory loads");
    registry
        .build_embedding_index()
        .expect("local lexical index builds");
    let mut engine = ClassificationEngine::default();
    let tx = sample(
        "Client invoice #INV-042 consulting services",
        "WF--BH-CHK--2024-02",
        "3500.00",
    );

    let selected = registry.select_rules_semantic(&tx, 3);
    let outcome = registry
        .classify_waterfall(&mut engine, &tx)
        .expect("waterfall classifies");

    assert!(!selected.is_empty());
    assert_eq!(outcome.category, "SelfEmployment");
}
