---
phase: 14
slug: ontology-persistence-and-query-surface
status: complete
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-29
---

# Phase 14 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `cargo test` integration tests |
| **Config file** | none (workspace defaults) |
| **Quick run command** | `cargo test -p turbo-mcp --test ontology_mcp_e2e onto_03_export_snapshot_stable_json_serialization_over_transport -- --nocapture` |
| **Full suite command** | `cargo test -p turbo-mcp -- --nocapture` |
| **Estimated runtime** | ~10 seconds (quick) / ~30 seconds (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p turbo-mcp --test ontology_mcp_e2e -- --nocapture`
- **After every plan wave:** Run `cargo test -p turbo-mcp -- --nocapture`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 14-02-01 | 02 | 2 | ONTO-03 | integration | `cargo test -p turbo-mcp --test ontology_mcp_e2e -- --nocapture` | ✅ | ✅ green |
| 14-02-02 | 02 | 2 | ONTO-03 | integration | `cargo test -p turbo-mcp --test ontology_mcp_e2e -- --nocapture && cargo test -p turbo-mcp --test mcp_adapter_contract -- --nocapture` | ✅ | ✅ green |
| 14-02-03 | 02 | 2 | ONTO-03 | integration | `cargo test -p turbo-mcp --test ontology_contract -- --nocapture && cargo test -p turbo-mcp --test ontology_mcp_e2e -- --nocapture && cargo test -p turbo-mcp -- --nocapture` | ✅ | ✅ green |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [x] `crates/turbo-mcp/tests/ontology_contract.rs` — ONTO-01/02 contract verification
- [x] `crates/turbo-mcp/tests/ontology_mcp_e2e.rs` — ONTO-03 MCP transport serialization checks

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 180s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** complete
