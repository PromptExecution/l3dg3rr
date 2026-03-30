---
description: Bootstrap and validate l3dg3rr MCP usage in Claude Cowork Plugin Create workflows
---

Use this skill when setting up or troubleshooting `l3dg3rr` in Claude Cowork Plugin Create.

## Install and activate

1. Add marketplace:
   - `/plugin marketplace add https://github.com/PromptExecution/l3dg3rr`
2. Install plugin:
   - `/plugin install l3dg3rr-plugin-create@promptexecution-fdkms`
3. Validate install:
   - `/plugin list`
   - `/plugin show l3dg3rr-plugin-create`

## MCP runtime choices

- Cargo (default in plugin manifest):
  - `cargo run -p turbo-mcp --bin turbo-mcp-server`
- Prebuilt binary:
  - `./target/release/turbo-mcp-server`
- Docker:
  - `docker run -i --rm -v "$PWD:/workspace" -w /workspace tax-ledger:dev cargo run -p turbo-mcp --bin turbo-mcp-server`
- Python launcher (local package in plugin):
  - `python -m l3dg3rr_mcp_launcher --mode cargo`

## First-call validation

1. `tools/list` and confirm at least one `l3dg3rr_*` tool appears.
2. `tools/call l3dg3rr_context_summary {}`.
3. If available, run one domain call such as `l3dg3rr_ontology_export_snapshot`.

## Start/stop behavior

- In Cowork, MCP process lifecycle is managed by Claude.
- Manual start is foreground stdio (`cargo run ...`); stop with `Ctrl+C`.
- Optional helper commands (repo root):
  - `just mcp-start`
  - `just mcp-stop`

## Troubleshooting checklist

- Confirm `cargo --version` or selected runtime binary exists.
- Confirm working directory is repository root.
- Confirm manifest path: `plugins/l3dg3rr-plugin-create/.claude-plugin/plugin.json`.
- Run MCP regression: `./scripts/mcp_e2e.sh`.

## Quality bar

- Keep outputs deterministic, concise, and machine-readable.
- Do not bypass l3dg3rr capability boundaries with ad-hoc file parsing.
- Preserve explicit blocked diagnostics for permission or invariant failures.
