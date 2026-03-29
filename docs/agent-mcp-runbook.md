# Agent MCP Runbook (Phases 13-16)

This runbook is MCP-only. Agent workflows must use MCP `initialize`, `notifications/initialized`, `tools/list`, and `tools/call` over stdio; no direct in-process service calls.

## Runtime Model

- `l3dg3rr` is the MCP boundary.
- Upstream capabilities are exposed through passthrough/proxy patterns.
- `proxy_docling_ingest_pdf` represents docling-style ingest orchestration.
- `proxy_rustledger_ingest_statement_rows` is callable over MCP `tools/call` as the rustledger-facing proxy surface.
- `l3dg3rr_ontology_query_path` exposes deterministic ontology path traversal.
- `l3dg3rr_ontology_export_snapshot` exposes deterministic ontology snapshot export.
- `l3dg3rr_validate_reconciliation` executes explicit validate-stage reconciliation checks.
- `l3dg3rr_reconcile_postings` executes explicit reconcile-stage totals checks.
- `l3dg3rr_commit_guarded` enforces commit-stage guardrails with deterministic blocking diagnostics.
- `l3dg3rr_hsm_transition` executes deterministic guarded lifecycle transitions.
- `l3dg3rr_hsm_status` returns concise deterministic lifecycle Display hints.
- `l3dg3rr_hsm_resume` resumes only from last valid checkpoint markers.

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
- `tools/list` includes `proxy_rustledger_ingest_statement_rows`.
- `tools/list` includes `l3dg3rr_ontology_query_path`.
- `tools/list` includes `l3dg3rr_ontology_export_snapshot`.
- Calls execute through MCP transport only.

To verify rustledger proxy callability specifically:

```bash
cargo test -p turbo-mcp --test mcp_stdio_e2e rustledger_proxy_ingest_statement_rows_over_transport -- --nocapture
```

Expected behavior:

- `tools/call` executes `proxy_rustledger_ingest_statement_rows` without unknown-tool fallback.
- Response includes deterministic `inserted_count` + stable `tx_ids` on replay.
- Canonical rows include provenance fields with `provider=rustledger` and `backend_tool=ingest_statement_rows`.

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

## Ontology Query + Export (ONTO-03)

Run:

```bash
cargo test -p turbo-mcp --test ontology_mcp_e2e -- --nocapture
```

Expected behavior:

- `tools/call` executes `l3dg3rr_ontology_query_path` and returns concise deterministic `nodes` and `edges`.
- `tools/call` executes `l3dg3rr_ontology_export_snapshot` and returns deterministic `entities`, `edges`, and `snapshot`.
- Repeating snapshot export with unchanged inputs yields byte-for-byte identical JSON serialization.

## Reconciliation Guardrails (RECON-01/02/03)

Run:

```bash
cargo test -p turbo-mcp --test reconciliation_contract -- --nocapture
cargo test -p turbo-mcp --test reconciliation_mcp_e2e -- --nocapture
```

Expected behavior:

- `tools/list` includes `l3dg3rr_validate_reconciliation`, `l3dg3rr_reconcile_postings`, and `l3dg3rr_commit_guarded`.
- `tools/call` on `l3dg3rr_commit_guarded` with imbalanced postings returns deterministic blocked payload fields:
  `isError=true`, `error_type=ReconciliationBlocked`, `stage=commit`, stable `blocked_reasons`.
- `tools/call` validate + reconcile + commit with matching totals and balanced postings yields deterministic ready payload:
  `isError=false`, `stage=commit`, `status=ready`, and stable `stage_marker`.

## HSM Lifecycle + Resume (HSM-01/02/03)

Run:

```bash
cargo test -p turbo-mcp --test hsm_contract -- --nocapture
cargo test -p turbo-mcp --test hsm_resume_contract -- --nocapture
cargo test -p turbo-mcp --test hsm_mcp_e2e -- --nocapture
```

Expected behavior:

- `tools/list` includes `l3dg3rr_hsm_transition`, `l3dg3rr_hsm_status`, and `l3dg3rr_hsm_resume`.
- Invalid transition over `tools/call` returns deterministic blocked payload with:
  `isError=true`, `error_type=HsmTransitionBlocked`, `guard_reason=invalid_transition`, stable `transition_evidence`.
- Invalid resume over `tools/call` returns deterministic blocked payload with:
  `isError=true`, `error_type=HsmResumeBlocked`, stable sorted `blockers`.
- Status and resume payloads include concise deterministic small-model hints:
  `display_state`, `next_hint`, `resume_hint`, and sorted `blockers`.

## Troubleshooting

- If MCP requests fail before tool calls: confirm lifecycle ordering (`initialize` before `tools/list` / `tools/call`).
- If ingest fails with `isError: true`: inspect request arguments for filename contract and row field shape.
- If replay is not idempotent: verify the same source payload (including account/date/amount/description/source_ref) is reused.
