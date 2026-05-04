//! Type mesh verification tests for pipeline stage compatibility.
//!
//! These tests assert that all pipeline stage input/output types are
//! structurally compatible at compile time. If a type changes and breaks
//! the mesh, these tests fail to compile — catching drift before runtime.

use ledger_core::classify::{ClassificationOutcome, ClassifiedTransaction, SampleTransaction};
use ledger_core::ingest::{deterministic_tx_id, IngestedTransaction, TransactionInput};
use ledger_core::journal::JournalTransaction;
use ledger_core::validation::{and_then, Disposition, Issue, IssueSource, MetaCtx, StageResult};
use ledger_core::workbook::TxProjectionRow;

use static_assertions::assert_impl_all;

// ── Leaf type invariants ──────────────────────────────────────────────

// All pipeline I/O types must be Send + Sync for async dispatch.
assert_impl_all!(TransactionInput: Send, Sync);
assert_impl_all!(IngestedTransaction: Send, Sync);
assert_impl_all!(SampleTransaction: Send, Sync);
assert_impl_all!(ClassificationOutcome: Send, Sync);
assert_impl_all!(ClassifiedTransaction: Send, Sync);
assert_impl_all!(JournalTransaction: Send, Sync);
assert_impl_all!(TxProjectionRow: Send, Sync);
assert_impl_all!(Issue: Send, Sync);
assert_impl_all!(MetaCtx: Send, Sync);

// ── Shape compatibility: TransactionInput ↔ SampleTransaction ─────────

#[test]
fn test_transaction_input_to_sample_transaction_shape() {
    fn check_shape(input: TransactionInput) -> SampleTransaction {
        SampleTransaction {
            tx_id: deterministic_tx_id(&input),
            account_id: input.account_id,
            date: input.date,
            amount: input.amount,
            description: input.description,
        }
    }

    let input = TransactionInput {
        account_id: "chase--checking".into(),
        date: "2024-03-15".into(),
        amount: "-142.50".into(),
        description: "AMAZON.COM".into(),
        source_ref: "statement.pdf".into(),
    };

    let sample = check_shape(input);
    assert_eq!(sample.account_id, "chase--checking");
    assert_eq!(sample.description, "AMAZON.COM");
}

// ── Shape compatibility: TransactionInput → JournalTransaction ────────

#[test]
fn test_transaction_input_to_journal_shape() {
    fn check_shape(input: &TransactionInput) -> JournalTransaction {
        JournalTransaction::from_input(input)
    }

    let input = TransactionInput {
        account_id: "WF-BH-CHK".into(),
        date: "2024-01-10".into(),
        amount: "-50.00".into(),
        description: "Office Depot".into(),
        source_ref: "wf-statement.pdf".into(),
    };

    let journal = check_shape(&input);
    assert_eq!(journal.tx_id, deterministic_tx_id(&input));
    assert_eq!(journal.amount, "-50.00");
}

// ── Shape compatibility: ClassificationOutcome → ClassifiedTransaction ─

#[test]
fn test_classification_outcome_to_classified_shape() {
    fn check_shape(tx_id: String, outcome: ClassificationOutcome) -> ClassifiedTransaction {
        ClassifiedTransaction {
            tx_id,
            category: outcome.category,
            confidence: outcome.confidence,
            needs_review: outcome.needs_review,
            reason: outcome.reason,
        }
    }

    let outcome = ClassificationOutcome {
        category: "OfficeSupplies".into(),
        confidence: 0.92,
        needs_review: false,
        reason: "keyword:business_expense matched".into(),
    };

    let classified = check_shape("abc123".into(), outcome);
    assert_eq!(classified.category, "OfficeSupplies");
    assert_eq!(classified.confidence, 0.92);
    assert!(!classified.needs_review);
}

// ── Shape compatibility: ClassifiedTransaction → TxProjectionRow ──────

#[test]
fn test_classified_to_projection_row_requires_context() {
    // This test documents the known gap: ClassifiedTransaction does not
    // carry account_id, date, amount, or description.
    fn check_gap(classified: &ClassifiedTransaction) -> TxProjectionRow {
        TxProjectionRow {
            tx_id: classified.tx_id.clone(),
            account_id: String::new(),
            date: String::new(),
            amount: String::new(),
            description: classified.reason.clone(),
            source_ref: String::new(),
        }
    }

    let classified = ClassifiedTransaction {
        tx_id: "abc123".into(),
        category: "SelfEmployment".into(),
        confidence: 0.85,
        needs_review: false,
        reason: "income > $5000 threshold".into(),
    };

    let row = check_gap(&classified);
    assert_eq!(row.tx_id, "abc123");
    assert_eq!(row.account_id, "");
    assert_eq!(row.description, "income > $5000 threshold");
}

// ── Validation pipeline: Issue → MetaCtx → StageResult<T> ─────────────

