# Requirements: tax-ledger

**Defined:** 2026-03-29
**Core Value:** Convert raw historical financial PDFs into accountant-usable, auditable Excel tax records without sending private data to third-party SaaS.

## v1 Requirements

### Document Ingestion (Docling)

- [ ] **DOC-01**: User can ingest statement PDFs through Docling/docling-mcp and produce normalized transaction candidates with per-field provenance.
- [ ] **DOC-02**: User can map extracted fields to canonical transaction schema (`account`, `date`, `amount`, `description`, `currency`, `source_ref`) deterministically.
- [ ] **DOC-03**: User can replay ingestion for the same source and receive stable candidate IDs with no duplicate candidates.

### Ontology & Knowledge Model

- [ ] **ONTO-01**: User can persist ontology entities for document, account, institution, transaction, tax-category, and evidence reference.
- [ ] **ONTO-02**: User can query ontology relationships (document -> extracted tx -> reconciliation state -> tax treatment).
- [ ] **ONTO-03**: User can serialize ontology data in structured machine-readable form for AI agent consumption.

### Reconciliation & Verification

- [ ] **RECON-01**: User can enforce double-entry balancing constraints before transactions become committed truth.
- [ ] **RECON-02**: User can run automated reconciliation checks between source totals, extracted rows, and ledger postings.
- [ ] **RECON-03**: User receives explicit blocking errors for invariant failures (imbalance, duplicate, schema mismatch).

### Hierarchical State Orchestration (Moku HSM)

- [ ] **HSM-01**: User can run pipeline lifecycle as hierarchical states (ingest -> normalize -> validate -> reconcile -> commit -> summarize).
- [ ] **HSM-02**: User can transition states only through validated guards and collect transition evidence.
- [ ] **HSM-03**: User can resume interrupted pipelines from last valid state without violating invariants.

### Event-Sourced Audit Log (Disintegrate)

- [ ] **EVT-01**: User can persist append-only domain events for ingestion, classification, reconciliation, and adjustment actions.
- [ ] **EVT-02**: User can reconstruct entity state from disintegrate event streams.
- [ ] **EVT-03**: User can query event history by transaction/document/time window for audit and agent explanation.

### US Expat Tax Agent Assist

- [ ] **TAXA-01**: User can derive US expat tax-relevant structured outputs (Schedule C/D/E and FBAR evidence views) from reconciled ontology truth.
- [ ] **TAXA-02**: AI agents can retrieve explainable evidence chains for tax decisions (source doc -> event log -> current state).
- [ ] **TAXA-03**: User can flag scenarios with elevated tax ambiguity for human review with linked provenance.

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
| DOC-01 | Phase 13 | Pending |
| DOC-02 | Phase 13 | Pending |
| DOC-03 | Phase 13 | Pending |
| ONTO-01 | Phase 14 | Pending |
| ONTO-02 | Phase 14 | Pending |
| ONTO-03 | Phase 14 | Pending |
| RECON-01 | Phase 15 | Pending |
| RECON-02 | Phase 15 | Pending |
| RECON-03 | Phase 15 | Pending |
| HSM-01 | Phase 16 | Pending |
| HSM-02 | Phase 16 | Pending |
| HSM-03 | Phase 16 | Pending |
| EVT-01 | Phase 17 | Pending |
| EVT-02 | Phase 17 | Pending |
| EVT-03 | Phase 17 | Pending |
| TAXA-01 | Phase 18 | Pending |
| TAXA-02 | Phase 18 | Pending |
| TAXA-03 | Phase 18 | Pending |

**Coverage:**
- v1 requirements: 18 total
- Mapped to phases: 18
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-29*
*Last updated: 2026-03-29 after v1.1 milestone gap-closure planning*
