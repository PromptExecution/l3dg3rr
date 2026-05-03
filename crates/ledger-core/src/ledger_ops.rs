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
use crate::calendar::{BusinessCalendar, ScheduledEvent};
use crate::classify::ClassifiedTransaction;

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
    #[error("workbook error: {0}")]
    Workbook(String),
    #[error("external process failed: {0}")]
    ExternalProcessFailed(String),
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
    /// Optional single-file input path for `IngestStatementOp`.
    pub input_path: Option<PathBuf>,
    /// Pre-classified transactions for `ExportWorkbookOp`.
    pub classified_transactions: Vec<ClassifiedTransaction>,
    /// Optional OPA policy bundle path for `OpaGateOp` (phase-3).
    pub opa_bundle_path: Option<PathBuf>,
}

impl OperationContext {
    pub fn new(working_dir: PathBuf, rules_dir: PathBuf) -> Self {
        Self {
            working_dir,
            rules_dir,
            calendar: None,
            dry_run: false,
            input_path: None,
            classified_transactions: Vec::new(),
            opa_bundle_path: None,
        }
    }

    pub fn with_input_path(mut self, path: PathBuf) -> Self {
        self.input_path = Some(path);
        self
    }

    pub fn with_classified_transactions(mut self, txs: Vec<ClassifiedTransaction>) -> Self {
        self.classified_transactions = txs;
        self
    }

