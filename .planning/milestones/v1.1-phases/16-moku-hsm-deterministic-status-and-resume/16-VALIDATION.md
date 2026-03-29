---
phase: 16
slug: moku-hsm-deterministic-status-and-resume
status: complete
nyquist_compliant: true
wave_1_complete: true
wave_2_complete: true
wave_3_complete: true
created: 2026-03-29
---

# Phase 16 — Validation Strategy

> Per-phase validation contract for deterministic HSM lifecycle, guarded transitions, and resume transport behavior.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` integration tests |
| **Config file** | none (workspace defaults) |
| **Quick run command** | `cargo test -p turbo-mcp --test hsm_mcp_e2e -- --nocapture` |
| **Full suite command** | `cargo test -p turbo-mcp -- --nocapture` |
| **Estimated runtime** | ~12 seconds (quick) / ~55 seconds (full) |

---

## Sampling Rate

- **After every task commit:** Run task-specific `<automated>` command from plan.
- **After each plan completion:** Run phase contract suites (`hsm_contract`, `hsm_resume_contract`, `hsm_mcp_e2e`).
- **Before `$gsd-verify-work`:** Full `turbo-mcp` suite must be green.
- **Max feedback latency:** 60 seconds.

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 16-01-01 | 01 | 1 | HSM-01, HSM-02 | integration | `cargo test -p turbo-mcp --test hsm_contract -- --nocapture` | ✅ | ✅ green |
| 16-01-02 | 01 | 1 | HSM-01, HSM-02 | integration | `cargo test -p turbo-mcp --test hsm_contract -- --nocapture` | ✅ | ✅ green |
| 16-01-03 | 01 | 1 | HSM-01, HSM-02 | integration | `cargo test -p turbo-mcp --test hsm_contract -- --nocapture && cargo test -p turbo-mcp --test interface -- --nocapture` | ✅ | ✅ green |
| 16-02-01 | 02 | 2 | HSM-03 | integration | `cargo test -p turbo-mcp --test hsm_resume_contract -- --nocapture` | ✅ | ✅ green |
| 16-02-02 | 02 | 2 | HSM-03 | integration | `cargo test -p turbo-mcp --test hsm_resume_contract -- --nocapture` | ✅ | ✅ green |
| 16-02-03 | 02 | 2 | HSM-03 | integration | `cargo test -p turbo-mcp --test hsm_contract -- --nocapture && cargo test -p turbo-mcp --test hsm_resume_contract -- --nocapture` | ✅ | ✅ green |
| 16-03-01 | 03 | 3 | HSM-02, HSM-03 | integration | `cargo test -p turbo-mcp --test hsm_mcp_e2e -- --nocapture` | ✅ | ✅ green |
| 16-03-02 | 03 | 3 | HSM-02, HSM-03 | integration | `cargo test -p turbo-mcp --test hsm_mcp_e2e -- --nocapture && cargo test -p turbo-mcp --test mcp_adapter_contract -- --nocapture` | ✅ | ✅ green |
| 16-03-03 | 03 | 3 | HSM-01, HSM-02, HSM-03 | integration | `cargo test -p turbo-mcp --test hsm_contract -- --nocapture && cargo test -p turbo-mcp --test hsm_resume_contract -- --nocapture && cargo test -p turbo-mcp --test hsm_mcp_e2e -- --nocapture && cargo test -p turbo-mcp -- --nocapture` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave Requirements

- [x] `crates/turbo-mcp/tests/hsm_contract.rs` — HSM-01/02 service lifecycle + guard contracts.
- [x] `crates/turbo-mcp/tests/hsm_resume_contract.rs` — HSM-03 checkpoint/resume service contracts.
- [x] `crates/turbo-mcp/tests/hsm_mcp_e2e.rs` — HSM MCP transport deterministic contracts.

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify commands.
- [x] Sampling continuity preserved across all three plans.
- [x] No `MISSING` validation entries.
- [x] No watch-mode flags.
- [x] Feedback latency < 180s.
- [x] `nyquist_compliant: true` set in frontmatter.

**Approval:** complete
