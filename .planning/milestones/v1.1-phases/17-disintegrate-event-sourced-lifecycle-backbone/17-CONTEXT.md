# Phase 17: Disintegrate Event-Sourced Lifecycle Backbone - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning
**Source:** ROADMAP + REQUIREMENTS + PROJECT + user constraints

<domain>
## Phase Boundary

Implement append-only lifecycle event persistence, deterministic replay/state reconstruction, and filtered event-history queries by transaction, document, and time windows.

</domain>

<decisions>
## Implementation Decisions

### Locked Decisions
- D-01: Scope is limited to append-only lifecycle events, deterministic replay/reconstruction, and event-history filtering by transaction/document/time (EVT-01/02/03).
- D-02: Integrate with existing l3dg3rr service + MCP adapter/server boundaries; do not reinvent architecture or bypass established passthrough patterns.
- D-03: Plans must be test-first (RED->GREEN) for code-producing tasks.
- D-04: Event payloads, replay outputs, and query responses must be deterministic and small-model-friendly (stable keys/order, explicit fields).
- D-05: Preserve existing invariants and safety rules (no panic-prone financial-path shortcuts; append-only semantics must not permit mutation/deletion).

### the agent's Discretion
- Internal event module boundaries and storage format details, provided append-only and deterministic replay guarantees hold.
- Naming of lifecycle event types and filter request/response structures, provided they are explicit and stable.
- Exact MCP tool names for event replay/query surfaces, provided they follow existing `l3dg3rr_*` conventions and deterministic envelopes.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/ROADMAP.md` - Phase 17 goal, dependency, and success criteria.
- `.planning/REQUIREMENTS.md` - EVT-01, EVT-02, EVT-03 requirement definitions.
- `.planning/PROJECT.md` - local-first and accountant-auditable constraints.
- `.planning/STATE.md` - carry-forward decisions and deterministic contract patterns.
- `AGENTS.md` - invariant, MCP boundary, and workflow requirements.
- `.planning/phases/16-moku-hsm-deterministic-status-and-resume/16-03-SUMMARY.md` - deterministic lifecycle/checkpoint and MCP boundary patterns to extend.
- `crates/turbo-mcp/src/lib.rs` - TurboLedgerService contracts and current audit/state surfaces.
- `crates/turbo-mcp/src/hsm.rs` - lifecycle marker and guarded transition vocabulary for event typing/replay anchoring.
- `crates/turbo-mcp/src/mcp_adapter.rs` - deterministic adapter parsing/result envelope patterns.
- `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` - tools/list and tools/call dispatch patterns.
- `crates/turbo-mcp/tests/hsm_mcp_e2e.rs` - subprocess MCP lifecycle test pattern.

</canonical_refs>

<specifics>
## Specific Ideas

- Add a dedicated event domain module and append-only store contract layered on existing lifecycle transitions/classification/reconciliation boundaries.
- Use deterministic event IDs/sequence ordering tied to existing lifecycle marker and transaction/document identifiers.
- Provide replay and filtered history APIs in service first, then expose over MCP with deterministic transport payloads.

</specifics>

<deferred>
## Deferred Ideas

- Tax-assist evidence synthesis and ambiguity-review UX (Phase 18).
- New external databases or cloud event buses.
- Cross-tenant/multi-user event partitioning.

</deferred>

---

*Phase: 17-disintegrate-event-sourced-lifecycle-backbone*
*Context gathered: 2026-03-29*
