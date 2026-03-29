---
phase: 13
slug: mcp-boundary-and-agent-only-runtime-surface
status: draft
nyquist_compliant: false
wave_0_complete: false
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
| 13-01-01 | 01 | 0 | DOC-01 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_01_mcp_only_ingest -- --nocapture` | ❌ W0 | ⬜ pending |
| 13-01-02 | 01 | 0 | DOC-02 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_02_canonical_mapping -- --nocapture` | ❌ W0 | ⬜ pending |
| 13-01-03 | 01 | 0 | DOC-03 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_03_replay_idempotent -- --nocapture` | ❌ W0 | ⬜ pending |
| 13-01-04 | 01 | 1 | DOC-01 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture` | ✅ | ⬜ pending |
| 13-01-05 | 01 | 1 | DOC-02 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture` | ✅ | ⬜ pending |
| 13-01-06 | 01 | 2 | DOC-03 | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` — MCP runtime entrypoint
- [ ] `crates/turbo-mcp/src/mcp_adapter.rs` — tool handler/schema mapping layer
- [ ] `crates/turbo-mcp/tests/mcp_stdio_e2e.rs` — subprocess MCP-only phase tests

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| MCP initialization sequence visible and deterministic for runbook users | DOC-01 | Protocol ordering/readability review in docs output | Start server, run `initialize`, send `notifications/initialized`, run `tools/list`; verify docs exactly match expected order and fields |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
