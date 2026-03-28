# Requirements: tax-ledger

**Defined:** 2026-03-28
**Core Value:** Convert raw historical financial PDFs into accountant-usable, auditable Excel tax records without sending private data to third-party SaaS.

## v1 Requirements

### Contracts

- [ ] **CONT-01**: User can initialize a workbook with locked required sheets (`META.config`, `ACCT.registry`, `CAT.taxonomy`, `SCHED.C`, `SCHED.D`, `SCHED.E`, `FBAR.accounts`, `FLAGS.open`, `FLAGS.resolved`, `AUDIT.log`)
- [ ] **CONT-02**: User can configure session context from `manifest.toml` without loading full workbook state
- [ ] **CONT-03**: User can ingest only files that match `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE` naming convention
- [ ] **CONT-04**: User can reject malformed input with clear validation errors before any ledger mutation

### Ingestion

- [ ] **ING-01**: User can ingest a renamed statement PDF from disk and materialize transaction rows into the corresponding `TX.<account-id>` sheet
- [ ] **ING-02**: User can re-ingest the same statement without duplicate transactions (idempotent behavior)
- [ ] **ING-03**: User can persist parsed document context as a `.rkyv` snapshot alongside the source PDF
- [ ] **ING-04**: User can trace each ingested transaction back to its source document reference

### Classification

- [ ] **CLSF-01**: User can load classification rules from `rules/classify.rhai` at runtime without recompiling
- [ ] **CLSF-02**: User can assign tax category and confidence score to each transaction using rule execution
- [ ] **CLSF-03**: User can flag transactions requiring review when confidence or policy thresholds fail
- [ ] **CLSF-04**: User can test candidate Rhai rules against sample transactions before applying changes

### Audit and Safety

- [ ] **AUD-01**: User can get append-only audit entries for every transaction mutation with timestamp, actor, field, old value, and new value
- [ ] **AUD-02**: User can edit transaction classifications in Excel and reconcile those edits back into the service audit trail
- [ ] **AUD-03**: User can rely on decimal-safe arithmetic for all money operations (no float-backed currency values)
- [ ] **AUD-04**: User can detect and report invariant violations for amount parsing, hash determinism, and schema conformance

### Workbook UX

- [ ] **WB-01**: User can open workbook transaction tables with Excel Table formatting for filter/sort and pivot compatibility
- [ ] **WB-02**: User can pick categories from Excel dropdown validation backed by taxonomy enum values
- [ ] **WB-03**: User can view unresolved and resolved flags in dedicated sheets without manual filtering across all transaction sheets

### Tax Outputs

- [ ] **TAX-01**: User can generate Schedule C summary values from categorized transaction data
- [ ] **TAX-02**: User can generate Schedule D summary values including crypto/bad-debt categories where tagged
- [ ] **TAX-03**: User can generate Schedule E summary values for rental-related categories
- [ ] **TAX-04**: User can generate FBAR account/year maximum USD balance views for accountant review

### MCP Interface

- [ ] **MCP-01**: User can call `ingest_pdf(path)` through MCP and receive deterministic transaction IDs
- [ ] **MCP-02**: User can call `classify_transaction(tx_id, category, confidence, note)` through MCP and record audit output
- [ ] **MCP-03**: User can call `query_flags(year, status)` through MCP and retrieve actionable review queue data
- [ ] **MCP-04**: User can call `get_schedule_summary(year, schedule)` through MCP for CPA-facing summaries
- [ ] **MCP-05**: User can call `get_raw_context(rkyv_ref)` through MCP for source evidence lookup
- [ ] **MCP-06**: User can call `list_accounts()` through MCP to enumerate configured account definitions
- [ ] **MCP-07**: User can call `run_rhai_rule(rule_file, sample_tx)` through MCP for rule validation workflows

### Packaging and Release

- [ ] **REL-01**: User can run the system as a local Docker container with mounted workbook/rules/tax-year directories
- [ ] **REL-02**: User can produce versioned releases and changelogs with Cocogitto-based workflow
- [ ] **REL-03**: User can run a behavior-driven end-to-end MVP test flow that validates ingest, classify, audit, and schedule outputs

## v2 Requirements

### Graph and Analysis

- **GPH-01**: User can project workbook truth into HelixDB and run relationship traversals for money-flow analysis
- **GPH-02**: User can execute parity checks ensuring projection answers match workbook source state
- **GPH-03**: User can fall back to alternative graph backend without changing domain logic

### API and UI

- **API-01**: User can access ledger operations through Axum HTTP API in addition to MCP
- **API-02**: User can review flags, rule tests, and schedule summaries through Leptos dashboard
- **API-03**: User can use Arrow IPC/DataFusion exports for advanced analytics queries

### Automation Extensions

- **AUT-01**: User can automate financial document retrieval via external browser/download system integration
- **AUT-02**: User can support additional compliance domains beyond US-expat tax with schema variants

## Out of Scope

| Feature | Reason |
|---------|--------|
| Cloud-hosted multi-tenant SaaS | Conflicts with local-first privacy and single-user operator model |
| Postgres as primary data store | Adds unnecessary ops complexity and diverges from Excel-first accountant workflow |
| Full bookkeeping platform replacement | Not required to deliver CPA handoff for retroactive tax compliance |
| Tax filing/e-file submission engine | Current scope is data preparation and auditability, not final filing submission |
| Fully autonomous no-human-review classification | Violates human-in-audit-seat requirement for accountant trust |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CONT-01 | Phase TBD | Pending |
| ING-01 | Phase TBD | Pending |
| CLSF-01 | Phase TBD | Pending |
| AUD-01 | Phase TBD | Pending |
| WB-01 | Phase TBD | Pending |
| TAX-01 | Phase TBD | Pending |
| MCP-01 | Phase TBD | Pending |
| REL-01 | Phase TBD | Pending |

**Coverage:**
- v1 requirements: 33 total
- Mapped to phases: 0
- Unmapped: 33 ⚠

---
*Requirements defined: 2026-03-28*
*Last updated: 2026-03-28 after initial definition*
