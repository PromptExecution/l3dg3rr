---
phase: 18-tax-assist-evidence-chain-interfaces
plan: 01
subsystem: api
tags: [tax-assist, ontology, reconciliation, deterministic-payloads]
requires:
  - phase: 17-disintegrate-event-sourced-lifecycle-backbone
    provides: deterministic event replay and MCP boundary patterns
provides:
  - deterministic tax-assist service contracts
  - reconciled ontology-gated schedule/fbar outputs
  - ambiguity payload with explicit review-state provenance links
affects: [taxa-02-evidence-chain, taxa-03-mcp-transport]
tech-stack:
  added: []
  patterns: [reconciliation-gated tax output, deterministic sectioned payloads]
key-files:
  created:
    - crates/turbo-mcp/src/tax_assist.rs
    - crates/turbo-mcp/tests/tax_assist_contract.rs
  modified:
    - crates/turbo-mcp/src/lib.rs
key-decisions:
  - "Tax-assist output remains blocked until reconciliation stage passes."
  - "Ambiguity records use explicit review_state/reason plus provenance_refs."
patterns-established:
  - "Service output shape: status/stage_marker/blocked_reasons + concise section vectors."
requirements-completed: [TAXA-01, TAXA-03]
duration: 6min
completed: 2026-03-29
---

# Phase 18 Plan 01: Tax Assist Service Surface Summary

**Deterministic tax-assist service contracts now derive schedule/FBAR/ambiguity sections from reconciled ontology truth with explicit review-linked ambiguity payloads.**

## Performance

- **Duration:** 6 min
- **Started:** 2026-03-29T08:34:41Z
- **Completed:** 2026-03-29T08:40:00Z
- **Tasks:** 3
- **Files modified:** 3

## Accomplishments
- Added additive tax-assist domain contracts and service tool signatures.
- Locked TAXA-01/TAXA-03 behavior with RED tests before implementation.
- Implemented deterministic schedule/fbar extraction and ambiguity records from ontology edges behind reconciliation gating.

## Task Commits

1. **Task 1: Define tax-assist interface contracts for service composition** - `749adf0` (feat)
2. **Task 2: Add RED TAXA-01/TAXA-03 service contract tests** - `ca1f8b6` (test)
3. **Task 3: Implement deterministic tax-assist and ambiguity service logic** - `75b6b97` (feat)

## Files Created/Modified
- `crates/turbo-mcp/src/tax_assist.rs` - Tax-assist/evidence-chain domain contract types and deterministic builders.
- `crates/turbo-mcp/src/lib.rs` - Service composition methods for tax assist, chain retrieval, and ambiguity review.
- `crates/turbo-mcp/tests/tax_assist_contract.rs` - TAXA-01/TAXA-03 contract tests.

## Decisions Made
- Keep tax-assist surfaces additive and l3dg3rr-owned without passthrough contract breakage.
- Normalize ambiguity to stable `needs_review` + `ambiguous_tax_treatment` vocabulary for downstream review queue consistency.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed partial-move compile error in taxonomy builder map**
- **Found during:** Task 3
- **Issue:** Node id extraction moved `String` before node reuse.
- **Fix:** Clone id when creating node lookup tuple.
- **Files modified:** `crates/turbo-mcp/src/tax_assist.rs`
- **Verification:** `cargo test -p turbo-mcp --test tax_assist_contract -- --nocapture`
- **Committed in:** `75b6b97`

**2. [Rule 1 - Bug] Preserved all direct ontology source edges for tax derivation**
- **Found during:** Task 3
- **Issue:** BFS path traversal omitted secondary edges to already-visited nodes, dropping schedule rows.
- **Fix:** Supplemented path result with deterministic direct source-edge merge from ontology store.
- **Files modified:** `crates/turbo-mcp/src/lib.rs`
- **Verification:** `cargo test -p turbo-mcp --test tax_assist_contract -- --nocapture`
- **Committed in:** `75b6b97`

---

**Total deviations:** 2 auto-fixed (2 rule-1 bugs)  
**Impact on plan:** Required for correctness and deterministic evidence completeness.

## Issues Encountered
- None beyond implementation-time bugs auto-fixed above.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Evidence-chain service implementation can build on stable tax-assist contracts and ambiguity vocabulary.

## Self-Check: PASSED

- Found file: `.planning/phases/18-tax-assist-evidence-chain-interfaces/18-01-SUMMARY.md`
- Found commits: `749adf0`, `ca1f8b6`, `75b6b97`

---
*Phase: 18-tax-assist-evidence-chain-interfaces*
*Completed: 2026-03-29*
