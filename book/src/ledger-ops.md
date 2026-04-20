# Ledger Operations

## Overview

The `LedgerOperation` trait is the primitive interface for every discrete action the pipeline can take: ingesting a statement, classifying transactions, checking a tax deadline, exporting the CPA workbook. Composing these operations through an `OperationDispatcher` rather than calling them directly provides:

- **Idempotency guarantees**: Each operation implementation declares whether it is idempotent. The dispatcher enforces this contract and deduplicates re-triggered operations where safe.
- **Uniform result surface**: Every operation returns an `OperationResult` that flows into the audit trail, regardless of what the operation does internally.
- **Calendar integration**: `ScheduledEvent` records carry an `OperationKind` that the dispatcher resolves to a concrete operation at runtime. The calendar and the operation layer are decoupled.
- **Agent-editable dispatch rules**: Rhai rules can inspect `OperationContext` fields and short-circuit or redirect operations without changing Rust code.

## Operation Dispatch Flow

```rhai
fn receive_trigger() -> resolve_operation
fn resolve_operation() -> validate_context
fn validate_context() -> execute_operation
fn execute_operation() -> record_result
if result_success -> mark_complete
if result_failure -> emit_issue
```

- `receive_trigger` — accepts an `OperationKind` from a scheduled event, MCP call, or manual invocation.
- `resolve_operation` — looks up the registered `LedgerOperation` implementation for the given `OperationKind`.
- `validate_context` — checks that `OperationContext` contains required fields (e.g. `journal_path`, `workbook_path`); returns `Err` if preconditions are not met.
- `execute_operation` — calls the operation's `execute()` method; the operation is responsible for its own internal error handling.
- `record_result` — writes the `OperationResult` (success or failure) to the audit trail with timestamps and the triggering event ID.
- `mark_complete` — on success, updates the scheduler's completion record so the event is not re-fired.
- `emit_issue` — on failure, emits a structured issue record for operator review; does not re-trigger automatically.

## Dispatcher Architecture

```rhai
fn operation_dispatcher() -> operation_context
fn operation_dispatcher() -> ingest_statement_op
fn operation_dispatcher() -> classify_transactions_op
fn operation_dispatcher() -> check_tax_deadline_op
fn operation_dispatcher() -> export_workbook_op
fn operation_dispatcher() -> generate_audit_trail_op
fn ingest_statement_op() -> operation_result
fn classify_transactions_op() -> operation_result
fn check_tax_deadline_op() -> operation_result
fn export_workbook_op() -> operation_result
fn generate_audit_trail_op() -> operation_result
fn operation_result() -> audit_trail
```

`IngestStatementOp` is annotated as idempotent: re-ingesting the same source file produces the same Blake3 content-hash transaction IDs and the dispatcher skips duplicate writes.

## Operations Reference

| Operation | ID | Idempotent | Status | Description |
|-----------|-----|-----------|--------|-------------|
| `IngestStatementOp` | `IngestStatement` | Yes (Blake3 dedup) | Implemented | Parse a source PDF or CSV, extract transactions, write to journal and workbook |
| `ClassifyTransactionsOp` | `ClassifyTransactions` | Yes (rule determinism) | Implemented | Run the Rhai classification waterfall over unclassified transactions |
| `CheckTaxDeadlineOp` | `CheckTaxDeadline` | Yes | Implemented | Emit a deadline notification record; no data mutation |
| `ExportWorkbookOp` | `ExportWorkbook` | Yes (overwrite) | Implemented | Write the current journal state to the CPA Excel workbook |
| `GenerateAuditTrailOp` | `GenerateAuditTrail` | Yes (overwrite) | Implemented | Produce the year-end audit trail report for a given tax year |

## Document Shape Classification

Before `IngestStatementOp` can extract transactions, it must know which extraction profile to use. `classify_document_shape()` in `document_shape.rs` (and mirrored in `rules/classify_document_shape.rhai`) maps a raw document to a `DocumentShape`:

```rhai
fn ingest_file() -> detect_shape
fn detect_shape() -> route_extractor
fn route_extractor() -> extract_transactions
fn extract_transactions() -> classify_transactions
```

- `ingest_file` — receives a source path matching the `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE` naming convention.
- `detect_shape` — calls `classify_document_shape()` with filename, doc_type, and a content sample. Returns `DocumentShape` with vendor, account_type, statement_format, currency, confidence, and signals.
- `route_extractor` — selects the extraction backend based on `statement_format`: `csv_generic`, `csv_ofx`, `pdf_tabular`, or `xlsx_native`.
- `extract_transactions` — runs the selected extractor; outputs raw `Transaction` rows with `Decimal` amounts.
- `classify_transactions` — passes extracted transactions through the Rhai classification waterfall in `rules/`.

Shape detection uses a confidence score. If confidence is below 0.5, `IngestStatementOp` emits a review flag and halts rather than ingesting with an unknown vendor profile.

### DocumentShape Fields

| Field | Type | Description |
|-------|------|-------------|
| `vendor` | `StatementVendor` | Institution slug: `WellsFargo`, `Chase`, `Anz`, `Commbank`, etc. |
| `account_type` | String | `checking`, `savings`, `brokerage`, `crypto` |
| `statement_format` | String | `csv_generic`, `csv_ofx`, `pdf_tabular`, `xlsx_native` |
| `currency` | String | `USD`, `AUD`, `EUR`, `GBP` |
| `confidence` | f64 | 0.0–1.0 heuristic score based on filename slug and content signals |
| `signals` | Vec\<String\> | Matched signal names for audit; e.g. `filename_vendor_slug`, `csv_header_match` |
| `reason` | String | Human-readable explanation for the audit trail |

## Integration with Calendar

`ScheduledEvent` in the TOML calendar manifests carries an `operation` field that is an `OperationKind` variant. When `BusinessCalendar::upcoming()` returns a due event, the caller passes `event.operation` to `OperationDispatcher::dispatch()`:

```rust
let due = calendar.upcoming(today, 30);
for event in due {
    let result = dispatcher.dispatch(&event.operation, &context).await?;
    audit_trail.record(event.id, result);
}
```

The TOML `operation` inline table maps directly to `OperationKind` enum variants:

```toml
# Maps to OperationKind::IngestStatement { source_glob: "samples/**/*.pdf" }
operation = { type = "IngestStatement", source_glob = "samples/**/*.pdf" }

# Maps to OperationKind::ClassifyTransactions { rule_dir: "rules" }
operation = { type = "ClassifyTransactions", rule_dir = "rules" }

# Maps to OperationKind::CheckTaxDeadline { deadline_id: "fbar_deadline" }
operation = { type = "CheckTaxDeadline", deadline_id = "fbar_deadline" }
```

This bidirectional mapping means calendar manifests are the single source of truth for what runs, when it runs, and what it does — the Rust dispatcher merely executes.
