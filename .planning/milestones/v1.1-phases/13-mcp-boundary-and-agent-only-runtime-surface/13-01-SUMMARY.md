---
phase: 13-mcp-boundary-and-agent-only-runtime-surface
plan: 01
subsystem: api
tags: [mcp, stdio, proxy, deterministic-contracts]
requires:
  - phase: 06-local-deployment-release-readiness
    provides: local runtime and release-ready turbo-mcp baseline
provides:
  - stdio MCP transport entrypoint for initialize/tools/list/tools/call
  - adapter boundary with passthrough provider metadata and canonical field normalization
  - deterministic pipeline status and explicit MCP/domain error-shaping helpers
affects: [14-ontology-model-and-storage-foundation, 16-moku-state-machine-orchestrator]
tech-stack:
  added: [serde_json]
  patterns: [adapter-first MCP boundary, deterministic status schema, explicit protocol-vs-domain error mapping]
key-files:
  created:
    - crates/turbo-mcp/tests/mcp_adapter_contract.rs
    - crates/turbo-mcp/src/mcp_adapter.rs
    - crates/turbo-mcp/src/bin/turbo-mcp-server.rs
  modified:
    - crates/turbo-mcp/Cargo.toml
    - crates/turbo-mcp/src/lib.rs
    - Cargo.lock
key-decisions:
  - "Implemented a local stdio MCP lifecycle boundary now (initialize/tools/list/tools/call) while keeping domain logic in TurboLedgerService."
  - "Separated protocol method errors from tool execution errors to keep deterministic isError semantics for agent retries."
patterns-established:
  - "Adapter module owns canonical/provenance shaping rather than embedding this logic inside server dispatch."
  - "Pipeline status contract uses fixed keys and stable enum-like status values (ready|blocked) with deterministic next_hint."
requirements-completed: [DOC-01, DOC-02]
duration: 25 min
completed: 2026-03-29
---

# Phase 13 Plan 01: MCP Boundary Proxy Surface Summary

**Stdio MCP transport with proxy tool catalog, deterministic canonical/provenance mapping, and stable status/error contracts for agent-only runtime access.**

## Performance

- **Duration:** 25 min
- **Started:** 2026-03-29T00:39:25Z
- **Completed:** 2026-03-29T01:04:45Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- Added DOC-01/DOC-02 contract tests defining MCP boundary expectations and deterministic payload fields.
- Implemented a runnable `turbo-mcp-server` stdio entrypoint supporting `initialize`, `tools/list`, and `tools/call`.
- Added `mcp_adapter` boundary helpers for tool catalog, canonical/provenance normalization, deterministic status output, and explicit error mapping.
- Passed full `cargo test -p turbo-mcp -- --nocapture` suite after hardening.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define MCP boundary/proxy contracts as failing tests** - `546c56e` (test)
2. **Task 2: Implement stdio MCP server and passthrough/proxy adapter layer** - `c95c675` (feat)
3. **Task 3: Finalize deterministic status/error shaping and full turbo-mcp verification** - `3f307c7` (fix)

## Files Created/Modified
- `crates/turbo-mcp/tests/mcp_adapter_contract.rs` - requirement-tagged DOC-01/DOC-02 contract tests.
- `crates/turbo-mcp/src/mcp_adapter.rs` - adapter boundary for catalog, normalization, status, and error helpers.
- `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` - stdio MCP transport runtime for lifecycle/tool methods.
- `crates/turbo-mcp/src/lib.rs` - exports `mcp_adapter` as crate surface.
- `crates/turbo-mcp/Cargo.toml` - adds `serde_json` dependency for deterministic MCP payload encoding.
- `Cargo.lock` - lockfile update for new dependency.

## Decisions Made
- Kept business logic in `TurboLedgerService`; implemented transport and schema concerns in a dedicated adapter/server boundary.
- Standardized tool-level failures as `isError: true` payloads and protocol-level unknown methods as JSON-RPC method errors.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Canonical currency was inferred from `source_ref` instead of account identity**
- **Found during:** Task 3 (deterministic status/error shaping)
- **Issue:** Currency derivation used the wrong input field, which could produce non-canonical values.
- **Fix:** Switched inference to `account`-based derivation and centralized response shaping helpers.
- **Files modified:** `crates/turbo-mcp/src/mcp_adapter.rs`, `crates/turbo-mcp/src/bin/turbo-mcp-server.rs`
- **Verification:** `cargo test -p turbo-mcp -- --nocapture`
- **Committed in:** `3f307c7`

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Correctness hardening only; no scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- MCP transport boundary and deterministic contract surface are now in place for additional provider-backed tools.
- Ready for Plan 13-02 and downstream ontology/state-machine phases that depend on stable agent runtime boundaries.

## Self-Check: PASSED
- Verified summary file exists on disk.
- Verified task commits `546c56e`, `c95c675`, and `3f307c7` exist in git history.

---
*Phase: 13-mcp-boundary-and-agent-only-runtime-surface*
*Completed: 2026-03-29*
