# Workbook & Audit

The workbook is the accountant-facing artifact. The journal, audit log, and sidecar state are machine-facing recovery surfaces. The application keeps these roles separate so a CPA can inspect and sign off in Excel while agents still get deterministic replay and audit evidence.

## Workbook Contract

`ledger_core::workbook::REQUIRED_SHEETS` is the base workbook contract. Export paths should rebuild the workbook from canonical service state rather than mutating partial output in place.

Required workbook concerns:

- transaction projection rows with stable transaction IDs
- account registry and metadata sheets
- Schedule-oriented summaries
- review and ambiguity flags
- audit log projection
- configuration and manifest metadata

```rhai
fn canonical_state() -> workbook_projection
fn workbook_projection() -> transaction_sheets
fn workbook_projection() -> schedule_summaries
fn workbook_projection() -> flag_sheets
fn workbook_projection() -> audit_sheet
fn audit_sheet() -> cpa_review
```

## Audit Flow

Every meaningful mutation should have an audit event before it becomes externally visible. Classification edits, reconciliation commits, lifecycle transitions, and workbook exports should all be explainable from event history.

Machine recovery state lives in deterministic sidecars next to the manifest workbook path. If a sidecar exists but cannot be parsed or has an unsupported version, the service should fail closed rather than silently resetting state.

## Journal Flow

The NDJSON journal provides append/replay behavior for ingested transactions. Transaction identity is content-addressed with Blake3 over account, date, amount, and description, making repeated ingest idempotent.

## Projection Rule

The workbook is a projection, not the only system of record for agent queues or replay state. It remains the canonical human/audit layer because the CPA workflow depends on Excel, but operational restart state belongs in the sidecar snapshot.

## Related Chapters

- [Document Ingestion](./document-ingestion.md)
- [Validation](./validation.md)
- [MCP Surface](./mcp-surface.md)
- [Pipeline](./pipeline.md)
