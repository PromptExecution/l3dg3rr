---
phase: 03-rule-driven-classification-flagging
plan: 01
type: execute
wave: 1
depends_on: ["02-deterministic-ingestion-pipeline"]
files_modified:
  - crates/ledger-core/Cargo.toml
  - crates/ledger-core/src/lib.rs
  - crates/ledger-core/src/classify.rs
  - crates/ledger-core/tests/phase3_classification.rs
  - crates/turbo-mcp/src/lib.rs
  - crates/turbo-mcp/tests/phase3_mcp_classification.rs
autonomous: true
requirements: ["CLSF-01", "CLSF-02", "CLSF-03", "CLSF-04", "MCP-03", "MCP-07"]
must_haves:
  truths:
    - "User can load and run Rhai rules at runtime from rules/classify.rhai-style files without recompilation."
    - "User can classify ingested transactions into category and confidence outputs deterministically for a given rule file and transaction input."
    - "Low-confidence or policy-requested outcomes create open review flags queryable by year/status."
    - "Candidate rules can be tested on sample transactions through MCP run_rhai_rule without mutating review queue state."
  artifacts:
    - path: "crates/ledger-core/src/classify.rs"
      provides: "Runtime Rhai classification engine with review-flag emission/query."
    - path: "crates/turbo-mcp/src/lib.rs"
      provides: "MCP contracts: run_rhai_rule, classify_ingested, query_flags."
---

<objective>
Deliver Phase 3 classification and review-queue behavior with runtime Rhai rules and explicit MCP operator contracts.
</objective>

<task_checklist>
- [x] Task 1: Add failing tests for CLSF-01..04 and MCP-03/MCP-07.
- [x] Task 2: Implement ledger-core runtime Rhai classification + review-flag query model.
- [x] Task 3: Wire Turbo MCP tools for run_rhai_rule/classify_ingested/query_flags and verify full workspace.
</task_checklist>
