# Project Milestones: tax-ledger

## v1.1 FDKMS Integrity (Shipped: 2026-03-29)

**Phases completed:** 12 phases, 16 plans, 47 tasks

**Key accomplishments:**

- Stdio MCP transport with proxy tool catalog, deterministic canonical/provenance mapping, and stable status/error contracts for agent-only runtime access.
- Subprocess MCP e2e coverage now proves ingest, canonical mapping, and replay idempotency through transport-only tool calls.
- Rustledger ingest-statement-rows proxy is now executable over MCP stdio tools/call with deterministic canonical and provenance payloads plus transport-proof tests.
- Ontology persistence and deterministic evidence-chain traversal implemented with Blake3 IDs, referential checks, and service-owned query primitives
- ONTO-03 MCP query/export surfaces now run over stdio transport with deterministic `nodes`/`edges` and `entities`/`edges`/`snapshot` payload contracts
- Service-level validate/reconcile/commit guardrails now deterministically block commit readiness on totals or balancing invariant failures with explicit machine-readable diagnostics
- Reconciliation validate/reconcile/commit guardrails are now callable over stdio MCP with deterministic blocked diagnostics and executable validation/runbook coverage
- Deterministic ingest-to-summarize lifecycle transitions with guarded blocked semantics and concise status hints
- Last-valid checkpoint resume flow with deterministic blocked reasons and invariant-preserving state behavior
- MCP transport wiring for deterministic HSM transition/status/resume with executable validation and verification artifacts
- Append-only lifecycle event persistence with deterministic payload/identity behavior across ingest, classify, reconcile, and adjust actions
- Deterministic event-stream reconstruction with explicit invariant diagnostics and tx/document replay filtering
- Deterministic MCP event replay/history transport with tx/document/time filters and executable phase validation
- Deterministic tax-assist service contracts now derive schedule/FBAR/ambiguity sections from reconciled ontology truth with explicit review-linked ambiguity payloads.
- Tax evidence-chain retrieval now produces deterministic `source -> events -> current_state` payloads with preserved provenance and explicit ambiguity linkage.
- Tax-assist, evidence-chain, and ambiguity-review surfaces are now callable end-to-end over MCP with deterministic concise envelopes and executable phase validation artifacts.

---

## v1.0 MVP (Shipped: 2026-03-29)

**Delivered:** Local-first, CPA-auditable tax-ledger MVP with deterministic ingest, runtime classification, audit trail, workbook outputs, and release automation.

**Phases completed:** 1-6 (6 plans total)

**Key accomplishments:**

- Contract-first ingest and deterministic transaction identity path completed.
- Runtime Rhai classification and review-flag queue delivered.
- Append-only audit mutation history and reconciliation flow implemented.
- CPA workbook export plus Schedule C/D/E and FBAR summaries exposed through MCP.
- CI, container-build checks, Cocogitto release workflow, and BDD e2e MVP flow added.

**Git range:** `4677e99` → `49b5a18`

**What's next:** Define v1.1 scope with `$gsd-new-milestone`.

---
