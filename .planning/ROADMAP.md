# Roadmap: tax-ledger

## Overview

This roadmap delivers a local-first, Excel-first tax ledger by locking contracts first, then shipping deterministic ingest/classification/audit flows, then packaging the system for repeatable local operation and release.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

- [x] **Phase 1: Contracts & Session Bootstrap** - Lock workbook/session contracts and input preflight before ledger mutation.
- [x] **Phase 2: Deterministic Ingestion Pipeline** - Convert renamed PDFs into idempotent transaction rows with source context linkage.
- [x] **Phase 3: Rule-Driven Classification & Flagging** - Apply/test Rhai rules and produce actionable review queues.
- [x] **Phase 4: Audit Integrity & Safe Reconciliation** - Enforce append-only mutation history and decimal-safe invariants.
- [ ] **Phase 5: CPA Workbook Outputs** - Deliver Excel-native usability plus Schedule/FBAR summaries.
- [ ] **Phase 6: Local Deployment & Release Readiness** - Containerize, version, and validate the full MVP behavior end to end.

## Phase Details

### Phase 1: Contracts & Session Bootstrap
**Goal**: Users can start from a locked workbook/session contract and only ingest files that pass naming and validation gates.
**Depends on**: Nothing (first phase)
**Requirements**: CONT-01, CONT-02, CONT-03, CONT-04, MCP-06
**Success Criteria** (what must be TRUE):
  1. User can initialize a workbook with all required locked sheet names present and ready for downstream flows.
  2. User can load session context from `manifest.toml` and list configured accounts without opening full workbook state.
  3. User can submit only files matching the required naming convention, and malformed names are rejected before any data mutation.
**Plans**: TBD

### Phase 2: Deterministic Ingestion Pipeline
**Goal**: Users can ingest statement PDFs into deterministic workbook transactions with replay-safe IDs and source evidence retrieval.
**Depends on**: Phase 1
**Requirements**: ING-01, ING-02, ING-03, ING-04, MCP-01, MCP-05
**Success Criteria** (what must be TRUE):
  1. User can ingest a renamed statement PDF and get transaction rows in the correct `TX.<account-id>` sheet.
  2. User can re-run ingest on the same statement and receive the same transaction IDs with no duplicate rows.
  3. User can retrieve source document context from the `.rkyv` reference for any ingested transaction.
**Plans**: TBD

### Phase 3: Rule-Driven Classification & Flagging
**Goal**: Users can classify transactions from runtime-editable rules and maintain a reliable queue of records needing review.
**Depends on**: Phase 2
**Requirements**: CLSF-01, CLSF-02, CLSF-03, CLSF-04, MCP-03, MCP-07
**Success Criteria** (what must be TRUE):
  1. User can load and run Rhai classification rules at runtime, including testing candidate rules on sample transactions.
  2. User can assign category and confidence per transaction through rule execution without recompiling the service.
  3. User can produce and query unresolved/review flags by year and status through MCP.
**Plans**: TBD

### Phase 4: Audit Integrity & Safe Reconciliation
**Goal**: Users can trust every transaction/classification mutation is auditable, replayable, and protected by strict money/invariant safety checks.
**Depends on**: Phase 3
**Requirements**: AUD-01, AUD-02, AUD-03, AUD-04, MCP-02
**Success Criteria** (what must be TRUE):
  1. User can see append-only audit entries for each mutation with timestamp, actor, field change, and old/new values.
  2. User can update classifications in Excel and reconcile those edits back into the service with matching audit records.
  3. User can rely on decimal-safe money operations and receive explicit errors when invariant checks fail.
**Plans**: TBD

### Phase 5: CPA Workbook Outputs
**Goal**: Users can hand accountants a workbook with usable transaction tables, controlled category entry, review flag sheets, and Schedule/FBAR summaries.
**Depends on**: Phase 4
**Requirements**: WB-01, WB-02, WB-03, TAX-01, TAX-02, TAX-03, TAX-04, MCP-04
**Success Criteria** (what must be TRUE):
  1. User can work with transaction tables using Excel-native filter/sort/pivot behavior.
  2. User can select valid categories from workbook validation dropdowns backed by the taxonomy contract.
  3. User can access unresolved and resolved flags in dedicated sheets.
  4. User can generate and retrieve Schedule C, Schedule D, Schedule E, and FBAR summary outputs for CPA review.
**Plans**: TBD

### Phase 6: Local Deployment & Release Readiness
**Goal**: Users can run, verify, and version the MVP consistently as a local containerized product.
**Depends on**: Phase 5
**Requirements**: REL-01, REL-02, REL-03
**Success Criteria** (what must be TRUE):
  1. User can run the system locally in Docker with mounted workbook, rules, and tax-year directories.
  2. User can create versioned releases and changelogs through the Cocogitto workflow.
  3. User can execute an end-to-end MVP test that validates ingest, classify, audit, and schedule output behavior.
**Plans**: TBD

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Contracts & Session Bootstrap | 1/1 | Complete | 2026-03-28 |
| 2. Deterministic Ingestion Pipeline | 1/1 | Complete | 2026-03-29 |
| 3. Rule-Driven Classification & Flagging | 1/1 | Complete | 2026-03-29 |
| 4. Audit Integrity & Safe Reconciliation | 1/1 | Complete | 2026-03-29 |
| 5. CPA Workbook Outputs | 0/TBD | Not started | - |
| 6. Local Deployment & Release Readiness | 0/TBD | Not started | - |

## Backlog

### Phase 999.1: CI + Release Automation Hardening (BACKLOG)

**Goal:** Add robust CI coverage and public status badges (including container build), enforce conventional commits with hooks, and implement a streamlined cocogitto-driven release workflow gated by passing CI. Include web research/examples before implementation to avoid bespoke patterns.
**Requirements:** TBD
**Plans:** 0 plans

Plans:
- [ ] TBD (promote with `$gsd-review-backlog` when ready)
