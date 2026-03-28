# tax-ledger (Autonomous MVP build)

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
docker run --rm tax-ledger:dev
```

## Versioning (Cocogitto)

```bash
cog check
cog changelog
```
