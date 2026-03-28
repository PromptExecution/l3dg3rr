# Project Research Summary

**Project:** tax-ledger
**Domain:** Local-first AI-assisted tax ledger and CPA handoff workflow
**Researched:** 2026-03-28
**Confidence:** HIGH

## Executive Summary

Tax-ledger is a local-first financial document intelligence product for turning retroactive PDF statements into CPA-ready Excel workbooks with an auditable change trail. The research converges on a strict architecture where Excel is the human source of truth, `.rkyv` sidecars provide fast machine context recall, and any graph/database layer is a derived projection only. This aligns with the PRD constraint that accountant handoff quality matters more than web UX sophistication.

The recommended implementation path is Rust-first with deterministic ingest and classification: `rust_decimal` for money, BLAKE3 content-hash IDs for idempotency, `rust_xlsxwriter` + `calamine` for roundtrip workbook control, and Rhai for runtime-editable rules. Expert pattern consensus is to lock schema and MCP contracts early, then deliver a usable ingest/classify/audit MVP before introducing HelixDB projection and optional API/UI layers.

The dominant risks are correctness and audit integrity rather than raw throughput: decimal drift, transaction ID instability, workbook contract drift, non-replayable audit logs, and human-edit race conditions. Mitigation is well-defined: invariant tests first, append-only/hash-chained audit rows, schema validators, optimistic concurrency + single-writer queue, and release discipline via cocogitto.

## Key Findings

### Recommended Stack

The stack is coherent and purpose-built for tax-grade correctness over convenience abstractions. Keep canonical storage in `tax-ledger.xlsx`; avoid cloud DBs and float money paths. Use Docker multi-stage + `cargo-chef` for reproducible local deployment and cocogitto for auditable release/versioning.

**Core technologies:**
- Rust + Tokio: core runtime and async orchestration for local ingest/classify pipelines.
- `rust_xlsxwriter` + `calamine`: write/read Excel contract without COM dependency.
- `rust_decimal`: mandatory money type to prevent float drift.
- `blake3`: deterministic `TxId` generation for idempotent re-ingest.
- Rhai: runtime-editable classification and flag rules with git-visible diffs.
- `rkyv` (+ `bytecheck`): fast, validated source-context sidecars.
- RMCP Rust SDK: stable MCP server implementation for agent tooling.
- HelixDB (phase 2+) with fallback (`heed` + `petgraph`): graph projection only, never truth.

### Expected Features

**Must have (table stakes):**
- Deterministic statement ingest with source linkage per transaction.
- Human review workflow with explicit confidence/flag states.
- Rule-based auto-categorization with deterministic re-run.
- Append-only audit history for classification/manual edits.
- Excel-native handoff workbook with schedule-oriented summaries (C/D/E + FBAR fields).

**Should have (competitive):**
- Local-first processing and storage by default.
- Agent-editable Rhai rules and explainable classification rationale.
- Fixed CPA-ready workbook contract (stable sheets/tables/dropdowns).

**Defer (v2+):**
- Full bookkeeping suite scope (invoicing/payroll/AP/AR).
- E-file/direct tax filing.
- Mandatory bank credential sync/aggregation.
- Multi-user collaboration workflow orchestration.
- Rich dashboard-first UX replacing Excel.

### Architecture Approach

Use a layered architecture centered on `ledger-core` invariants and a `ledger-service` orchestration boundary. `ledger-io` is the only mutation path for workbook truth; `ledger-rules` is deterministic compute; `ledger-audit` records append-only diffs; `ledger-projection` is rebuildable; `ledger-mcp`/API/UI are thin adapters.

**Major components:**
1. `ledger-core` — domain types, invariants, and traits (`ExcelRecord`, `Auditable`, `GraphNode`).
2. `ledger-io` — workbook schema enforcement, ingest normalization, and `.rkyv` persistence.
3. `ledger-rules` — Rhai classification/flag execution with versioned rule metadata.
4. `ledger-audit` — append-only, replayable mutation log with actor/session context.
5. `ledger-service` — use-case orchestration and canonical write ordering.
6. `ledger-projection` — rebuildable graph read-model synced from workbook rows.
7. `ledger-mcp` — versioned tool contract for AI agent operations.

### Critical Pitfalls

1. **Decimal drift at Excel boundaries** — enforce a single money parser and ban float conversions.
2. **Non-deterministic `TxId`** — freeze canonical hash input contract and test normalization variants.
3. **Workbook contract drift** — validate required sheets/tables/named ranges at startup and fail fast.
4. **Non-replayable audit trail** — append-only writes with monotonic sequence + hash chaining.
5. **Human edit race conditions** — optimistic concurrency (`etag`/row version), debounced file-watch diff, single writer queue.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 0: Contract Lock + Input Preflight
**Rationale:** Most downstream failures come from schema/tool drift and bad input naming, so lock contracts first.
**Delivers:** Workbook schema validator/initializer, `manifest.toml` schema, MCP tool contract v1, preflight filename scanner.
**Addresses:** Deterministic ingest readiness, stable CPA handoff contract.
**Avoids:** Workbook contract drift, MCP interface drift, ingest ambiguity.

