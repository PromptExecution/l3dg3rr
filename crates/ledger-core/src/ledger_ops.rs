//! Composable ledger operation interface.
//!
//! [`LedgerOperation`] is the stable trait; concrete implementations below are
//! either stubs (returning `NotImplemented`) or functional. The skeleton bodies
//! document the intended logic so a reader can see what each operation would do.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// `BusinessCalendar` is defined in `calendar`, which imports from here — use a
// forward-reference via the module path; actual Arc usage is behind `Option`.
use crate::calendar::BusinessCalendar;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum LedgerOpError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("operation not implemented: {0}")]
    NotImplemented(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("classification failed: {0}")]
    Classification(String),
}

// ---------------------------------------------------------------------------
// Operation kind — carried in scheduled events
// ---------------------------------------------------------------------------

/// Discriminated union of operation kinds, used in [`crate::calendar::ScheduledEvent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationKind {
    IngestStatement { source_glob: String },
    ClassifyTransactions { rule_dir: String },
    ReconcileAccount { account_id: String },
    ExportWorkbook { output_path: String },
    GenerateAuditTrail { year: i32 },
    CheckTaxDeadline { deadline_id: String },
}

// ---------------------------------------------------------------------------
// Execution context
// ---------------------------------------------------------------------------

/// Execution context passed to every operation.
#[derive(Clone)]
pub struct OperationContext {
    pub working_dir: PathBuf,
    pub rules_dir: PathBuf,
    pub calendar: Option<Arc<BusinessCalendar>>,
    pub dry_run: bool,
}

impl OperationContext {
    pub fn new(working_dir: PathBuf, rules_dir: PathBuf) -> Self {
        Self {
            working_dir,
            rules_dir,
            calendar: None,
            dry_run: false,
        }
    }

    pub fn with_calendar(mut self, cal: Arc<BusinessCalendar>) -> Self {
        self.calendar = Some(cal);
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }
}

// ---------------------------------------------------------------------------
// Operation result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub operation_id: String,
    pub success: bool,
    pub items_processed: usize,
    pub items_flagged: usize,
    pub issues: Vec<String>,
    pub duration_ms: u64,
}

impl OperationResult {
    pub fn success(id: impl Into<String>, items: usize) -> Self {
        Self {
            operation_id: id.into(),
            success: true,
            items_processed: items,
            items_flagged: 0,
            issues: Vec::new(),
            duration_ms: 0,
        }
    }

    pub fn failure(id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            operation_id: id.into(),
            success: false,
            items_processed: 0,
            items_flagged: 0,
            issues: vec![reason.into()],
            duration_ms: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Core trait
// ---------------------------------------------------------------------------

/// Core trait for all ledger operations.
pub trait LedgerOperation: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    /// Whether running the same operation twice is safe (e.g. ingest is idempotent via Blake3).
    fn is_idempotent(&self) -> bool {
        false
    }
    fn execute(&self, ctx: &OperationContext) -> Result<OperationResult, LedgerOpError>;
}

// ---------------------------------------------------------------------------
// Concrete operations
// ---------------------------------------------------------------------------

/// Ingest all statement files matching a glob pattern.
pub struct IngestStatementOp {
    pub source_glob: String,
    pub vendor_hint: Option<String>,
}

impl LedgerOperation for IngestStatementOp {
    fn id(&self) -> &str {
        "ingest-statement"
    }

    fn description(&self) -> &str {
        "Ingest statement files matching a glob pattern into the ledger"
    }

    fn is_idempotent(&self) -> bool {
        // Blake3 content-hash IDs prevent duplicate records on re-ingest
        true
    }

    fn execute(&self, _ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        // Intended logic:
        //   1. Expand `self.source_glob` against `ctx.working_dir`
        //   2. For each matched file:
        //      a. Derive DocType from extension
        //      b. Run `classify_document_shape` to identify vendor/format
        //      c. Call Docling sidecar (or stub) to extract text/tables
        //      d. Parse transactions from extracted content
        //      e. Compute Blake3 content-hash ID per transaction
        //      f. Upsert into the ledger store (skipping existing IDs)
        //      g. Write .rkyv sidecar snapshot alongside the source file
        //   3. Return item counts and any extraction issues
        Err(LedgerOpError::NotImplemented(
            "IngestStatementOp: PDF/CSV extraction pipeline not yet wired".to_string(),
        ))
    }
}

/// Run the Rhai classification waterfall over unclassified transactions.
pub struct ClassifyTransactionsOp {
    pub rule_dir: PathBuf,
    pub review_threshold: f64,
    pub account_filter: Option<String>,
}

impl LedgerOperation for ClassifyTransactionsOp {
    fn id(&self) -> &str {
        "classify-transactions"
    }

