# tax-ledger (Autonomous MVP build)

[![CI](https://github.com/PromptExecution/l3dg3rr/actions/workflows/ci.yml/badge.svg)](https://github.com/PromptExecution/l3dg3rr/actions/workflows/ci.yml)
[![Release](https://github.com/PromptExecution/l3dg3rr/actions/workflows/release.yml/badge.svg)](https://github.com/PromptExecution/l3dg3rr/actions/workflows/release.yml)

Rust workspace for a local-first, Excel-first tax ledger system.

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
