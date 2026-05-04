//! Integration test for the legal-z3 feature path.
//! This test is gated behind `--features legal-z3` and requires native libz3.
//! It validates that the Z3-native `violation_result` path produces correct
//! SatResult→Z3Result mapping rather than the pure-boolean fallback.

#[cfg(feature = "legal-z3")]
#[test]
fn z3_native_violation_satisfied() {
    use ledger_core::legal::{LegalSolver, TransactionFacts, Z3Result};

    let solver = LegalSolver::new();
    let rules = [ledger_core::legal::au_gst::rule_38_190()];
    let facts = TransactionFacts::new()
        .with_vendor("US")
        .with_supply_type("SaaS")
        .with_tax_code("BASEXCLUDED");

    let (confidence, issues) = solver.verify_all(&rules, &facts);
    assert!((confidence - 1.0).abs() < 0.001);
    assert!(issues.is_empty(), "expected no issues, got {issues:?}");
}

#[cfg(feature = "legal-z3")]
#[test]
fn z3_native_violation_violated() {
    use ledger_core::legal::{LegalSolver, TransactionFacts};

    let solver = LegalSolver::new();
    let rules = [ledger_core::legal::au_gst::rule_38_190()];
    let facts = TransactionFacts::new()
        .with_vendor("US")
        .with_supply_type("SaaS")
        .with_tax_code("INPUT");

    let (confidence, issues) = solver.verify_all(&rules, &facts);
    assert_eq!(confidence, 0.0);
    assert!(!issues.is_empty(), "expected issues for violation");
    assert_eq!(issues[0].code, "legal_violation");
}

#[cfg(feature = "legal-z3")]
#[test]
fn z3_native_disposition_via_to_issues() {
    use ledger_core::legal::{LegalSolver, TransactionFacts, Z3Result};

    let solver = LegalSolver::new();
    let rules = [ledger_core::legal::au_gst::rule_38_190()];

    // Violated → Unrecoverable
    let violated_facts = TransactionFacts::new()
        .with_vendor("US")
        .with_supply_type("SaaS")
        .with_tax_code("INPUT");
    let (_, issues) = solver.verify_all(&rules, &violated_facts);
    assert_eq!(issues.len(), 1);
    assert_eq!(
        issues[0].disposition,
        ledger_core::validation::Disposition::Unrecoverable
    );

    // Satisfied → no issues
    let satisfied_facts = TransactionFacts::new()
        .with_vendor("US")
        .with_supply_type("SaaS")
        .with_tax_code("BASEXCLUDED");
    let (_, issues) = solver.verify_all(&rules, &satisfied_facts);
    assert!(issues.is_empty());
}

// Without legal-z3 feature, these tests compile but only test the boolean fallback
#[cfg(not(feature = "legal-z3"))]
#[test]
fn z3_feature_disabled_fallback() {
    use ledger_core::legal::Z3Result;
    let violated = Z3Result::Violated {
        witness: "test".into(),
    };
    assert_eq!(
        violated.to_disposition(),
        ledger_core::validation::Disposition::Unrecoverable
    );
    assert_eq!(violated.to_confidence(), 0.0);
}
