---
phase: 02-deterministic-ingestion-pipeline
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/ledger-core/Cargo.toml
  - crates/ledger-core/src/ingest.rs
  - crates/ledger-core/src/journal.rs
  - crates/ledger-core/src/workbook.rs
  - crates/ledger-core/src/lib.rs
  - crates/ledger-core/tests/phase2_ingest_pipeline_remaining.rs
  - crates/turbo-mcp/src/lib.rs
  - crates/turbo-mcp/tests/phase2_mcp_contract_remaining.rs
autonomous: true
requirements: ["ING-01", "ING-02", "ING-03", "ING-04", "MCP-01", "MCP-05"]
must_haves:
  truths:
    - "User can ingest a contract-valid PDF path and produce deterministic tx IDs plus persisted evidence reference."
    - "User can re-run ingest for the same source with zero duplicate journal/workbook transactions."
    - "User can retrieve raw source context bytes via MCP get_raw_context(rkyv_ref) for any ingested transaction."
  artifacts:
    - path: "crates/ledger-core/src/ingest.rs"
      provides: "Deterministic ingest orchestration and replay-safe dedupe behavior."
    - path: "crates/ledger-core/src/journal.rs"
      provides: "Rustledger-compatible Beancount append output with txid/source_ref metadata."
    - path: "crates/ledger-core/src/workbook.rs"
      provides: "TX.<account-id> materialization path from deterministic ingest outputs."
    - path: "crates/turbo-mcp/src/lib.rs"
      provides: "MCP ingest_pdf and get_raw_context tool contracts wired to ingest pipeline."
  key_links:
    - from: "turbo-mcp::ingest_pdf"
      to: "ledger-core ingest pipeline"
      via: "validated filename -> deterministic ingest -> journal/workbook writes"
      pattern: "validate_source_filename.*ingest.*journal.*TX\\."
    - from: "ledger ingest output source_ref"
      to: "turbo-mcp::get_raw_context"
      via: "persisted .rkyv path reference"
      pattern: "source_ref.*rkyv_ref.*std::fs::read"
---

<objective>
Close the remaining Phase 2 work for deterministic ingestion without re-planning already completed behavior.

Purpose: finish end-to-end deterministic ingest from PDF path through rustledger-compatible Beancount persistence, TX sheet materialization, and MCP evidence retrieval.
Output: passing Phase 2 TDD/integration tests for ING-01..04 + MCP-01/MCP-05 with replay-safe guarantees.
</objective>

<execution_context>
@/home/brianh/promptexecution/mbse/l3dg3rr/.codex/get-shit-done/workflows/execute-plan.md
@/home/brianh/promptexecution/mbse/l3dg3rr/.codex/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/REQUIREMENTS.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/02-deterministic-ingestion-pipeline/02-CONTEXT.md
@crates/ledger-core/src/ingest.rs
@crates/ledger-core/src/journal.rs
@crates/ledger-core/src/workbook.rs
@crates/turbo-mcp/src/lib.rs

<decisions_locked>
- D-01: Enforce filename preflight contract before mutation (`VENDOR--ACCOUNT--YYYY-MM--DOCTYPE`).
- D-02: Deterministic content-hash IDs remain the idempotency key (no random or sequence IDs).
- D-03: Re-ingest must be replay-safe and duplicate-free.
- D-04: Persist explicit `.rkyv` source references and use them for deterministic raw-context lookup.
- D-05: Keep rustledger-compatible plain-text Beancount journal direction; do not replace with DB-first persistence.
</decisions_locked>

<already_complete_do_not_replan>
- Deterministic `deterministic_tx_id` behavior tests.
- Baseline replay-safe in-memory ingest/journal append tests.
- Baseline MCP `list_accounts` contract.
</already_complete_do_not_replan>
</context>

<task_checklist>
- [ ] Task 1 (no dependencies): Write failing TDD tests for all remaining Phase 2 outcomes.
- [ ] Task 2 (depends on Task 1): Implement ingest pipeline and persistence wiring to satisfy new tests while preserving existing passing tests.
- [ ] Task 3 (depends on Task 2): Finalize MCP contracts and run full verification suite for Phase 2 requirements.
</task_checklist>

