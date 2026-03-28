---
phase: 04-audit-integrity-safe-reconciliation
plan: 01
type: execute
wave: 1
depends_on: ["03-rule-driven-classification-flagging"]
files_modified:
  - crates/turbo-mcp/Cargo.toml
  - crates/turbo-mcp/src/lib.rs
  - crates/turbo-mcp/tests/phase4_audit_integrity.rs
  - crates/ledger-core/src/classify.rs
autonomous: true
requirements: ["AUD-01", "AUD-02", "AUD-03", "AUD-04", "MCP-02"]
must_haves:
  truths:
    - "Classification mutations append field-level audit entries with actor and note."
    - "Excel-originated classification reconciliation reuses mutation+audit path."
    - "Confidence and amount validations use decimal-safe parsing."
    - "Invariant checks reject bad tx_id/hash/schema conditions before mutation."
  artifacts:
    - path: "crates/turbo-mcp/src/lib.rs"
      provides: "classify_transaction, reconcile_excel_classification, query_audit_log contracts."
    - path: "crates/turbo-mcp/tests/phase4_audit_integrity.rs"
      provides: "Requirement-tagged behavior checks for AUD-01..04 and MCP-02."
---

<objective>
Deliver append-only audit integrity and safe reconciliation over classification mutation paths.
</objective>

<task_checklist>
- [x] Task 1: Add failing Phase 4 tests for audit trail, reconciliation, decimal safety, and invariants.
- [x] Task 2: Implement mutation audit model and MCP classify/reconcile/query contracts.
- [x] Task 3: Enforce decimal+invariant checks and verify workspace regression suite.
</task_checklist>

