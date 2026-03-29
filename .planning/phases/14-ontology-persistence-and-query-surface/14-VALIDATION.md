---
phase: 14
slug: ontology-persistence-and-query-surface
status: draft
nyquist_compliant: false
wave_0_complete: false
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
| **Quick run command** | `cargo test -p turbo-mcp --test ontology_contract onto_01_persistence_integrity -- --nocapture` |
| **Full suite command** | `cargo test --workspace -- --nocapture` |
| **Estimated runtime** | ~25 seconds (quick) / ~150 seconds (full) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p turbo-mcp --test ontology_contract onto_01_persistence_integrity -- --nocapture`
- **After every plan wave:** Run `cargo test -p turbo-mcp -- --nocapture`
- **Before `$gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 14-01-01 | 01 | 1 | ONTO-01 | integration | `cargo test -p turbo-mcp --test ontology_contract onto_01_persistence_integrity -- --nocapture` | ❌ W0 | ⬜ pending |
| 14-01-02 | 01 | 1 | ONTO-02 | integration | `cargo test -p turbo-mcp --test ontology_contract onto_02_relationship_query -- --nocapture` | ❌ W0 | ⬜ pending |
| 14-01-03 | 01 | 2 | ONTO-03 | integration | `cargo test -p turbo-mcp --test ontology_mcp_e2e onto_03_export_snapshot_stable -- --nocapture` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `crates/turbo-mcp/tests/ontology_contract.rs` — ONTO-01/02 contract verification
- [ ] `crates/turbo-mcp/tests/ontology_mcp_e2e.rs` — ONTO-03 MCP transport serialization checks

---

## Manual-Only Verifications

All phase behaviors have automated verification.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 180s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
