---
phase: 13
slug: mcp-boundary-and-agent-only-runtime-surface
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-29
---

# Phase 13 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` integration tests |
| **Config file** | none (workspace defaults) |
| **Quick run command** | `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture` |
| **Full suite command** | `cargo test --workspace -- --nocapture` |
| **Estimated runtime** | ~120 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture`
- **After every plan wave:** Run `cargo test -p turbo-mcp -- --nocapture`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 180 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 13-02-01 | 02 | 0 | DOC-01 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_01_mcp_only_ingest_via_tools_call -- --nocapture` | ✅ | ✅ green |
| 13-02-02 | 02 | 0 | DOC-02 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_02_canonical_mapping_and_provenance_fields_over_transport -- --nocapture` | ✅ | ✅ green |
| 13-02-03 | 02 | 0 | DOC-03 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_03_replay_idempotent_with_stable_tx_ids_over_mcp -- --nocapture` | ✅ | ✅ green |
| 13-02-04 | 02 | 1 | DOC-01/02/03 | integration | `bash scripts/mcp_e2e.sh` | ✅ | ✅ green |
| 13-02-05 | 02 | 1 | DOC-01/02/03 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture` | ✅ | ✅ green |
| 13-03-01 | 03 | 3 | DOC-01/02/03 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e rustledger_proxy_ingest_statement_rows_over_transport -- --nocapture` | ✅ | ✅ green |
| 13-03-02 | 03 | 3 | DOC-01/02/03 | integration | `cargo test -p turbo-mcp --test mcp_adapter_contract -- --nocapture` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` — MCP runtime entrypoint
- [x] `crates/turbo-mcp/src/mcp_adapter.rs` — tool handler/schema mapping layer
- [x] `crates/turbo-mcp/tests/mcp_stdio_e2e.rs` — subprocess MCP-only phase tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| MCP initialization sequence visible and deterministic for runbook users | DOC-01 | Protocol ordering/readability review in docs output | Run `bash scripts/mcp_e2e.sh`; confirm test harness performs `initialize`, `notifications/initialized`, `tools/list`, and `tools/call` in order |
| Rustledger proxy callable shape remains operator-readable in MCP-only docs | DOC-01/02/03 | Validate runbook instructions match executable transport command names | Run `cargo test -p turbo-mcp --test mcp_stdio_e2e rustledger_proxy_ingest_statement_rows_over_transport -- --nocapture`; confirm docs reference this exact command and tool name |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 180s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** ready for verify-work
