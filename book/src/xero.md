# Xero Integration

Xero is a supervised accounting integration, not a raw credential surface for models. Agents should interact through `ledgerr_xero` actions and supervised worker processes while l3dg3rr owns credentials, approvals, audit, and process supervision.

## Capability Boundary

The current MCP family is `ledgerr_xero`. It exposes catalog and linkage operations:

- authorization URL generation and code exchange
- contacts, accounts, bank accounts, and invoice fetches
- contact search
- local entity linking
- catalog synchronization

```rhai
fn xero_auth() -> supervised_token_store
fn supervised_token_store() -> fetch_catalog
fn fetch_catalog() -> link_entities
fn link_entities() -> reconciliation_candidates
fn reconciliation_candidates() -> operator_review
```

## Credential Model

Long-lived secrets should be mediated by the host credential abstraction, with Windows Credential Manager as the first practical backend. `.env` is acceptable for local bootstrap and tests, but it is not the target long-term secret model.

## Reconciliation Use

Xero data should enrich local evidence rather than replace local-first records:

- local statements and workbook remain primary for tax preparation
- Xero contacts/accounts provide counterparties and entity hints
- reconciliation gates compare extracted totals, local postings, and remote accounting facts
- link decisions should be audit-visible and reversible where practical

## Related Chapters

- [MCP Surface](./mcp-surface.md)
- [Reconciliation & Ledger Operations](./ledger-ops.md)
- [Workbook & Audit](./workbook-audit.md)
