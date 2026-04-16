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

## Prerequisites

| Tool | Install | Purpose |
|------|---------|---------|
| Rust 1.88+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | Build toolchain |
| [just](https://github.com/casey/just) | `cargo install just` | Task runner (all `just *` recipes) |
| [cocogitto](https://docs.cocogitto.io/) | `cargo install cocogitto` | Conventional commits + version bumps |
| [cross](https://github.com/cross-rs/cross) | `cargo install cross --locked` | Cross-compilation for musl/macOS release bundles |
| [mcp-publisher](https://github.com/modelcontextprotocol/registry) | See registry releases | MCP Registry submission (`just publish-registry`) |

Optional: Docker or Podman for container builds.

## Quickstart

```bash
# Run the full test suite
cargo test --workspace --all-features

# Or via just (also runs mcp-outcome-test)
just test

# Start the MCP server (stdio transport)
just mcp-start
```

## Install in Claude Code

Download the pre-built binary for your platform from the [latest GitHub Release](https://github.com/PromptExecution/l3dg3rr/releases/latest), then register it with Claude Code:

### macOS / Linux

```bash
# Replace VERSION with the latest release tag, e.g. v0.1.1
VERSION=v1.2.1

# macOS (Apple Silicon)
curl -fsSL "https://github.com/PromptExecution/l3dg3rr/releases/download/${VERSION}/ledgerr-mcp-aarch64-apple-darwin.mcpb" -o /tmp/ledgerr-mcp.mcpb

# macOS (Intel)
curl -fsSL "https://github.com/PromptExecution/l3dg3rr/releases/download/${VERSION}/ledgerr-mcp-x86_64-apple-darwin.mcpb" -o /tmp/ledgerr-mcp.mcpb

# Linux (x86_64)
curl -fsSL "https://github.com/PromptExecution/l3dg3rr/releases/download/${VERSION}/ledgerr-mcp-x86_64-unknown-linux-musl.mcpb" -o /tmp/ledgerr-mcp.mcpb

# Register with Claude Code
claude mcp add ledgerr /tmp/ledgerr-mcp.mcpb
```

### Windows (PowerShell)

```powershell
$v = "v1.2.1"   # replace with latest
Invoke-WebRequest `
  "https://github.com/PromptExecution/l3dg3rr/releases/download/$v/ledgerr-mcp-x86_64-pc-windows-msvc.mcpb" `
  -OutFile "$env:TEMP\ledgerr-mcp.mcpb"
claude mcp add ledgerr "$env:TEMP\ledgerr-mcp.mcpb"
```

After adding, restart Claude Code. The `l3dg3rr_*` tools will appear automatically.

## Docker

The container runs the `ledgerr-mcp-server` binary (stdio MCP transport).
Mount `/data` for the workbook and PDF inbox.

```bash
docker build -t tax-ledger:dev .
docker run --rm -i \
  -v "$PWD/data:/data" \
  tax-ledger:dev
```

## Release

```bash
just release           # patch release (default)
just release major   # major release
just release minor  # minor release

# Or trigger via GitHub: Actions > Release > Run workflow
```

Uses cocogitto for conventional commit versioning. Releases trigger on CI success or manual dispatch.

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
  - crates.io crates: `ledger-core`, `ledgerr-mcp` (requires `CRATES_IO_TOKEN`)
  - PyPI package: `l3dg3rr-mcp-launcher` (requires `PYPI_API_TOKEN`)