### Phase 1: Core Ingest/Classify/Audit MVP
**Rationale:** This is the smallest end-to-end value path and validates product thesis.
**Delivers:** PDF -> `.rkyv` snapshot -> `TX.*` rows with deterministic IDs, Rhai classification + flags, `AUDIT.log`, schedule summaries, key MCP tools.
**Addresses:** Table-stakes ingest/review/categorization/audit + Excel handoff.
**Uses:** Rust, `rust_decimal`, `blake3`, `rust_xlsxwriter`, `calamine`, Rhai, RMCP.
**Avoids:** Decimal drift, ID nondeterminism, missing substantiation trail.

### Phase 2: Reconciliation Hardening + Release Discipline
**Rationale:** Human-in-the-loop Excel edits are guaranteed; harden for real operator behavior before adding major new surfaces.
**Delivers:** Debounced file watcher, merge-aware diff reconciliation, conflict handling, invariant/property tests, Dockerized build/run, cocogitto release flow.
**Addresses:** Trust/stability requirements for CPA workflow.
**Avoids:** Lost edits, stale overwrites, non-reproducible builds/releases.

### Phase 3: Graph Projection and Advanced Queries
**Rationale:** Relationship analysis is valuable but should follow stable canonical core.
**Delivers:** HelixDB-backed projection with full rebuild/parity checks, graph query MCP tools, fallback abstraction readiness.
**Addresses:** Cross-account flow analysis and complex traceability.
**Avoids:** Projection-as-truth drift via startup rebuild + consistency checks.

### Phase 4: Optional API/UI and Analytics Convenience
**Rationale:** Browser layer is a convenience, not delivery-critical for CPA handoff.
**Delivers:** Axum API, optional dashboard for flags/rules, Arrow/DataFusion analytics export.
**Addresses:** Operator ergonomics and richer reporting.
**Avoids:** Premature UI scope blocking correctness-critical pipeline delivery.

### Phase Ordering Rationale

- Dependencies are strict: contract/schema -> deterministic core -> reconciliation hardening -> projection -> optional interfaces.
- This ordering ships a usable CPA handoff system by early phases and isolates higher-volatility components.
- Risks are front-loaded into testable invariants before feature expansion.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 3:** HelixDB operational maturity, drift detection strategy, and fallback trigger criteria.
- **Phase 4:** UI ergonomics for rule editing and safe concurrency semantics across API/UI and Excel edits.
- **Phase 1 (targeted):** Tax evidence completeness rules for basis, FX conversion, and FBAR max-value provenance.

Phases with standard patterns (can usually skip deep research-phase):
- **Phase 0:** Workbook schema/MCP contract definition is mostly internal decision discipline.
- **Phase 2:** File watching, optimistic concurrency, Docker multi-stage, and cocogitto are established patterns.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Strong official crate/docs coverage and clear fit to PRD constraints. |
| Features | HIGH | Table-stakes/differentiators align with competitor expectations and first-party scope. |
| Architecture | HIGH | Layering, write-ordering, and projection boundaries are explicit and internally consistent. |
| Pitfalls | HIGH | Concrete failure modes with prevention/detection controls and phase mapping. |

**Overall confidence:** HIGH

### Gaps to Address

- **MCP tool schema details:** finalize exact request/response/error contracts and versioning policy before implementation starts.
- **`manifest.toml` contract:** define required keys, lifecycle updates, and drift handling with workbook metadata.
- **Tax evidence completeness checks:** formalize minimum required provenance fields for Schedule D and FBAR readiness.
- **Helix fallback policy:** define objective thresholds for staying on Helix vs switching to `heed`/`petgraph`.

## Sources

### Primary (HIGH confidence)
- `/home/brianh/promptexecution/mbse/l3dg3rr/prd.md` — architecture constraints and phased intent.
- `/home/brianh/promptexecution/mbse/l3dg3rr/.planning/research/STACK.md` — recommended technologies and version guidance.
- `/home/brianh/promptexecution/mbse/l3dg3rr/.planning/research/FEATURES.md` — table stakes, differentiators, anti-features.
- `/home/brianh/promptexecution/mbse/l3dg3rr/.planning/research/ARCHITECTURE.md` — boundaries, patterns, build sequence.
- `/home/brianh/promptexecution/mbse/l3dg3rr/.planning/research/PITFALLS.md` — critical risks, mitigations, and phase warnings.

### Secondary (MEDIUM confidence)
- Official crate docs: `rust_xlsxwriter`, `calamine`, `rust_decimal`, `rkyv`, `rhai`, `axum`, `tokio`, `datafusion`.
- MCP and HelixDB documentation for adapter/projection patterns.
- Cocogitto and `cargo-chef` docs for release/container workflows.

### Tertiary (LOW confidence)
- Vendor/product pages used in feature landscape benchmarking (marketing-heavy, directional only).

---
*Research completed: 2026-03-28*
*Ready for roadmap: yes*
