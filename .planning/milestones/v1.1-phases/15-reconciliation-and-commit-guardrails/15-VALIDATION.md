---
phase: 15
slug: reconciliation-and-commit-guardrails
status: complete
nyquist_compliant: true
wave_1_complete: true
wave_2_complete: true
created: 2026-03-29
---

# Phase 15 — Validation Strategy

> Per-phase validation contract for deterministic reconciliation and commit guardrail checks.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` integration tests |
| **Config file** | none (workspace defaults) |
| **Quick run command** | `cargo test -p turbo-mcp --test reconciliation_mcp_e2e -- --nocapture` |
| **Full suite command** | `cargo test -p turbo-mcp -- --nocapture` |
| **Estimated runtime** | ~10 seconds (quick) / ~45 seconds (full) |

---

## Sampling Rate

- **After every task commit:** Run task-specific `<automated>` command from plan.
- **After each plan completion:** Run phase contract suites (`reconciliation_contract`, `reconciliation_mcp_e2e`).
- **Before `$gsd-verify-work`:** Full `turbo-mcp` suite must be green.
- **Max feedback latency:** 45 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 15-01-01 | 01 | 1 | RECON-01, RECON-02 | integration | `cargo test -p turbo-mcp --test reconciliation_contract -- --nocapture` | ✅ | ✅ green |
| 15-01-02 | 01 | 1 | RECON-01, RECON-02 | integration | `cargo test -p turbo-mcp --test reconciliation_contract -- --nocapture` | ✅ | ✅ green |
| 15-01-03 | 01 | 1 | RECON-01, RECON-02 | integration | `cargo test -p turbo-mcp --test reconciliation_contract -- --nocapture && cargo test -p turbo-mcp --test interface -- --nocapture` | ✅ | ✅ green |
| 15-02-01 | 02 | 2 | RECON-03 | integration | `cargo test -p turbo-mcp --test reconciliation_mcp_e2e -- --nocapture` | ✅ | ✅ green |
| 15-02-02 | 02 | 2 | RECON-03 | integration | `cargo test -p turbo-mcp --test reconciliation_mcp_e2e -- --nocapture && cargo test -p turbo-mcp --test mcp_adapter_contract -- --nocapture` | ✅ | ✅ green |
| 15-02-03 | 02 | 2 | RECON-01, RECON-02, RECON-03 | integration | `cargo test -p turbo-mcp --test reconciliation_contract -- --nocapture && cargo test -p turbo-mcp --test reconciliation_mcp_e2e -- --nocapture && cargo test -p turbo-mcp -- --nocapture` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave Requirements

- [x] `crates/turbo-mcp/tests/reconciliation_contract.rs` — RECON-01/02 service guardrail contracts.
- [x] `crates/turbo-mcp/tests/reconciliation_mcp_e2e.rs` — RECON-03 MCP transport guardrail checks.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify commands.
- [x] Sampling continuity preserved across both plans.
- [x] No `MISSING` validation entries.
- [x] No watch-mode flags.
- [x] Feedback latency < 180s.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** complete
