---
phase: 05-cpa-workbook-outputs
plan: 01
type: execute
wave: 1
depends_on: ["04-audit-integrity-safe-reconciliation"]
files_modified:
  - crates/turbo-mcp/Cargo.toml
  - crates/turbo-mcp/src/lib.rs
  - crates/turbo-mcp/tests/phase5_cpa_outputs.rs
autonomous: true
requirements: ["WB-01", "WB-02", "WB-03", "TAX-01", "TAX-02", "TAX-03", "TAX-04", "MCP-04"]
must_haves:
  truths:
    - "CPA workbook export includes transaction sheets, taxonomy, and flag sheets."
    - "Schedule C/D/E and FBAR summaries are available for a given tax year."
    - "MCP get_schedule_summary(year, schedule) returns deterministic totals."
---

<objective>
Deliver Phase 5 CPA workbook usability outputs and summary retrieval APIs.
</objective>

<task_checklist>
- [x] Task 1: Add failing tests for workbook export and schedule summary MCP retrieval.
- [x] Task 2: Implement workbook export contract from ledger classification state.
- [x] Task 3: Implement year-scoped schedule summary aggregation and verify full regression.
</task_checklist>

