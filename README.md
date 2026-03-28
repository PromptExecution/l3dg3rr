# tax-ledger (Phase 1 bootstrap)

Rust workspace for a local-first, Excel-first tax ledger system.

## Current scope

- Contract-first filename preflight (`VENDOR--ACCOUNT--YYYY-MM--DOCTYPE`)
- Session manifest parsing and account listing
- Workbook initialization with required sheet names
- Idiomatic turbo MCP interface surface for `list_accounts`

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