    fn description(&self) -> &str {
        "Run the Rhai rule waterfall over unclassified transactions"
    }

    fn execute(&self, _ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        // Intended logic:
        //   1. Load all `.rhai` rule files from `self.rule_dir` via RuleRegistry
        //   2. Fetch unclassified transactions from ledger store
        //      (optionally filtered by `self.account_filter`)
        //   3. For each transaction:
        //      a. Run ClassificationEngine.classify(transaction)
        //      b. If confidence >= review_threshold → write classification
        //      c. If confidence < threshold → flag for human review
        //   4. Persist updated classifications back to the store
        //   5. Return processed/flagged counts and any rule errors
        Err(LedgerOpError::NotImplemented(
            "ClassifyTransactionsOp: Rhai engine integration not yet wired".to_string(),
        ))
    }
}

/// Reconcile a single account against external source (Xero stub).
pub struct ReconcileAccountOp {
    pub account_id: String,
    pub dry_run: bool,
}

impl LedgerOperation for ReconcileAccountOp {
    fn id(&self) -> &str {
        "reconcile-account"
    }

    fn description(&self) -> &str {
        "Reconcile a single account against Xero or another external source"
    }

    fn execute(&self, ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        // Intended logic:
        //   1. Load local transactions for `self.account_id` from ledger store
        //   2. Fetch corresponding transactions from Xero API (ledgerr-xero crate)
        //   3. Match transactions by date/amount/description heuristics
        //   4. Flag unmatched items on either side
        //   5. If !self.dry_run && !ctx.dry_run → write reconciliation status
        //   6. Return matched/unmatched counts and issues
        let _ = ctx; // suppress unused warning while stubbed
        Err(LedgerOpError::NotImplemented(format!(
            "ReconcileAccountOp: Xero integration not yet wired (account={})",
            self.account_id
        )))
    }
}

/// Write the current ledger state to an Excel workbook.
pub struct ExportWorkbookOp {
    pub output_path: PathBuf,
    pub include_flags: bool,
}

impl LedgerOperation for ExportWorkbookOp {
    fn id(&self) -> &str {
        "export-workbook"
    }

    fn description(&self) -> &str {
        "Write the current ledger state to an Excel workbook"
    }

    fn execute(&self, _ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        // Intended logic:
        //   1. Open `rust_xlsxwriter::Workbook` at `self.output_path`
        //   2. Write Transactions sheet: all ledger transactions with classifications
        //   3. Write Schedule C / Schedule B sheets: pivoted tax-category summaries
        //   4. If `self.include_flags` → write Flags sheet with review items
        //   5. Add data validation drop-downs from TaxCategory/Flag enums (strum)
        //   6. Write mutation history to Audit sheet
        //   7. Save and close workbook
        Err(LedgerOpError::NotImplemented(
            "ExportWorkbookOp: workbook writer integration not yet wired".to_string(),
        ))
    }
}

/// Generate a full audit trail document.
pub struct GenerateAuditTrailOp {
    pub output_path: PathBuf,
    pub year: i32,
}

impl LedgerOperation for GenerateAuditTrailOp {
    fn id(&self) -> &str {
        "generate-audit-trail"
    }

    fn description(&self) -> &str {
        "Generate a CPA-auditable audit trail document for a tax year"
    }

    fn execute(&self, _ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        // Intended logic:
        //   1. Query all mutation events for `self.year` from audit log
        //   2. Serialize to a structured JSON/XLSX audit document
        //   3. Include: ingest timestamps, classification changes, reconciliation
        //      outcomes, human review sign-offs
        //   4. Write to `self.output_path`
        Err(LedgerOpError::NotImplemented(format!(
            "GenerateAuditTrailOp: audit trail export not yet wired (year={})",
            self.year
        )))
    }
}

/// Check a tax deadline and emit an advisory issue if it is approaching.
pub struct CheckTaxDeadlineOp {
    pub deadline_id: String,
    pub warn_days_before: u32,
}

impl LedgerOperation for CheckTaxDeadlineOp {
    fn id(&self) -> &str {
        "check-tax-deadline"
    }

    fn description(&self) -> &str {
        "Check a scheduled tax deadline and emit advisory issues if approaching"
    }

