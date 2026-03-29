---
created: 2026-03-29T07:00:02.026Z
title: Add Claude Cowork MCP install matrix and CI gate
area: docs
files:
  - docs/claude-cowork-mcp-install.md
  - .github/workflows/ci.yml
  - .planning/STATE.md
---

## Problem

UAT consumers need concise, copy/paste-ready installation guidance for Claude Cowork plugin marketplace usage of the MCP server. Current guidance is not standardized across runtimes (docker, cargo, uvx, wasi), lacks a single canonical operator doc, and has no explicit CI merge gate tied to validating install flow clarity.

## Solution

Create a canonical install matrix document at `docs/claude-cowork-mcp-install.md` with minimal prose and explicit sections for prerequisites, marketplace add/install steps, runtime-specific `mcpServers` configs, verification commands, and troubleshooting quick table. Add discoverability metadata stating support for four runtimes, a minimal tools/list + tools/call validation path, and concise operator-format instructions. Add a release note line for this addition and gate merge on CI pass so install docs stay verifiable.