#[test]
fn test_validation_pipeline_mesh() {
    let issue = Issue::recoverable(
        "V-001",
        "low confidence classification",
        IssueSource::RhaiRule {
            rule_id: "classify_schedule_c".into(),
        },
    );
    let issues = vec![issue];

    let initial = MetaCtx::initial();
    let advanced = initial.advance("classify", 0.72, &issues);

    // First stage: accumulated_confidence == 0.0, so it uses stage_confidence directly
    assert_eq!(advanced.accumulated_confidence, 0.72);
    assert_eq!(advanced.stage_trace.len(), 1);
    assert_eq!(advanced.stage_trace[0].stage, "classify");
    assert_eq!(advanced.stage_trace[0].confidence, 0.72);
    assert_eq!(advanced.stage_trace[0].issue_count, 1);

    // StageResult constructors use MetaCtx::default() (confidence 0.0).
    // The and_then() function is the correct way to chain stages.
    let ingest_result = StageResult::ok("ingested_data".to_string(), 1.0);
    let classify_result = and_then(ingest_result, "classify", |_ctx| {
        StageResult::with_issues("classified_data".to_string(), 0.72, issues.clone())
    });

    // The and_then function advances meta with the next stage's confidence
    assert_eq!(classify_result.meta.accumulated_confidence, 0.72);
    assert_eq!(classify_result.confidence, 0.72);
    assert_eq!(classify_result.issues.len(), 1);
}

// ── Disposition invariants ────────────────────────────────────────────

#[test]
fn test_disposition_enum_coverage() {
    let unrecoverable = Issue::unrecoverable("E-001", "data corruption detected");
    let recoverable = Issue::recoverable(
        "W-001",
        "low confidence classification",
        IssueSource::Constraint { strength: 0.5 },
    );
    let advisory = Issue::advisory("A-001", "consider reviewing category");

    assert_eq!(unrecoverable.disposition, Disposition::Unrecoverable);
    assert_eq!(recoverable.disposition, Disposition::Recoverable);
    assert_eq!(advisory.disposition, Disposition::Advisory);
}

// ── Ingest idempotency type contract ──────────────────────────────────

#[test]
fn test_ingest_idempotency_contract() {
    let input = TransactionInput {
        account_id: "chase--checking".into(),
        date: "2024-03-15".into(),
        amount: "-142.50".into(),
        description: "AMAZON.COM".into(),
        source_ref: "statement.pdf".into(),
    };

    let id1 = deterministic_tx_id(&input);
    let id2 = deterministic_tx_id(&input);

    assert_eq!(id1, id2);
    assert_eq!(id1.len(), 64); // Blake3 hex length
}

// ── MetaCtx confidence accumulation ───────────────────────────────────

#[test]
fn test_meta_ctx_confidence_compounds_multiplicatively() {
    let ctx = MetaCtx::initial();

    // Stage 1: ingest (deterministic, confidence 1.0)
    // accumulated_confidence == 0.0, so sets to 1.0
    let ctx = ctx.advance("ingest", 1.0, &[]);
    assert_eq!(ctx.accumulated_confidence, 1.0);

    // Stage 2: validate (high confidence, 0.95)
    // 1.0 * 0.95 = 0.95
    let ctx = ctx.advance("validate", 0.95, &[]);
    assert!((ctx.accumulated_confidence - 0.95).abs() < 0.001);

    // Stage 3: classify (medium confidence, 0.72)
    // 0.95 * 0.72 = 0.684
    let ctx = ctx.advance("classify", 0.72, &[]);
    assert!((ctx.accumulated_confidence - 0.684).abs() < 0.001);

    // Stage 4: reconcile (low confidence, 0.50)
    // 0.684 * 0.50 = 0.342
    let ctx = ctx.advance("reconcile", 0.50, &[]);
    assert!((ctx.accumulated_confidence - 0.342).abs() < 0.001);
}

// ── Full mesh: ingest → classify → validation ─────────────────────────

#[test]
fn test_full_pipeline_type_mesh() {
    let input = TransactionInput {
        account_id: "chase--checking".into(),
        date: "2024-03-15".into(),
        amount: "-142.50".into(),
        description: "AMAZON.COM".into(),
        source_ref: "statement.pdf".into(),
    };

    let tx_id = deterministic_tx_id(&input);

    let journal = JournalTransaction::from_input(&input);
    assert_eq!(journal.tx_id, tx_id);

    let sample = SampleTransaction {
        tx_id: tx_id.clone(),
        account_id: input.account_id.clone(),
        date: input.date.clone(),
        amount: input.amount.clone(),
        description: input.description.clone(),
    };

    let outcome = ClassificationOutcome {
        category: "OfficeSupplies".into(),
        confidence: 0.85,
        needs_review: false,
        reason: "keyword:business_expense matched".into(),
    };

    let classified = ClassifiedTransaction {
        tx_id: sample.tx_id,
        category: outcome.category,
        confidence: outcome.confidence,
        needs_review: outcome.needs_review,
        reason: outcome.reason,
    };

    let ingest_result = StageResult::ok(classified.clone(), 1.0);
    let classify_result = and_then(ingest_result, "classify", |_ctx| {
        StageResult::ok(classified, 0.85)
    });

    assert_eq!(classify_result.data.category, "OfficeSupplies");
    assert_eq!(classify_result.data.confidence, 0.85);
    assert!((classify_result.confidence - 0.85).abs() < 0.001);
    assert_eq!(classify_result.meta.accumulated_confidence, 0.85);
}
