---
phase: 13-mcp-boundary-and-agent-only-runtime-surface
plan: 02
subsystem: testing
tags: [mcp, stdio, e2e, docling, rustledger]
requires:
  - phase: 13-mcp-boundary-and-agent-only-runtime-surface-01
    provides: stdio transport boundary and adapter contract surface
provides:
  - MCP-only subprocess e2e validation for DOC-01, DOC-02, DOC-03
  - reproducible wrapper script for MCP e2e execution
  - operator runbook and validation map aligned to real test identifiers
affects: [phase-13-validation, verify-work, operator-runbook]
tech-stack:
  added: []
  patterns: [transport-only MCP lifecycle verification, deterministic replay assertions]
key-files:
  created:
    - crates/turbo-mcp/tests/mcp_stdio_e2e.rs
    - scripts/mcp_e2e.sh
    - docs/agent-mcp-runbook.md
  modified:
    - crates/turbo-mcp/src/mcp_adapter.rs
    - crates/turbo-mcp/src/bin/turbo-mcp-server.rs
    - .planning/phases/13-mcp-boundary-and-agent-only-runtime-surface/13-VALIDATION.md
key-decisions:
  - "DOC verification uses MCP subprocess transport only; direct service calls are excluded from this acceptance path."
  - "Replay responses return stable tx_ids even when inserted_count becomes zero on idempotent replays."
patterns-established:
  - "MCP lifecycle pattern: initialize -> notifications/initialized -> tools/list -> tools/call"
  - "Adapter returns deterministic canonical and provenance fields in transport payloads"
requirements-completed: [DOC-01, DOC-02, DOC-03]
duration: 10 min
completed: 2026-03-29
---

# Phase 13 Plan 02: MCP-only DOC Transport Verification Summary

**Subprocess MCP e2e coverage now proves ingest, canonical mapping, and replay idempotency through transport-only tool calls.**

## Performance

- **Duration:** 10 min
- **Started:** 2026-03-29T01:08:57Z
- **Completed:** 2026-03-29T01:19:12Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- Added RED-to-GREEN subprocess stdio tests for DOC-01/02/03 in `mcp_stdio_e2e`.
- Implemented MCP `proxy_docling_ingest_pdf` handling with deterministic canonical/provenance output and replay-safe tx ID semantics.
- Published an MCP-only runbook and updated Phase 13 validation mapping to exact command/test identifiers.

## Task Commits

1. **Task 1: Define MCP-only DOC requirement tests as failing subprocess scenarios** - `d95d617` (test)
2. **Task 2: Implement MCP subprocess harness and deterministic replay assertions** - `cf28f37` (feat)
3. **Task 3: Publish MCP-only runbook and update validation contract** - `787c501` (docs)

## Files Created/Modified
- `crates/turbo-mcp/tests/mcp_stdio_e2e.rs` - subprocess JSON-RPC harness and DOC transport assertions.
- `crates/turbo-mcp/src/mcp_adapter.rs` - ingest argument parsing, tool execution wrapper, deterministic replay tx-id handling.
- `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` - notifications handling and ingest tool routing via process-global service instance.
- `scripts/mcp_e2e.sh` - one-command MCP e2e runner.
- `docs/agent-mcp-runbook.md` - MCP-only bootstrap/lifecycle/discovery/replay operator guidance.
- `.planning/phases/13-mcp-boundary-and-agent-only-runtime-surface/13-VALIDATION.md` - real task/test command mapping and sign-off updates.

## Decisions Made
- Used transport-level subprocess verification as the acceptance boundary for DOC requirements.
- Preserved passthrough/proxy framing to rustledger/docling in docs and tool naming.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed test harness deadlock on notifications/initialized**
- **Found during:** Task 2
- **Issue:** Tests waited for a response to `notifications/initialized`, but compliant server behavior is no response.
- **Fix:** Changed notification helper to write-and-flush without read.
- **Files modified:** `crates/turbo-mcp/tests/mcp_stdio_e2e.rs`
- **Verification:** `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture`
- **Committed in:** `cf28f37`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for protocol-correct MCP lifecycle testing; no scope creep.

## Authentication Gates
None.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Phase 13 now has executable MCP-only verification and runbook coverage for DOC-01/02/03.
- Ready for `$gsd-verify-work` and phase closure flow.

## Self-Check: PASSED

```text
FOUND: .planning/phases/13-mcp-boundary-and-agent-only-runtime-surface/13-02-SUMMARY.md
FOUND: docs/agent-mcp-runbook.md
FOUND: scripts/mcp_e2e.sh
FOUND: crates/turbo-mcp/tests/mcp_stdio_e2e.rs
FOUND: commit d95d617
FOUND: commit cf28f37
FOUND: commit 787c501
```
