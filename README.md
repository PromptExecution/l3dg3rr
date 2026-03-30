# tax-ledger (Autonomous MVP build)

[![CI](https://github.com/PromptExecution/l3dg3rr/actions/workflows/ci.yml/badge.svg)](https://github.com/PromptExecution/l3dg3rr/actions/workflows/ci.yml)
[![Release](https://github.com/PromptExecution/l3dg3rr/actions/workflows/release.yml/badge.svg)](https://github.com/PromptExecution/l3dg3rr/actions/workflows/release.yml)

Rust workspace for a local-first, Excel-first tax ledger system.

## Agent Guide

See [AGENTS.md](AGENTS.md) for agent-facing purpose, capability boundaries, operating expectations, and persistent session-learning rules.
See [docs/mcp-capability-contract.md](docs/mcp-capability-contract.md) for the concrete MCP tool matrix, API relationships, and contrived end-to-end usage.

## Current scope

- Contract-first filename preflight (`VENDOR--ACCOUNT--YYYY-MM--DOCTYPE`)
- Session manifest parsing and account listing
- Workbook initialization with required sheet names
- Git-friendly plain-text ingest output via Beancount journal entries (rustledger-compatible)
- Idiomatic turbo MCP interface surface for `list_accounts` and `ingest_statement_rows`

## Quickstart

```bash
cargo test
```

## Docker

```bash
docker build -t tax-ledger:dev .
docker run --rm \
  -v "$PWD/data:/data" \
  -v "$PWD/rules:/rules" \
  -v "$PWD/tax-years:/tax-years" \
  tax-ledger:dev
```

## Versioning (Cocogitto)

```bash
./scripts/install-hooks.sh
cog check
cog changelog
cog bump --auto
```

## Behavior-Driven MVP E2E

```bash
./scripts/e2e_mvp.sh
```

This validates the full ingest -> classify -> audit -> schedule summary flow.

## Claude Cowork Plugin Marketplace

- Approach and operator workflow: `docs/claude-cowork-plugin-marketplace.md`
- Marketplace catalog: `.claude-plugin/marketplace.json`
- MCP runtime helpers: `Justfile` (`just mcp-start`, `just mcp-stop`, `just mcp-e2e`)

## CI/CD Publish Targets

- Workflow: `.github/workflows/publish.yml`
- Trigger: GitHub Release `published` (or manual `workflow_dispatch`)
- Targets:
  - GHCR image: `ghcr.io/promptexecution/l3dg3rr`
  - crates.io crates: `ledger-core`, `turbo-mcp` (requires `CRATES_IO_TOKEN`)
  - PyPI package: `l3dg3rr-mcp-launcher` (requires `PYPI_API_TOKEN`)
