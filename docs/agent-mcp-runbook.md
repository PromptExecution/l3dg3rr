# Agent MCP Runbook (Phase 13)

This runbook is MCP-only. Agent workflows must use MCP `initialize`, `notifications/initialized`, `tools/list`, and `tools/call` over stdio; no direct in-process service calls.

## Runtime Model

- `l3dg3rr` is the MCP boundary.
- Upstream capabilities are exposed through passthrough/proxy patterns.
- `proxy_docling_ingest_pdf` represents docling-style ingest orchestration.
- `proxy_rustledger_ingest_statement_rows` remains the rustledger-facing proxy surface.

## Bootstrap

From repo root:

```bash
cargo build -p turbo-mcp --bin turbo-mcp-server
```

## MCP Lifecycle

The required lifecycle order is:

1. `initialize`
2. `notifications/initialized`
3. `tools/list`
4. `tools/call`

The e2e suite in `crates/turbo-mcp/tests/mcp_stdio_e2e.rs` enforces this order.

## Tool Discovery

Run:

```bash
cargo test -p turbo-mcp --test mcp_stdio_e2e doc_01_mcp_only_ingest_via_tools_call -- --nocapture
```

Expected behavior:

- `tools/list` includes `proxy_docling_ingest_pdf`.
- Calls execute through MCP transport only.

## Deterministic Mapping + Replay

Run full MCP e2e:

```bash
bash scripts/mcp_e2e.sh
```

Expected behavior:

- DOC-02: canonical fields are present in response rows:
  `account`, `date`, `amount`, `description`, `currency`, `source_ref`.
- Provenance fields are present:
  `provider`, `backend_tool`, `backend_version`, `backend_call_id`.
- DOC-03: replaying identical ingest input returns stable `tx_ids` and `inserted_count` transitions from `1` to `0`.

## Troubleshooting

- If MCP requests fail before tool calls: confirm lifecycle ordering (`initialize` before `tools/list` / `tools/call`).
- If ingest fails with `isError: true`: inspect request arguments for filename contract and row field shape.
- If replay is not idempotent: verify the same source payload (including account/date/amount/description/source_ref) is reused.
