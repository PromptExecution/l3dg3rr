# Agent MCP Runbook (Generated)

This file is generated from `crates/ledgerr-mcp/src/contract.rs`.

Agent workflows must use `initialize`, `notifications/initialized`, `tools/list`, and `tools/call` over stdio.

## Runtime Model

The default published surface is the reduced 7-tool catalog:

- `ledgerr_documents`
- `ledgerr_review`
- `ledgerr_reconciliation`
- `ledgerr_workflow`
- `ledgerr_audit`
- `ledgerr_tax`
- `ledgerr_ontology`
- `ledgerr_xero`

Each tool requires an `action` argument.

## Bootstrap

From repo root:

```bash
cargo build -p ledgerr-mcp --bin ledgerr-mcp-server
```

## Lifecycle

Required order:

1. `initialize`
2. `notifications/initialized`
3. `tools/list`
4. `tools/call`

## Basic Happy Path

```json
{"arguments":{"action":"pipeline_status"},"name":"ledgerr_documents"}
{"arguments":{"action":"list_accounts"},"name":"ledgerr_documents"}
{"arguments":{"action":"ingest_pdf","extracted_rows":[{"account_id":"WF-BH-CHK","amount":"-42.11","date":"2023-01-15","description":"Coffee Shop","source_ref":"wf-2023-01.rkyv"}],"journal_path":"/tmp/demo.beancount","pdf_path":"WF--BH-CHK--2023-01--statement.pdf","raw_context_bytes":[99,116,120],"workbook_path":"/tmp/demo.xlsx"},"name":"ledgerr_documents"}
{"arguments":{"action":"get_raw_context","rkyv_ref":"wf-2023-01.rkyv"},"name":"ledgerr_documents"}
```

## Troubleshooting / Spinning Wheels

```json
{"arguments":{"action":"resume","state_marker":"invalid-checkpoint"},"name":"ledgerr_workflow"}
{"arguments":{"action":"commit","extracted_total":"95.00","posting_amounts":["-95.00","95.00"],"source_total":"100.00"},"name":"ledgerr_reconciliation"}
{"arguments":{"action":"event_history","time_end":"2026-01-01","time_start":"2026-12-31"},"name":"ledgerr_audit"}
```

Expected blocked outcomes:

- invalid workflow resume returns `HsmResumeBlocked`
- imbalanced reconciliation commit returns `ReconciliationBlocked`
- invalid audit time range returns `EventHistoryBlocked`

## Suggested Test Commands

```bash
cargo test -p ledgerr-mcp --test mcp_stdio_e2e -- --nocapture
cargo test -p ledgerr-mcp --test plugin_info_mcp_e2e -- --nocapture
bash scripts/mcp_cli_demo.sh
bash scripts/mcp_e2e.sh
```

## Notes

- Hidden compatibility aliases still exist for older `l3dg3rr_*` and proxy-style calls, but agents should not depend on them.
- Use `docs/mcp-capability-contract.md` as the concise surface map.
