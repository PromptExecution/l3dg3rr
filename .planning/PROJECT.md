# tax-ledger

## What This Is

tax-ledger is a local-first personal financial document intelligence system focused on retroactive U.S. expat tax preparation from raw PDF statements. It ingests statement PDFs, classifies transactions with agent-editable rules, and produces a CPA-auditable Excel workbook with Schedule-oriented outputs and full mutation history. It is built for an operator/agent workflow where AI performs ingestion, classification, reconciliation, and flagging while a human accountant reviews and signs off in Excel.

## Core Value

Convert raw historical financial PDFs into accountant-usable, auditable Excel tax records without sending private data to third-party SaaS.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Ingest renamed statement PDFs and write deterministic transaction rows into `TX.*` sheets in `tax-ledger.xlsx`
- [ ] Enforce strict monetary correctness with `rust_decimal` domain types and no floating-point money usage
- [ ] Apply Rhai classification rules from git-tracked rule files and persist category + confidence outputs
- [ ] Persist append-only audit history for all transaction mutations and classification changes
- [ ] Generate workbook schema with locked sheet names, table formatting, and Excel validation dropdowns from enum-backed taxonomy
- [ ] Produce Schedule C/D/E and FBAR-focused outputs from workbook data in CPA-reviewable form
- [ ] Expose stable MCP tool surface for ingest/classify/query/summary/context workflows
- [ ] Support containerized local deployment and reproducible versioned releases

### Out of Scope

- Cloud-first hosted accounting platform — local-first privacy and operator-controlled storage are required
- Replacing Excel as the primary accountant interface — CPA workflow requires workbook handoff and review in Excel
- General bookkeeping suite parity (e.g., full QuickBooks replacement) — focus is compliance-oriented document intelligence and auditability
- Automated PDF download from institutions — ingest assumes files already exist on disk with required naming convention

## Context

Primary use case is catching up three years of unfiled U.S. expat returns with CPA handoff constraints. The architecture is intentionally Excel-centric (`rust_xlsxwriter` write path + `calamine` read path), with AI-assisted classification through MCP tooling and human audit in spreadsheet form. Core technical choices from the brief include: `rkyv` document snapshots for rapid source-context retrieval, `blake3` content-addressed transaction IDs for idempotent re-ingest, `rhai` for runtime-editable classification logic, and HelixDB as a query projection over workbook truth. The design must maintain deterministic ingest, schema-bound validation, and a transparent audit trail suitable for accountant review.

## Constraints

- **Data Interface**: Excel workbook is the canonical human/audit layer — CPA workflow and signoff depend on it
- **Money Semantics**: `rust_decimal::Decimal` only for currency values — financial correctness and reproducibility
- **Identity Model**: Content-hash IDs only (Blake3 over account/date/amount/description) — idempotent ingest and dedup safety
- **Deployment Model**: Local-first single-user operation — no mandatory cloud services or ops-heavy infrastructure
- **Input Shape**: Source files must follow `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE` naming — deterministic ingest routing
- **Safety Bar**: No panic-prone pipeline code (`unwrap`, unchecked indexing) in financial paths — avoid silent data corruption and runtime faults

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Excel is the primary audit and handoff interface | Matches CPA workflows and practical auditability expectations | — Pending |
| `rust_xlsxwriter` + `calamine` for workbook roundtrip | Strong write features plus pure-Rust cross-platform read path | — Pending |
| `rkyv` snapshot sidecars for parsed document context | Fast, bounded-context retrieval for agent verification workflows | — Pending |
| Rhai scripts for classification/flag logic | Runtime-editable, diffable rules without recompilation | — Pending |
| HelixDB is projection over workbook truth | Relationship queries without displacing accountant-facing source of truth | — Pending |
| Local-first architecture (no Postgres/cloud dependency) | Privacy and low-ops constraints from use case | — Pending |
| MCP wrapper is first-class integration contract | Agent operation depends on stable callable tool semantics | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `$gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `$gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check -> still the right priority?
3. Audit Out of Scope -> reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-28 after initialization*
