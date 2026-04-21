# MCP Surface

The MCP surface is the agent-facing contract for l3dg3rr. It is intentionally smaller than the internal Rust API: agents see a compact set of capability families, each selected with a required `action` argument.

The source of truth is `crates/ledgerr-mcp/src/contract.rs`. Generated operator docs live in `docs/mcp-capability-contract.md` and `docs/agent-mcp-runbook.md`; regenerate them with `cargo run -p xtask-mcpb -- generate-mcp-artifacts` after changing the published surface.

## Published Tool Families

| Tool | Capability family | Typical actions |
|---|---|---|
| `ledgerr_documents` | intake, filename validation, raw context, tags, filesystem metadata | `ingest_pdf`, `ingest_rows`, `document_inventory`, `normalize_filename` |
| `ledgerr_review` | rule execution, classification, review flags | `run_rule`, `classify_ingested`, `query_flags`, `classify_transaction` |
| `ledgerr_reconciliation` | totals and posting guardrails | `validate`, `reconcile`, `commit` |
| `ledgerr_workflow` | lifecycle and plugin operations | `status`, `transition`, `resume`, `plugin_info` |
| `ledgerr_audit` | event history and audit replay | `event_history`, `event_replay`, `query_audit_log` |
| `ledgerr_tax` | evidence, ambiguity review, workbook export | `assist`, `evidence_chain`, `ambiguity_review`, `export_workbook` |
| `ledgerr_ontology` | graph/ontology query and write operations | `query_path`, `export_snapshot`, `upsert_entities`, `upsert_edges` |
| `ledgerr_xero` | supervised Xero catalog and entity linkage | `get_auth_url`, `fetch_contacts`, `link_entity`, `sync_catalog` |

## Runtime Flow

```rhai
fn initialize() -> tools_list
fn tools_list() -> choose_capability
fn choose_capability() -> call_action
fn call_action() -> service_dispatch
fn service_dispatch() -> audit_event
if action == commit -> approval_gate
if action == export_workbook -> workbook_projection
```

## Layering

The transport adapter should not redefine business behavior. It parses the published shape, normalizes boundary variance, and dispatches to `TurboLedgerService`.

1. `ledgerr-mcp-server`: stdio transport.
2. `contract`: published tool families, actions, generated JSON Schema.
3. `mcp_adapter`: request parsing, envelope shaping, compatibility aliases.
4. `TurboLedgerService`: domain behavior, state, audit, lifecycle.
5. `ledger-core`: deterministic financial primitives.

## Compatibility Rule

Hidden legacy `l3dg3rr_*` and proxy names may continue to parse, but documentation and examples should use `ledgerr_*` only. Drift between `contract.rs` and generated docs is a test failure, not a manual documentation chore.

## Related Chapters

- [Capability Map](./capability-map.md)
- [Document Ingestion](./document-ingestion.md)
- [Workbook & Audit](./workbook-audit.md)
- [Xero Integration](./xero.md)