    fn execute(&self, ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        // Intended logic:
        //   1. Look up `self.deadline_id` in `ctx.calendar`
        //   2. Compute next due date via BusinessCalendar::next_due
        //   3. If today + warn_days_before >= due_date → emit advisory issue
        //   4. Return result with issue text if approaching
        //
        // For now, this requires a calendar to be attached.
        let _ = ctx;
        Err(LedgerOpError::NotImplemented(format!(
            "CheckTaxDeadlineOp: calendar lookup not yet wired (deadline={})",
            self.deadline_id
        )))
    }
}

// ---------------------------------------------------------------------------
// Dispatcher
// ---------------------------------------------------------------------------

/// Collects and runs multiple [`LedgerOperation`] instances.
#[derive(Default)]
pub struct OperationDispatcher {
    ops: Vec<Box<dyn LedgerOperation>>,
}

impl OperationDispatcher {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn register(&mut self, op: Box<dyn LedgerOperation>) -> &mut Self {
        self.ops.push(op);
        self
    }

    /// Run every registered operation and collect results.
    pub fn run_all(
        &self,
        ctx: &OperationContext,
    ) -> Vec<Result<OperationResult, LedgerOpError>> {
        self.ops.iter().map(|op| op.execute(ctx)).collect()
    }

    /// Run the first operation whose `id()` matches, returning `None` if not found.
    pub fn run_by_id(
        &self,
        id: &str,
        ctx: &OperationContext,
    ) -> Option<Result<OperationResult, LedgerOpError>> {
        self.ops.iter().find(|op| op.id() == id).map(|op| op.execute(ctx))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_ctx() -> OperationContext {
        OperationContext::new(PathBuf::from("/tmp/working"), PathBuf::from("/tmp/rules"))
    }

    #[test]
    fn operation_result_success_constructor() {
        let r = OperationResult::success("test-op", 42);
        assert!(r.success);
        assert_eq!(r.items_processed, 42);
        assert_eq!(r.operation_id, "test-op");
        assert!(r.issues.is_empty());
    }

    #[test]
    fn operation_result_failure_constructor() {
        let r = OperationResult::failure("test-op", "something broke");
        assert!(!r.success);
        assert_eq!(r.issues.len(), 1);
        assert!(r.issues[0].contains("something broke"));
    }

    #[test]
    fn operation_context_new() {
        let ctx = OperationContext::new(
            PathBuf::from("/work"),
            PathBuf::from("/rules"),
        );
        assert_eq!(ctx.working_dir, PathBuf::from("/work"));
        assert_eq!(ctx.rules_dir, PathBuf::from("/rules"));
        assert!(!ctx.dry_run);
        assert!(ctx.calendar.is_none());
    }

    #[test]
    fn operation_context_builder_dry_run() {
        let ctx = OperationContext::new(PathBuf::from("/w"), PathBuf::from("/r"))
            .dry_run();
        assert!(ctx.dry_run);
    }

    #[test]
    fn dispatcher_register_and_find_by_id() {
        let mut dispatcher = OperationDispatcher::new();
        dispatcher.register(Box::new(CheckTaxDeadlineOp {
            deadline_id: "us-q1".to_string(),
            warn_days_before: 30,
        }));

        let ctx = test_ctx();
        let result = dispatcher.run_by_id("check-tax-deadline", &ctx);
        assert!(result.is_some(), "should find operation by id");
    }

    #[test]
    fn dispatcher_run_by_id_not_found_returns_none() {
        let dispatcher = OperationDispatcher::new();
        let ctx = test_ctx();
        let result = dispatcher.run_by_id("nonexistent", &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn check_tax_deadline_returns_not_implemented_gracefully() {
        let op = CheckTaxDeadlineOp {
            deadline_id: "us-annual".to_string(),
            warn_days_before: 30,
        };
        let ctx = test_ctx();
        let result = op.execute(&ctx);
        match result {
            Err(LedgerOpError::NotImplemented(_)) => {} // expected
            other => panic!("expected NotImplemented, got {other:?}"),
        }
    }

    #[test]
    fn dispatcher_run_all_collects_results() {
        let mut dispatcher = OperationDispatcher::new();
        dispatcher.register(Box::new(CheckTaxDeadlineOp {
            deadline_id: "us-q1".to_string(),
            warn_days_before: 14,
        }));
        dispatcher.register(Box::new(CheckTaxDeadlineOp {
            deadline_id: "us-annual".to_string(),
            warn_days_before: 30,
        }));
        let ctx = test_ctx();
        let results = dispatcher.run_all(&ctx);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn ingest_op_is_idempotent() {
        let op = IngestStatementOp {
            source_glob: "statements/*.pdf".to_string(),
            vendor_hint: None,
        };
        assert!(op.is_idempotent());
    }
}
