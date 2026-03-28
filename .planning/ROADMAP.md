# Roadmap: tax-ledger

## Milestones

- ✅ **v1.0 MVP** — Phases 1-6 shipped 2026-03-29 ([archive](./milestones/v1.0-ROADMAP.md))
- 🚧 **v1.1 FDKMS Integrity** — Phases 7-12 (in progress)

## Overview

Milestone v1.1 evolves l3dg3rr into a financial document management knowledge system (FDKMS) optimized for AI-assisted US expat tax workflows with ontology-structured truth, strict reconciliation guarantees, and event-sourced auditability.

## Phases

- [ ] **Phase 7: Docling Statement Ingestion Canonicalization** - Integrate docling/docling-mcp ingestion into deterministic canonical transaction candidates.
- [ ] **Phase 8: Ontological Knowledge Graph Layer** - Persist/query ontology entities and relations for documents, transactions, and evidence chains.
- [ ] **Phase 9: Double-Entry Reconciliation Gates** - Enforce balancing and reconciliation checks before transaction truth commitment.
- [ ] **Phase 10: Moku Hierarchical State Machine Orchestration** - Implement guarded hierarchical pipeline state transitions and resumability.
- [ ] **Phase 11: Disintegrate Event-Sourced Audit Backbone** - Persist and replay domain events for full lifecycle reconstruction and audit queries.
- [ ] **Phase 12: US Expat Tax Agent Assist Surfaces** - Expose structured tax-assist outputs and explainable evidence retrieval for agents.

## Phase Details

### Phase 7: Docling Statement Ingestion Canonicalization
**Goal**: Ingest statements via docling/docling-mcp into canonical transaction candidates with deterministic IDs and provenance.
**Depends on**: Phase 6
**Requirements**: DOC-01, DOC-02, DOC-03
**Success Criteria**:
  1. Statement ingestion via docling/docling-mcp yields canonical candidate rows with provenance metadata.
  2. Canonical field mapping is deterministic and stable across re-runs.
  3. Replaying same source produces no duplicate candidates.

### Phase 8: Ontological Knowledge Graph Layer
**Goal**: Build ontology entities/relations for document-to-transaction-to-tax semantics and machine-readable query.
**Depends on**: Phase 7
**Requirements**: ONTO-01, ONTO-02, ONTO-03
**Success Criteria**:
  1. Core ontology entities and relations persist with referential integrity.
  2. Relationship queries cover full evidence chains.
  3. Ontology data is serializable for AI agent workflows.

### Phase 9: Double-Entry Reconciliation Gates
**Goal**: Ensure no transaction commits without passing double-entry and reconciliation validations.
**Depends on**: Phase 8
**Requirements**: RECON-01, RECON-02, RECON-03
**Success Criteria**:
  1. Double-entry constraints block imbalanced commits.
  2. Reconciliation checks validate source totals vs committed postings.
  3. Invariant failures return explicit blocking diagnostics.

### Phase 10: Moku Hierarchical State Machine Orchestration
**Goal**: Represent pipeline lifecycle with moku HSM and enforce guarded/resumable transitions.
**Depends on**: Phase 9
**Requirements**: HSM-01, HSM-02, HSM-03
**Success Criteria**:
  1. Pipeline states and substates are encoded in moku HSM.
  2. Invalid transitions are blocked by guard checks.
  3. Interrupted runs resume from valid prior state.

### Phase 11: Disintegrate Event-Sourced Audit Backbone
**Goal**: Persist/replay lifecycle events using disintegrate for reconstructable domain truth.
**Depends on**: Phase 10
**Requirements**: EVT-01, EVT-02, EVT-03
**Success Criteria**:
  1. Domain events are append-only and typed across lifecycle operations.
  2. Entity state reconstruction from event streams is deterministic.
  3. Audit history queries support transaction/document/time slicing.

### Phase 12: US Expat Tax Agent Assist Surfaces
**Goal**: Deliver ontology-backed, reconciled, explainable outputs for US expat tax AI agent workflows.
**Depends on**: Phase 11
**Requirements**: TAXA-01, TAXA-02, TAXA-03
**Success Criteria**:
  1. Schedule/FBAR tax-assist outputs derive from reconciled ontology truth.
  2. Agents can retrieve explainable evidence chains for tax reasoning.
  3. Ambiguous scenarios are flagged for human review with linked provenance.

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 7. Docling Statement Ingestion Canonicalization | 0/TBD | Not started | - |
| 8. Ontological Knowledge Graph Layer | 0/TBD | Not started | - |
| 9. Double-Entry Reconciliation Gates | 0/TBD | Not started | - |
| 10. Moku Hierarchical State Machine Orchestration | 0/TBD | Not started | - |
| 11. Disintegrate Event-Sourced Audit Backbone | 0/TBD | Not started | - |
| 12. US Expat Tax Agent Assist Surfaces | 0/TBD | Not started | - |

## Backlog

- Phase 999.1: CI + Release Automation Hardening (deferred from prior cycle; can be promoted if needed)

