# Phase 16: Moku HSM Deterministic Status and Resume - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning
**Source:** ROADMAP + REQUIREMENTS + PROJECT (+ Phase 13/14/15 outcomes)

<domain>
## Phase Boundary

Implement lifecycle HSM state/substate orchestration for ingest -> normalize -> validate -> reconcile -> commit -> summarize with guarded transitions, resumability from last valid checkpoint, and deterministic status output designed for small-model agent execution.

</domain>

<decisions>
## Implementation Decisions

### Locked Decisions
- D-01: Keep upstream rustledger/docling passthrough/proxy interfaces unchanged; implement HSM as l3dg3rr-owned orchestration above existing tools.
- D-02: Encode lifecycle using explicit parent states and deterministic substates covering ingest, normalize, validate, reconcile, commit, and summarize (HSM-01).
- D-03: All transitions must be guard-validated and return explicit deterministic blocked reasons and transition evidence when denied (HSM-02).
- D-04: Resume must continue only from the last valid checkpoint/state marker and must not bypass guardrails or invariants (HSM-03).
- D-05: Status payloads must include deterministic Display hints for small-model agents using fixed-key, fixed-vocabulary fields (for example: `display_state`, `next_hint`, `resume_hint`, sorted `blockers`).
- D-06: Preserve local-first, deterministic, panic-safe behavior and align with established phase 13-15 MCP boundary patterns.

### the agent's Discretion
- Internal HSM module layout and state transition table representation.
- Session/checkpoint persistence strategy (in-memory plus deterministic serialization shape) as long as resumability and guard fidelity hold.
- Exact HSM MCP tool naming, provided names remain explicit, stable, and consistent with existing `l3dg3rr_*` surfaces.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/ROADMAP.md` - Phase 16 goal, dependencies, and success criteria.
- `.planning/REQUIREMENTS.md` - HSM-01/HSM-02/HSM-03 definitions.
- `.planning/PROJECT.md` - local-first constraints and financial safety requirements.
- `.planning/STATE.md` - carry-forward decisions and deterministic contract patterns.
- `AGENTS.md` - operator constraints, invariant expectations, and session-learning rules.
- `.planning/phases/13-mcp-boundary-and-agent-only-runtime-surface/13-03-SUMMARY.md` - MCP transport/passthrough boundary discipline.
- `.planning/phases/14-ontology-persistence-and-query-surface/14-02-SUMMARY.md` - deterministic payload shaping patterns.
- `.planning/phases/15-reconciliation-and-commit-guardrails/15-02-SUMMARY.md` - explicit stage-guard and deterministic blocked diagnostics patterns.
- `crates/turbo-mcp/src/lib.rs` - current service contracts and tool wrappers.
- `crates/turbo-mcp/src/reconciliation.rs` - deterministic stage guardrail implementation style.
- `crates/turbo-mcp/src/mcp_adapter.rs` - adapter parsing and deterministic transport payload shaping patterns.
- `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` - tools/list and tools/call transport dispatch pattern.

</canonical_refs>

<specifics>
## Specific Ideas

- Add a dedicated HSM domain module with explicit state/substate enums, guard evaluation, transition evidence, and deterministic status rendering.
- Use test-first RED->GREEN planning across service contracts, resume behavior, and MCP transport surface.
- Keep Display hints concise and deterministic for small models (single canonical state token + stable next action hint + sorted blockers/reasons).

</specifics>

<deferred>
## Deferred Ideas

- Full event persistence/replay backbone (Phase 17).
- Tax-assist evidence chain and ambiguity review outputs (Phase 18).
- Cross-session cloud orchestration or multi-tenant workflows (out of scope for local-first milestone).

</deferred>

---

*Phase: 16-moku-hsm-deterministic-status-and-resume*
*Context gathered: 2026-03-29*
