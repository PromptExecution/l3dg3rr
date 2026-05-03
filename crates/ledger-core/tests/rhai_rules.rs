/// Integration tests for `.rhai` rule files in `rules/` at the project root.
///
/// Each test invokes `ClassificationEngine::run_rule_from_file` — the same
/// path the production pipeline uses — so these tests exercise the real Rhai
/// engine, real file I/O, and real map extraction without mocking.
///
/// Run with: `cargo test -p ledger-core rhai_rules`
///
/// # Path convention
/// Tests resolve rule files relative to the Cargo workspace root.
/// `CARGO_MANIFEST_DIR` for an integration test points to the crate root
/// (`crates/ledger-core`), so we navigate two levels up to reach the
/// project root where `rules/` lives.
use std::path::PathBuf;

use ledger_core::classify::{ClassificationEngine, SampleTransaction};

/// Resolve a path under `<workspace-root>/rules/`.
fn rule_path(filename: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR = .../crates/ledger-core
    // two parents up = workspace root
    let manifest =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by cargo test");
    PathBuf::from(manifest)
        .parent() // crates/
        .expect("crates parent")
        .parent() // workspace root
        .expect("workspace root")
        .join("rules")
        .join(filename)
}

fn engine() -> ClassificationEngine {
    ClassificationEngine::default()
}

// ---------------------------------------------------------------------------
// classify_foreign_income.rhai
// ---------------------------------------------------------------------------