    pub fn with_opa_bundle_path(mut self, path: PathBuf) -> Self {
        self.opa_bundle_path = Some(path);
        self
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

    fn execute(&self, ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        use crate::document::DocType;
        use crate::document_shape::classify_document_shape;
        use crate::ingest::{IngestedLedger, TransactionInput};
        use calamine::{open_workbook_auto, Reader};

        let input_path = ctx.input_path.as_ref().ok_or_else(|| {
            LedgerOpError::InvalidInput("input_path not set in context".to_string())
        })?;

        let filename = input_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        let doc_type = DocType::from_path(input_path);

        // Read a small sample for shape classification (first 2 KB of the file
        // for CSV; not applicable for XLSX — just use the filename).
        let sample_content = if matches!(doc_type, DocType::SpreadsheetCsv) {
            std::fs::read_to_string(input_path)
                .map(|s| s.chars().take(2048).collect::<String>())
                .unwrap_or_default()
        } else {
            String::new()
        };

        let shape = classify_document_shape(&doc_type, filename, &sample_content);

        // Resolve canonical column names from the shape's column_map.
        // column_map: canonical → source_header. We need to find the
        // 0-based column index for "date", "amount", "description".
        //
        // For XLSX/CSV via calamine, we scan the header row.
        let mut workbook = open_workbook_auto(input_path)
            .map_err(|e| LedgerOpError::InvalidInput(format!("calamine open: {e}")))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let first_sheet = sheet_names
            .first()
            .cloned()
            .ok_or_else(|| LedgerOpError::InvalidInput("no sheets in file".to_string()))?;

        let range = workbook
            .worksheet_range(&first_sheet)
            .map_err(|e| LedgerOpError::InvalidInput(format!("calamine range: {e}")))?;

        let mut rows_iter = range.rows();

        // Read header row to build column-index map
        let header_row = match rows_iter.next() {
            Some(h) => h,
            None => {
                return Ok(OperationResult::success("ingest-statement", 0));
            }
        };

        // Build header → index map from the actual file
        let header_map: std::collections::HashMap<String, usize> = header_row
            .iter()
            .enumerate()
            .filter_map(|(i, cell)| {
                let s = cell.to_string().trim().to_ascii_lowercase();
                if s.is_empty() {
                    None
                } else {
                    Some((s, i))
                }
            })
            .collect();

        // Resolve canonical names through shape.column_map → actual header name → index
        let find_col = |canon: &str| -> Option<usize> {
            // First try shape column_map canonical → source_header → index
            if let Some(source_header) = shape.column_map.get(canon) {
                let lower = source_header.to_ascii_lowercase();
                if let Some(&idx) = header_map.get(&lower) {
                    return Some(idx);
                }
            }
            // Fallback: direct canonical name match in header
            header_map.get(canon).copied()
        };

        let date_col = find_col("date");
        let amount_col = find_col("amount");
        let desc_col = find_col("description");

        // Derive account_id from filename (vendor slug or filename stem)
        let account_id = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.split("--").next().unwrap_or(s).to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let mut transactions: Vec<TransactionInput> = Vec::new();

        for row in rows_iter {
            let get_cell = |col: Option<usize>| -> String {
                col.and_then(|i| row.get(i))
                    .map(|c| c.to_string().trim().to_string())
                    .unwrap_or_default()
            };

            let date = get_cell(date_col);
            let amount = get_cell(amount_col);
            let description = get_cell(desc_col);

            // Skip empty rows
            if date.is_empty() && amount.is_empty() && description.is_empty() {
                continue;
            }

            transactions.push(TransactionInput {
                account_id: account_id.clone(),
                date,
                amount,
                description,
                source_ref: filename.to_string(),
            });
        }

        let count = transactions.len();
        let mut ledger = IngestedLedger::default();
        ledger.ingest(&transactions);

        Ok(OperationResult::success("ingest-statement", count))
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

    fn execute(&self, ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        use crate::workbook::TxProjectionRow;
        use rust_xlsxwriter::Workbook;

        let txs = &ctx.classified_transactions;

        if ctx.dry_run {
            return Ok(OperationResult::success("export-workbook", txs.len()));
        }

        // Route transactions to sheet groups by category
        let mut sched_c: Vec<TxProjectionRow> = Vec::new();
        let mut sched_d: Vec<TxProjectionRow> = Vec::new();
        let mut sched_e: Vec<TxProjectionRow> = Vec::new();
        let mut fbar: Vec<TxProjectionRow> = Vec::new();
        let mut flags_open: Vec<TxProjectionRow> = Vec::new();

        for tx in txs {
            let row = TxProjectionRow {
                tx_id: tx.tx_id.clone(),
                account_id: String::new(), // not carried in ClassifiedTransaction
                date: String::new(),
                amount: String::new(),
                description: String::new(),
                source_ref: tx.reason.clone(),
            };

            if tx.needs_review {
                if tx.category == "ForeignIncome" {
                    fbar.push(row.clone());
                }
                flags_open.push(row);
                continue;
            }

            match tx.category.as_str() {
                "SelfEmployment" => sched_c.push(row),
                "CapitalGain" | "CryptoGain" | "CryptoLoss" => sched_d.push(row),
                "RentalIncome" => sched_e.push(row),
                _ => {} // Other categories not yet routed to a specific sheet
            }
        }

        // Materialize the workbook with all required sheets
        let mut workbook = Workbook::new();
        for sheet_name in crate::workbook::REQUIRED_SHEETS {
            workbook
                .add_worksheet()
                .set_name(*sheet_name)
                .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
        }

        // Write each sheet group
        let write_sheet = |wb: &mut Workbook,
                           sheet_name: &str,
                           rows: &[TxProjectionRow]|
         -> Result<(), LedgerOpError> {
            let ws = wb
                .worksheet_from_name(sheet_name)
                .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
            ws.write_string(0, 0, "tx_id")
                .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
            ws.write_string(0, 1, "account_id")
                .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
            ws.write_string(0, 2, "date")
                .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
            ws.write_string(0, 3, "amount")
                .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
            ws.write_string(0, 4, "description")
                .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
            ws.write_string(0, 5, "source_ref")
                .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
            for (idx, row) in rows.iter().enumerate() {
                let r = (idx + 1) as u32;
                ws.write_string(r, 0, &row.tx_id)
                    .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
                ws.write_string(r, 1, &row.account_id)
                    .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
                ws.write_string(r, 2, &row.date)
                    .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
                ws.write_string(r, 3, &row.amount)
                    .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
                ws.write_string(r, 4, &row.description)
                    .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
                ws.write_string(r, 5, &row.source_ref)
                    .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;
            }
            Ok(())
        };

        write_sheet(&mut workbook, "SCHED.C", &sched_c)?;
        write_sheet(&mut workbook, "SCHED.D", &sched_d)?;
        write_sheet(&mut workbook, "SCHED.E", &sched_e)?;
        write_sheet(&mut workbook, "FBAR.accounts", &fbar)?;
        write_sheet(&mut workbook, "FLAGS.open", &flags_open)?;

        workbook
            .save(&self.output_path)
            .map_err(|e| LedgerOpError::Workbook(e.to_string()))?;

        let total = sched_c.len() + sched_d.len() + sched_e.len() + fbar.len() + flags_open.len();
        Ok(OperationResult::success("export-workbook", total))
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
        &self.deadline_id
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
        // For now, just return success if calendar is not available.
        let _calendar = &ctx.calendar;

        // TODO: Implement full calendar lookup when calendar integration is complete
        Ok(OperationResult::success("check-tax-deadline", 0))
    }
}

/// Ingest a PDF statement file via the `reqif-opa-mcp` Python sidecar.
///
/// This op is a Phase 2 stub. See the TODO below for the intended implementation.
pub struct PdfIngestOp {
    pub input_path: PathBuf,
}

impl LedgerOperation for PdfIngestOp {
    fn id(&self) -> &str {
        "pdf-ingest"
    }

    fn description(&self) -> &str {
        "Ingest a PDF statement file via the reqif-opa-mcp Python sidecar (phase-2)"
    }

    fn is_idempotent(&self) -> bool {
        // Blake3 content-hash IDs prevent duplicate records on re-ingest
        true
    }

    fn execute(&self, _ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        // TODO(phase-2): PDF ingestion via reqif-opa-mcp subprocess
        //
        // Intended behavior:
        //   1. Spawn subprocess: `reqif-opa-mcp ingest --file <path> --output ndjson`
        //      (reqif-opa-mcp is a Python CLI at https://github.com/PromptExecution/reqif-opa-mcp)
        //   2. Read NDJSON lines from stdout; deserialize each line as ReqIfCandidate
        //      (type is already defined in rule_registry.rs)
        //   3. For each candidate: call RuleRegistry::classify_waterfall with candidate fields
        //      mapped to a tx HashMap (id→tx_id, text→description, metadata.amount→amount)
        //   4. Emit ClassificationOutcome rows to the workbook via ExportWorkbookOp
        //   5. Error handling: if subprocess exits non-zero, return LedgerOpError::ExternalProcessFailed
        //      with stdout+stderr captured for audit logging
        //   6. This op should be idempotent: the Blake3 content hash prevents duplicate rows
        //      even if the PDF is re-ingested
        Err(LedgerOpError::NotImplemented(
            "PdfIngestOp: PDF ingestion via reqif-opa-mcp not yet implemented (phase-2)"
                .to_string(),
        ))
    }
}

/// Gate classified transactions through OPA policies before workbook commit.
///
/// This op is a Phase 3 stub. See the TODO below for the intended implementation.
pub struct OpaGateOp {
    pub policy_bundle_path: Option<PathBuf>,
}

impl LedgerOperation for OpaGateOp {
    fn id(&self) -> &str {
        "opa-gate"
    }

    fn description(&self) -> &str {
        "Run classified transactions through OPA policy gate before workbook commit (phase-3)"
    }

    fn execute(&self, _ctx: &OperationContext) -> Result<OperationResult, LedgerOpError> {
        // TODO(phase-3): OPA (Open Policy Agent) gate integration
        //
        // Intended behavior:
        //   1. Before committing classified transactions to the workbook, run each
        //      ClassificationOutcome through an OPA policy bundle
        //   2. Policy bundle path: configured via OperationContext.opa_bundle_path (add this field)
        //   3. OPA HTTP API: POST to http://localhost:8181/v1/data/ledger/allow
        //      with body: { "input": { "category": "...", "confidence": 0.9, "review": false } }
        //   4. If OPA returns { "result": false }, move the transaction to FLAGS.open with
        //      reason "opa_gate_rejected" instead of committing to schedule sheet
        //   5. If OPA is unreachable (connection refused), fall through with a warning logged
        //      via tracing::warn! — do not block pipeline for a missing OPA sidecar
        //   6. OPA policy source lives in opa/policies/ledger_classify.rego (create this directory)
        Err(LedgerOpError::NotImplemented(
            "OpaGateOp: OPA gate not yet implemented (phase-3)".to_string(),
        ))
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

    /// Create a dispatcher from a slice of scheduled events.
    ///
    /// Each event's `operation` field is converted to a concrete operation struct
    /// and registered with the dispatcher.
    pub fn from_scheduled_events(events: &[ScheduledEvent]) -> Self {
        let mut dispatcher = Self::new();

        for event in events {
            let op: Box<dyn LedgerOperation> = match &event.operation {
                OperationKind::CheckTaxDeadline { deadline_id } => Box::new(CheckTaxDeadlineOp {
                    deadline_id: deadline_id.clone(),
                    warn_days_before: 30,
                }),
                OperationKind::IngestStatement { source_glob } => Box::new(IngestStatementOp {
                    source_glob: source_glob.clone(),
                    vendor_hint: None,
                }),
                OperationKind::ClassifyTransactions { rule_dir } => {
                    Box::new(ClassifyTransactionsOp {
                        rule_dir: PathBuf::from(rule_dir),
                        review_threshold: 0.8,
                        account_filter: None,
                    })
                }
                OperationKind::ReconcileAccount { account_id } => Box::new(ReconcileAccountOp {
                    account_id: account_id.clone(),
                    dry_run: false,
                }),
                OperationKind::ExportWorkbook { output_path } => Box::new(ExportWorkbookOp {
                    output_path: PathBuf::from(output_path),
                    include_flags: true,
                }),
                OperationKind::GenerateAuditTrail { year } => Box::new(GenerateAuditTrailOp {
                    output_path: PathBuf::from(format!("audit-trail-{}.xlsx", year)),
                    year: *year,
                }),
            };

            dispatcher.ops.push(op);
        }

        dispatcher
    }

    /// Run every registered operation and collect results.
    pub fn run_all(&self, ctx: &OperationContext) -> Vec<Result<OperationResult, LedgerOpError>> {
        self.ops.iter().map(|op| op.execute(ctx)).collect()
    }

    /// Run the first operation whose `id()` matches, returning `None` if not found.
    pub fn run_by_id(
        &self,
        id: &str,
        ctx: &OperationContext,
    ) -> Option<Result<OperationResult, LedgerOpError>> {
        self.ops
            .iter()
            .find(|op| op.id() == id)
            .map(|op| op.execute(ctx))
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
        let ctx = OperationContext::new(PathBuf::from("/work"), PathBuf::from("/rules"));
        assert_eq!(ctx.working_dir, PathBuf::from("/work"));
        assert_eq!(ctx.rules_dir, PathBuf::from("/rules"));
        assert!(!ctx.dry_run);
        assert!(ctx.calendar.is_none());
    }

    #[test]
    fn operation_context_builder_dry_run() {
        let ctx = OperationContext::new(PathBuf::from("/w"), PathBuf::from("/r")).dry_run();
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
        let result = dispatcher.run_by_id("us-q1", &ctx);
        assert!(result.is_some(), "should find operation by its deadline_id");
    }

    #[test]
    fn dispatcher_run_by_id_not_found_returns_none() {
        let dispatcher = OperationDispatcher::new();
        let ctx = test_ctx();
        let result = dispatcher.run_by_id("nonexistent", &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn check_tax_deadline_returns_success() {
        let op = CheckTaxDeadlineOp {
            deadline_id: "us-annual".to_string(),
            warn_days_before: 30,
        };
        let ctx = test_ctx();
        let result = op.execute(&ctx);
        match result {
            Ok(op_result) => {
                assert!(op_result.success);
                assert_eq!(op_result.operation_id, "check-tax-deadline");
            }
            other => panic!("expected success, got {other:?}"),
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