<requirements_traceability>
- Task 1 covers ING-01, ING-02, ING-03, ING-04, MCP-01, MCP-05 by defining failing behavior tests.
- Task 2 implements ING-01, ING-02, ING-03, ING-04 in `ledger-core` per D-01/D-02/D-03/D-04/D-05.
- Task 3 implements/verifies MCP-01 and MCP-05 against the completed core ingest path.
</requirements_traceability>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Define remaining Phase 2 behavior as failing tests</name>
  <read_first>.planning/phases/02-deterministic-ingestion-pipeline/02-CONTEXT.md, .planning/phases/02-deterministic-ingestion-pipeline/02-RESEARCH.md, crates/ledger-core/tests, crates/turbo-mcp/tests, crates/ledger-core/src/ingest.rs, crates/turbo-mcp/src/lib.rs</read_first>
  <files>crates/ledger-core/tests/phase2_ingest_pipeline_remaining.rs, crates/turbo-mcp/tests/phase2_mcp_contract_remaining.rs</files>
  <behavior>
    - Test 1 (ING-01, D-05): ingest from contract-valid PDF path writes Beancount entry and materializes `TX.<account-id>` row state.
    - Test 2 (ING-02, D-02/D-03): re-ingesting same source yields same tx IDs and no duplicate journal/workbook records.
    - Test 3 (ING-03/ING-04, D-04): ingest persists `.rkyv` snapshot reference bound to each tx.
    - Test 4 (MCP-01): MCP `ingest_pdf(path)` response contains deterministic tx IDs from actual ingest execution.
    - Test 5 (MCP-05): MCP `get_raw_context(rkyv_ref)` returns stored bytes for ingested source reference.
  </behavior>
  <action>Author red tests only for remaining work; keep existing passing tests unchanged and avoid re-testing already-complete Phase 2 primitives.</action>
  <verify>
    <automated>cargo test -p ledger-core phase2_ingest_pipeline_remaining -- --nocapture ; cargo test -p turbo-mcp phase2_mcp_contract_remaining -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `rg -n "ING-01|ING-02|ING-03|ING-04" crates/ledger-core/tests/phase2_ingest_pipeline_remaining.rs` returns at least 4 matches.
    - `rg -n "MCP-01|MCP-05" crates/turbo-mcp/tests/phase2_mcp_contract_remaining.rs` returns at least 2 matches.
    - `cargo test -p ledger-core phase2_ingest_pipeline_remaining -- --nocapture` fails due to newly added RED-phase assertions.
    - `cargo test -p turbo-mcp phase2_mcp_contract_remaining -- --nocapture` fails due to newly added RED-phase assertions.
  </acceptance_criteria>
  <done>New tests fail for expected missing functionality and clearly describe required deterministic ingest behavior.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 2: Implement deterministic ingest pipeline completion (core + persistence)</name>
  <read_first>crates/ledger-core/src/ingest.rs, crates/ledger-core/src/journal.rs, crates/ledger-core/src/workbook.rs, crates/ledger-core/src/lib.rs, crates/ledger-core/tests/phase2_ingest_pipeline_remaining.rs, .planning/phases/02-deterministic-ingestion-pipeline/02-CONTEXT.md</read_first>
  <files>crates/ledger-core/Cargo.toml, crates/ledger-core/src/ingest.rs, crates/ledger-core/src/journal.rs, crates/ledger-core/src/workbook.rs, crates/ledger-core/src/lib.rs</files>
  <behavior>
    - Ingest path validates source filename before mutation (D-01).
    - Deterministic tx IDs remain content-hash based and stable (D-02).
    - Journal persistence stays rustledger-compatible Beancount with `txid` and `source_ref` metadata (D-05).
    - Ingest materializes records to target `TX.<account-id>` workbook projection for ING-01.
    - Parsed raw context snapshot is persisted as `.rkyv` sidecar and referenced per tx (ING-03/ING-04, D-04).
  </behavior>
  <action>Implement missing pipeline wiring only (no replacement of journal direction): add sidecar persistence and TX-sheet projection while preserving replay-safe dedupe semantics and panic-safe error handling.</action>
  <verify>
    <automated>cargo test -p ledger-core phase2_ingest -- --nocapture ; cargo test -p ledger-core phase2_rustledger_journal -- --nocapture ; cargo test -p ledger-core phase2_ingest_pipeline_remaining -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p ledger-core phase2_ingest -- --nocapture` passes.
    - `cargo test -p ledger-core phase2_rustledger_journal -- --nocapture` passes.
    - `cargo test -p ledger-core phase2_ingest_pipeline_remaining -- --nocapture` passes.
    - `rg -n "source_ref|rkyv" crates/ledger-core/src/ingest.rs crates/ledger-core/src/journal.rs crates/ledger-core/src/workbook.rs` returns references in all ingest surfaces.
  </acceptance_criteria>
  <done>All ledger-core ingest tests pass with deterministic/replay-safe behavior and explicit source evidence linkage.</done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Finalize MCP ingest/raw-context contract and run requirement-level verification</name>
  <read_first>crates/turbo-mcp/src/lib.rs, crates/turbo-mcp/tests/interface.rs, crates/turbo-mcp/tests/phase2_mcp_contract_remaining.rs, crates/ledger-core/src/ingest.rs, .planning/phases/02-deterministic-ingestion-pipeline/02-RESEARCH.md</read_first>
  <files>crates/turbo-mcp/src/lib.rs, crates/turbo-mcp/tests/phase2_mcp_contract_remaining.rs, crates/turbo-mcp/tests/interface.rs</files>
  <behavior>
    - MCP `ingest_pdf` contract executes deterministic ingest flow and returns deterministic tx IDs (MCP-01).
    - MCP `get_raw_context` resolves `.rkyv` references emitted by ingest output (MCP-05, D-04).
    - Tool-level contract remains explicit and replay-safe for repeated calls (D-03).
  </behavior>
  <action>Align request/response shapes to the completed ingest pipeline, keep adapters thin, and ensure no hidden mutation side effects in MCP handlers.</action>
  <verify>
    <automated>cargo test -p turbo-mcp -- --nocapture ; cargo test --workspace -- --nocapture</automated>
  </verify>
  <acceptance_criteria>
    - `cargo test -p turbo-mcp -- --nocapture` passes.
    - `cargo test --workspace -- --nocapture` passes.
    - `rg -n "ingest_pdf|get_raw_context" crates/turbo-mcp/src/lib.rs` confirms both MCP contracts are implemented in a thin adapter path.
    - `rg -n "MCP-01|MCP-05" crates/turbo-mcp/tests/phase2_mcp_contract_remaining.rs crates/turbo-mcp/tests/interface.rs` confirms requirement-tagged coverage.
  </acceptance_criteria>
  <done>All MCP and workspace tests pass; Phase 2 requirements ING-01/02/03/04 and MCP-01/05 are verifiably satisfied.</done>
</task>

</tasks>

<verification>
- Requirement mapping check: ING-01, ING-02, ING-03, ING-04, MCP-01, MCP-05 each have at least one passing automated test.
- Replay safety check: duplicate ingest attempts do not append duplicate journal entries or duplicate TX rows.
- Evidence traceability check: every ingested transaction includes `source_ref` that resolves with `get_raw_context`.
</verification>

<success_criteria>
1. `ingest_pdf(path)` performs deterministic ingest end-to-end with filename preflight and stable tx IDs.
2. Re-ingest is idempotent across both Beancount journal and workbook projection surfaces.
3. `.rkyv` evidence snapshots are persisted and retrievable through MCP `get_raw_context`.
4. Existing completed Phase 2 work remains passing (no regression in current deterministic ingest/journal contracts).
</success_criteria>

<output>
After completion, create `.planning/phases/02-deterministic-ingestion-pipeline/02-SUMMARY.md`.
</output>