#[test]
fn rhai_01_foreign_income_happy_path() {
    // A transaction from an HSBC account with a German employer description
    // should classify as ForeignIncome with confidence 0.90 and no review
    // flag (amount is below the 10 000 threshold).
    let sample = SampleTransaction {
        tx_id: "test-foreign-01".into(),
        account_id: "HSBC--BH-CHK--2024-03".into(),
        date: "2024-03-15".into(),
        amount: "4250.00".into(),
        description: "Wire transfer from DE employer — salary".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_foreign_income.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(
        outcome.category, "ForeignIncome",
        "expected ForeignIncome, got: {}",
        outcome.category
    );
    assert!(
        (outcome.confidence - 0.90).abs() < f64::EPSILON,
        "expected confidence 0.90, got: {}",
        outcome.confidence
    );
    assert!(
        !outcome.needs_review,
        "amount 4250 should not trigger review (threshold 10000)"
    );
}

#[test]
fn rhai_02_foreign_income_zero_amount_no_review() {
    // Zero-amount transactions (e.g. bank fee reversals) must still classify
    // correctly and must NOT trigger the high-value review flag.
    let sample = SampleTransaction {
        tx_id: "test-foreign-02".into(),
        account_id: "HSBC--BH-CHK--2024-04".into(),
        date: "2024-04-01".into(),
        amount: "0.00".into(),
        description: "Bank fee reversal — HSBC DE".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_foreign_income.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(
        !outcome.needs_review,
        "zero amount must not trigger review flag"
    );
}

#[test]
fn rhai_03_foreign_income_high_value_triggers_review() {
    // A large foreign transfer (> 10 000) must set review: true.
    // The Rust ReviewFlag is emitted by classify_rows_from_file; here we
    // verify that the rule itself sets the review field.
    let sample = SampleTransaction {
        tx_id: "test-foreign-03".into(),
        account_id: "HSBC--BH-CHK--2024-06".into(),
        date: "2024-06-30".into(),
        amount: "12000.00".into(),
        description: "Annual bonus — DE entity".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_foreign_income.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(
        outcome.needs_review,
        "amount > 10000 must set review: true (FBAR-adjacent threshold)"
    );
}

#[test]
fn rhai_04_foreign_income_negative_high_value_triggers_review() {
    // Negative amounts (debits) above 10 000 in absolute value should also
    // trigger review. The rule uses abs() internally.
    let sample = SampleTransaction {
        tx_id: "test-foreign-04".into(),
        account_id: "HSBC--BH-CHK--2024-07".into(),
        date: "2024-07-10".into(),
        amount: "-15000.00".into(),
        description: "Payment to DE contractor".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_foreign_income.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "ForeignIncome");
    assert!(
        outcome.needs_review,
        "negative amount with abs > 10000 must trigger review"
    );
}

#[test]
fn rhai_05_foreign_income_no_signal_returns_unclassified() {
    // A domestic transaction with no foreign signals must not be classified
    // as ForeignIncome — this rule should return Unclassified so the engine
    // can continue to the next rule in a chain.
    let sample = SampleTransaction {
        tx_id: "test-foreign-05".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-10".into(),
        amount: "500.00".into(),
        description: "Grocery store purchase".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_foreign_income.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(
        outcome.category, "Unclassified",
        "domestic transaction must not match foreign income rule"
    );
    assert!(
        (outcome.confidence - 0.0).abs() < f64::EPSILON,
        "no-signal outcome must have confidence 0.0"
    );
}

// ---------------------------------------------------------------------------
// classify_self_employment.rhai
// ---------------------------------------------------------------------------

#[test]
fn rhai_06_self_employment_strong_keyword() {
    // "invoice" in description → strong match → confidence 0.85, no review.
    let sample = SampleTransaction {
        tx_id: "test-se-01".into(),
        account_id: "WF--BH-CHK--2024-02".into(),
        date: "2024-02-05".into(),
        amount: "3500.00".into(),
        description: "Client invoice #INV-042 — consulting services".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_self_employment.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "SelfEmployment");
    assert!(
        (outcome.confidence - 0.85).abs() < f64::EPSILON,
        "strong keyword should yield confidence 0.85"
    );
    assert!(
        !outcome.needs_review,
        "strong match should not require review"
    );
}

#[test]
fn rhai_07_self_employment_weak_keyword_triggers_review() {
    // "freelance" matches the weak tier → confidence 0.65 → review: true.
    // This tests the ambiguous-description edge case: the rule fires but
    // signals uncertainty for CPA inspection.
    let sample = SampleTransaction {
        tx_id: "test-se-02".into(),
        account_id: "WF--BH-CHK--2024-03".into(),
        date: "2024-03-20".into(),
        amount: "800.00".into(),
        description: "Freelance design work".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_self_employment.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "SelfEmployment");
    assert!(
        (outcome.confidence - 0.65).abs() < f64::EPSILON,
        "weak keyword should yield confidence 0.65"
    );
    assert!(
        outcome.needs_review,
        "confidence < 0.70 must trigger review flag"
    );
}

#[test]
fn rhai_08_self_employment_no_keyword_unclassified() {
    // Transaction with no SE keywords must return Unclassified.
    let sample = SampleTransaction {
        tx_id: "test-se-03".into(),
        account_id: "WF--BH-CHK--2024-04".into(),
        date: "2024-04-15".into(),
        amount: "120.00".into(),
        description: "Netflix subscription".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_self_employment.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
}

// ---------------------------------------------------------------------------
// classify_fallback.rhai
// ---------------------------------------------------------------------------

#[test]
fn rhai_09_fallback_always_unclassified_review_true() {
    // The fallback rule must classify ANY transaction as Unclassified with
    // review: true — no exceptions. Confidence must be exactly 0.0.
    let sample = SampleTransaction {
        tx_id: "test-fallback-01".into(),
        account_id: "WF--BH-CHK--2024-01".into(),
        date: "2024-01-20".into(),
        amount: "9999.99".into(),
        description: "Mystery payment".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fallback.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
    assert!(
        (outcome.confidence - 0.0).abs() < f64::EPSILON,
        "fallback confidence must be 0.0"
    );
    assert!(
        outcome.needs_review,
        "fallback must unconditionally set review: true"
    );
}

#[test]
fn rhai_10_fallback_zero_amount_still_review() {
    // Zero-amount transactions that reach fallback must still be flagged.
    let sample = SampleTransaction {
        tx_id: "test-fallback-02".into(),
        account_id: "WF--BH-CHK--2024-05".into(),
        date: "2024-05-01".into(),
        amount: "0.00".into(),
        description: "Pending hold".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fallback.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "Unclassified");
    assert!(
        outcome.needs_review,
        "zero-amount fallback must require review"
    );
}

#[test]
fn rhai_11_fallback_reason_contains_description() {
    // The fallback rule should embed the original description in its reason
    // string so auditors can see what the unclassified transaction said.
    let sample = SampleTransaction {
        tx_id: "test-fallback-03".into(),
        account_id: "WF--BH-CHK--2024-05".into(),
        date: "2024-05-10".into(),
        amount: "42.00".into(),
        description: "Acme Corp refund".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_fallback.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert!(
        outcome.reason.contains("Acme Corp refund"),
        "fallback reason should include original description for audit trail; got: {}",
        outcome.reason
    );
}
