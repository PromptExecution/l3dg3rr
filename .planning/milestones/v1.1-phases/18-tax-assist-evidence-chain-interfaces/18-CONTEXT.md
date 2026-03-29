# Phase 18: Tax Assist Evidence-Chain Interfaces - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning
**Source:** ROADMAP + REQUIREMENTS + PROJECT + STATE + user constraints

<domain>
## Phase Boundary

Deliver deterministic tax-assist interfaces that derive from reconciled ontology truth, expose explainable evidence chains from source to event history to reconstructed state, and surface ambiguity review queues with provenance links for human signoff.

</domain>

<decisions>
## Implementation Decisions

### Locked Decisions
- D-01: Tax-assist outputs must derive from reconciled ontology truth only (TAXA-01).
- D-02: Evidence-chain retrieval must explicitly link `source -> events -> current state` (TAXA-02).
- D-03: Ambiguity must be flagged with linked provenance and explicit review state fields (TAXA-03).
- D-04: Payloads must remain deterministic and concise for small-model agents (stable key vocab, deterministic ordering, no verbose noise).
- D-05: Preserve MCP boundary and existing upstream passthrough approach; add l3dg3rr-owned surfaces without breaking existing tool contracts.
- D-06: Follow repository safety constraint: do not revert unrelated edits while implementing this phase.

### the agent's Discretion
- Internal tax-assist domain module layout and helper function boundaries.
- Exact evidence-chain response schema names, provided the chain is explicit, deterministic, and machine-readable.
- Exact MCP tool names for tax-assist surfaces, provided names follow established `l3dg3rr_*` pattern and preserve existing tool behavior.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/ROADMAP.md` - Phase 18 goal, dependencies, and success criteria.
- `.planning/REQUIREMENTS.md` - TAXA-01, TAXA-02, TAXA-03 requirement definitions.
- `.planning/PROJECT.md` - local-first and accountant-auditable workflow constraints.
- `.planning/STATE.md` - carry-forward deterministic payload and MCP boundary decisions.
- `AGENTS.md` - TurboLedgerService canonical contract and invariant rules.
- `.planning/phases/14-ontology-persistence-and-query-surface/14-02-SUMMARY.md` - ontology transport and deterministic export/query payload conventions.
- `.planning/phases/15-reconciliation-and-commit-guardrails/15-02-SUMMARY.md` - reconciliation blocked/ready deterministic semantics.
- `.planning/phases/17-disintegrate-event-sourced-lifecycle-backbone/17-02-SUMMARY.md` - replay and reconstruction deterministic diagnostics.
- `.planning/phases/17-disintegrate-event-sourced-lifecycle-backbone/17-03-SUMMARY.md` - MCP event query transport and filter envelope patterns.
- `crates/turbo-mcp/src/lib.rs` - service-level ontology/reconciliation/event interfaces.
- `crates/turbo-mcp/src/ontology.rs` - deterministic entity/edge/path traversal and provenance model.
- `crates/turbo-mcp/src/events.rs` - append-only event history and replay projection contracts.
- `crates/turbo-mcp/src/mcp_adapter.rs` - adapter parsing and deterministic MCP payload shaping.
- `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` - tools/list and tools/call dispatch patterns.

</canonical_refs>

<specifics>
## Specific Ideas

- Add a dedicated tax-assist domain surface that composes ontology query/export, reconciliation stage checks, and event replay/history into deterministic tax-evidence outputs.
- Keep evidence-chain payload concise with stable sections: `source`, `events`, `current_state`, and `ambiguity`.
- Expose tax-assist capabilities over MCP via additive `l3dg3rr_*` tools and deterministic transport envelopes.

</specifics>

<deferred>
## Deferred Ideas

- Full autonomous tax filing workflows and filing submission UX.
- External SaaS dependency for inference/review workflow orchestration.
- Non-tax analytics dashboards outside tax-assist evidence scope.

</deferred>

---

*Phase: 18-tax-assist-evidence-chain-interfaces*
*Context gathered: 2026-03-29*
