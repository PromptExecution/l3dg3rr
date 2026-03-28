# Feature Landscape

**Domain:** Local-first AI-assisted financial document intelligence for tax prep handoff to CPA in Excel  
**Researched:** 2026-03-28

## Table Stakes

Features users expect for this category. Missing these makes the product non-credible for real tax prep.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Deterministic statement ingest (PDF/image to transactions) | Mainstream tools already accept statement uploads and extract transaction rows | Med | Must preserve idempotency via content-hash IDs |
| Human-reviewable extraction UI/workflow | Competitors position extraction as draft data that users review and correct | Med | Include explicit "needs review" states per transaction |
| Rule-based auto-categorization | QuickBooks bank rules and similar systems have trained users to expect automatic categorization | Med | v1 needs simple editable rules + deterministic re-run |
| Reconciliation-ready transaction output | Users expect downstream matching/reconciliation after ingest and categorization | Med | Keep statuses in-sheet so CPA can track completion |
| Source-document linkage per transaction | Supporting documents are required for substantiation and audit readiness | Low | Each row needs stable source pointer to original PDF/snapshot |
| Append-only change history for classification edits | Tax workflow requires explainability for who changed what and when | Med | Include actor, timestamp, old/new value, reason note |
| Excel-native handoff package | Tax/accounting ecosystems still export to spreadsheets and rely on workbook review flows | Med | Produce one workbook that opens cleanly without macros/add-ins |
| Tax-year and schedule-oriented views (C/D/E + FBAR support fields) | CPA handoff needs tax-form-aligned summarization, not just raw transaction dumps | High | v1 can start with pivots/summary sheets rather than full form automation |

## Differentiators

Features that create clear separation for this v1 versus cloud accounting/document-capture tools.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Local-first processing and storage by default | Avoids cloud upload/privacy tradeoffs and external reviewer exposure | Med | Core product thesis; no mandatory SaaS dependency |
| Agent-editable classification and flag rules (Rhai) | Faster adaptation to edge-case merchants/tax logic without redeploy | Med | Treat rule files as auditable artifacts |
| Sidecar raw-context snapshots (`.rkyv`) tied to transaction rows | CPA/agent can instantly verify extracted text without re-parsing PDFs | Med | Enables high-trust audit and rapid dispute resolution |
| Explainable confidence + flag queue instead of silent auto-post | Prioritizes accountant trust over "black-box autopilot" | Med | Confidence and rationale columns should be first-class |
| CPA-ready workbook contract (fixed sheets, validation dropdowns, structured tables) | Reduces handoff friction because output is already in accountant-native review format | Med | Schema stability is a product feature, not an implementation detail |

## Anti-Features

Features to explicitly NOT build in v1.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Full bookkeeping suite parity (invoicing, payroll, AP/AR ops) | Expands scope into QuickBooks/Xero replacement and kills delivery velocity | Stay focused on ingest -> classify -> audit -> CPA handoff |
| Direct tax return filing (e-file) | High regulatory/compliance burden and distracts from preparation quality | Export CPA-ready workbook and let CPA’s tax stack file returns |
| Mandatory bank credential aggregation/sync | Adds security/compliance complexity and conflicts with local-first + retroactive PDF workflow | Keep file-based ingest from user-controlled statements |
| Fully autonomous classification with no review gate | Increases risk of silent misclassification and erodes trust | Require human/audit checkpoints and unresolved-flag queue |
| Multi-user collaboration, roles, and workflow orchestration | Premature for single-operator local-first v1 | Single-user workflow with explicit audit trail |
| Replacing Excel with bespoke web UI as primary interface | Violates core handoff constraint for CPA workflows | Keep web UI optional convenience layer only |

## Feature Dependencies

```text
File naming convention + ingest parser -> deterministic Tx row model
Deterministic Tx row model -> rule-based categorization
Rule-based categorization -> confidence + flag generation
Source linkage + audit log -> CPA trust and substantiation
Validated TX sheets -> Schedule/FBAR summary outputs
Stable workbook schema -> repeatable CPA handoff
```

## MVP Recommendation

Prioritize:
1. Deterministic PDF/image ingest into `TX.*` with source linkage and idempotent IDs
2. Rule-based categorization with confidence, manual override, and append-only audit log
3. CPA handoff workbook contract (validated taxonomy dropdowns + schedule summary sheets for C/D/E and FBAR fields)

Defer: Full graph analytics and rich browser dashboard UI: valuable, but not required to make the CPA handoff usable in v1.

## Confidence Notes

- **HIGH:** Table-stakes around ingest, extraction review, categorization rules, and reconciliation expectations (confirmed across QuickBooks/Xero/Dext docs)
- **HIGH:** Substantiation and record-retention need for source documents and auditable records (IRS Publication 583)
- **MEDIUM:** Excel-centric handoff as practical CPA review surface (strongly supported by CCH Axcess export patterns and project constraints)
- **HIGH:** Anti-feature exclusions tied to explicit project scope constraints in `prd.md` and `.planning/PROJECT.md`

## Sources

- Project brief: [prd.md](/home/brianh/promptexecution/mbse/l3dg3rr/prd.md) (HIGH, first-party)
- Project context: [PROJECT.md](/home/brianh/promptexecution/mbse/l3dg3rr/.planning/PROJECT.md) (HIGH, first-party)
- Intuit QuickBooks Help, "Manually upload transactions into QuickBooks Online" (updated 2026-03-27): https://quickbooks.intuit.com/learn-support/en-us/help-article/import-transactions/manually-upload-transactions-quickbooks-online/L0rE9OXBz_US_en_US (HIGH)
- Xero, "Data capture (Hubdoc)" (AU page, crawled current): https://www.xero.com/au/accounting-software/capture-data-with-hubdoc/ (MEDIUM, marketing page but product-specific details)
- Dext Help, "Using Line Item Extraction" (updated this week): https://help.dext.com/en/articles/377044-using-line-item-extraction (HIGH)
- IRS Publication 583, "Starting a Business and Keeping Records" (rev. 12/2024, IRS page reviewed 2026-02-11): https://www.irs.gov/publications/p583 (HIGH)
- CCH Axcess Tax Help, "Exporting Data from Worksheet Grids" (published 2026-02): https://download.cchaxcess.com/PfxBrowserHelp/TaxHelp/Content/ImportExport/IE_Exporting%20Data%20from%20Worksheet%20Grids.htm (MEDIUM)
