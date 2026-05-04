//! Lint tests: one per domain type, asserting VisualizationSpec invariants.

use ledger_core::constraints::{
    ConstraintEvaluation, InvoiceConstraintSolver, InvoiceVerification, VendorConstraintSet,
};
use ledger_core::iso::HasVisualization;
use ledger_core::legal::{Jurisdiction, LegalRule, LegalSolver, TransactionFacts, Z3Result};
use ledger_core::pipeline::{
    Classified, Committed, Ingested, KasuariSolver, NeedsReview, PipelineState, Reconciled,
    Validated,
};
use ledger_core::validation::{CommitGate, Issue, MetaFlag, StageResult};

// Helper macro to avoid repeating the four assertions.
macro_rules! lint_spec {
    ($ty:ty) => {{
        let spec = <$ty>::viz_spec();
        assert!(
            !spec.description.is_empty(),
            "description is empty for {}",
            stringify!($ty)
        );
        assert!(
            !spec.rhai_dsl.is_empty(),
            "rhai_dsl is empty for {}",
            stringify!($ty)
        );
        assert!(
            spec.z_layer.index() <= 5,
            "z_layer.index() > 5 for {}",
            stringify!($ty)
        );
        assert!(
            !spec.semantic_type.known_name().is_empty(),
            "known_name is empty for {}",
            stringify!($ty)
        );
    }};
}

#[test]
fn iso_lint_pipeline_ingested() {
    lint_spec!(PipelineState<Ingested>);
}

#[test]
fn iso_lint_pipeline_validated() {
    lint_spec!(PipelineState<Validated>);
}

#[test]
fn iso_lint_pipeline_classified() {
    lint_spec!(PipelineState<Classified>);
}

#[test]
fn iso_lint_pipeline_reconciled() {
    lint_spec!(PipelineState<Reconciled>);
}

#[test]
fn iso_lint_pipeline_committed() {
    lint_spec!(PipelineState<Committed>);
}

#[test]
fn iso_lint_pipeline_needs_review() {
    lint_spec!(PipelineState<NeedsReview>);
}

#[test]
fn iso_lint_constraint_evaluation() {
    lint_spec!(ConstraintEvaluation);
}

#[test]
fn iso_lint_vendor_constraint_set() {
    lint_spec!(VendorConstraintSet);
}

#[test]
fn iso_lint_invoice_constraint_solver() {
    lint_spec!(InvoiceConstraintSolver);
}

#[test]
fn iso_lint_invoice_verification() {
    lint_spec!(InvoiceVerification);
}

#[test]
fn iso_lint_z3_result() {
    lint_spec!(Z3Result);
}

#[test]
fn iso_lint_legal_rule() {
    lint_spec!(LegalRule);
}

#[test]
fn iso_lint_legal_solver() {
    lint_spec!(LegalSolver);
}

#[test]
fn iso_lint_jurisdiction() {
    lint_spec!(Jurisdiction);
}

#[test]
fn iso_lint_transaction_facts() {
    lint_spec!(TransactionFacts);
}

#[test]
fn iso_lint_commit_gate() {
    lint_spec!(CommitGate);
}

#[test]
fn iso_lint_issue() {
    lint_spec!(Issue);
}

#[test]
fn iso_lint_meta_flag() {
    lint_spec!(MetaFlag);
}

#[test]
fn iso_lint_stage_result() {
    // Use a concrete type parameter; the impl is generic over T: 'static.
    lint_spec!(StageResult<()>);
}

#[test]
fn iso_lint_kasuari_solver() {
    lint_spec!(KasuariSolver);
}
