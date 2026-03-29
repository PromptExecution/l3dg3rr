# Requirements: tax-ledger

**Defined:** 2026-03-29
**Core Value:** Convert raw historical financial PDFs into accountant-usable, auditable Excel tax records without sending private data to third-party SaaS.

## v1 Requirements

### Document Ingestion (Docling)

- [x] **DOC-01**: User can ingest statement PDFs through Docling/docling-mcp and produce normalized transaction candidates with per-field provenance.
- [x] **DOC-02**: User can map extracted fields to canonical transaction schema (`account`, `date`, `amount`, `description`, `currency`, `source_ref`) deterministically.
- [x] **DOC-03**: User can replay ingestion for the same source and receive stable candidate IDs with no duplicate candidates.

### Ontology & Knowledge Model

- [x] **ONTO-01**: User can persist ontology entities for document, account, institution, transaction, tax-category, and evidence reference.
- [x] **ONTO-02**: User can query ontology relationships (document -> extracted tx -> reconciliation state -> tax treatment).
- [x] **ONTO-03**: User can serialize ontology data in structured machine-readable form for AI agent consumption.

### Reconciliation & Verification

- [x] **RECON-01**: User can enforce double-entry balancing constraints before transactions become committed truth.
- [x] **RECON-02**: User can run automated reconciliation checks between source totals, extracted rows, and ledger postings.
- [x] **RECON-03**: User receives explicit blocking errors for invariant failures (imbalance, duplicate, schema mismatch).

### Hierarchical State Orchestration (Moku HSM)

- [x] **HSM-01**: User can run pipeline lifecycle as hierarchical states (ingest -> normalize -> validate -> reconcile -> commit -> summarize).
- [x] **HSM-02**: User can transition states only through validated guards and collect transition evidence.
- [x] **HSM-03**: User can resume interrupted pipelines from last valid state without violating invariants.

### Event-Sourced Audit Log (Disintegrate)

- [x] **EVT-01**: User can persist append-only domain events for ingestion, classification, reconciliation, and adjustment actions.
- [x] **EVT-02**: User can reconstruct entity state from disintegrate event streams.
- [x] **EVT-03**: User can query event history by transaction/document/time window for audit and agent explanation.

### US Expat Tax Agent Assist

- [x] **TAXA-01**: User can derive US expat tax-relevant structured outputs (Schedule C/D/E and FBAR evidence views) from reconciled ontology truth.
- [x] **TAXA-02**: AI agents can retrieve explainable evidence chains for tax decisions (source doc -> event log -> current state).
- [x] **TAXA-03**: User can flag scenarios with elevated tax ambiguity for human review with linked provenance.

## v2 Requirements

### Extended Intelligence

- **INTEL-01**: Probabilistic anomaly detection over historical ingestion/reconciliation behavior.
- **INTEL-02**: Automated agent-generated remediation proposals for detected reconciliation anomalies.

## Out of Scope

| Feature | Reason |
|---------|--------|
| Replacing CPA review with full autonomous filing | Human-in-the-loop accountant signoff remains mandatory |
| Multi-tenant cloud SaaS deployment | Current milestone remains local-first and operator-controlled |
| Non-tax personal finance dashboards as primary objective | Milestone scope is integrity and tax-assist knowledge workflows |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| DOC-01 | Phase 13 | Complete |
| DOC-02 | Phase 13 | Complete |
| DOC-03 | Phase 13 | Complete |
| ONTO-01 | Phase 14 | Complete |
| ONTO-02 | Phase 14 | Complete |
| ONTO-03 | Phase 14 | Complete |
| RECON-01 | Phase 15 | Complete |
| RECON-02 | Phase 15 | Complete |
| RECON-03 | Phase 15 | Complete |
| HSM-01 | Phase 16 | Complete |
| HSM-02 | Phase 16 | Complete |
| HSM-03 | Phase 16 | Complete |
| EVT-01 | Phase 17 | Complete |
| EVT-02 | Phase 17 | Complete |
| EVT-03 | Phase 17 | Complete |
| TAXA-01 | Phase 18 | Complete |
| TAXA-02 | Phase 18 | Complete |
| TAXA-03 | Phase 18 | Complete |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 18
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-29*
*Last updated: 2026-03-29 after v1.1 milestone gap-closure planning*
