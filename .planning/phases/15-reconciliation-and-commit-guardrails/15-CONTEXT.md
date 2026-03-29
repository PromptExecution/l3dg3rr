# Phase 15: Reconciliation and Commit Guardrails - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning
**Source:** ROADMAP + REQUIREMENTS + PROJECT (+ Phase 13/14 decisions)

<domain>
## Phase Boundary

Implement explicit validate/reconcile/commit guardrails that block transaction truth commits unless double-entry and source-reconciliation invariants pass with deterministic diagnostics.

</domain>

<decisions>
## Implementation Decisions

### Locked Decisions
- D-01: Do not reinvent upstream rustledger/docling MCP interfaces; keep passthrough/proxy boundary intact.
- D-02: Add reconciliation and commit guardrails as l3dg3rr-owned abstractions layered above upstream tools.
- D-03: Fail closed on invariant breaches (imbalance, duplicate, schema mismatch) with deterministic, explicit machine-readable diagnostics.
- D-04: Validate/reconcile/commit stages must be explicit and enforced before any commit success is returned.
- D-05: Keep local-first deterministic behavior and existing content-hash identity guarantees.

### the agent's Discretion
- Internal reconciliation data structures and module boundaries.
- Exact naming of stage/tool functions, provided stage semantics remain explicit and deterministic.
- Test file split and command granularity, provided all RECON requirements have executable automated coverage.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

- `.planning/ROADMAP.md` - Phase 15 goal, requirements, and success criteria.
- `.planning/REQUIREMENTS.md` - RECON-01/02/03 definitions.
- `.planning/PROJECT.md` - milestone constraints and upstream passthrough decision.
- `.planning/STATE.md` - latest executed phase and locked carry-forward decisions.
- `AGENTS.md` - operational guardrails and deterministic/invariant expectations.
- `.planning/phases/13-mcp-boundary-and-agent-only-runtime-surface/13-03-SUMMARY.md` - MCP passthrough/proxy boundary decisions.
- `.planning/phases/14-ontology-persistence-and-query-surface/14-02-SUMMARY.md` - deterministic payload and l3dg3rr-owned abstraction patterns.
- `crates/turbo-mcp/src/lib.rs` - current service/tool contracts.
- `crates/turbo-mcp/src/mcp_adapter.rs` - deterministic adapter parsing/error mapping patterns.
- `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` - stdio MCP tools/list and tools/call dispatch pattern.

</canonical_refs>

<specifics>
## Specific Ideas

- Add service-level reconciliation guardrails first, then expose MCP stage tools.
- Use test-first RED->GREEN task flow to lock deterministic diagnostics before implementation.
- Keep blocking diagnostics explicit (`isError`, `error_type`, stable message keys) for small-model agent reliability.

</specifics>

<deferred>
## Deferred Ideas

- Full lifecycle HSM transition orchestration (Phase 16).
- Event-sourced lifecycle/event replay backbone (Phase 17).
- Tax-assist evidence-chain outputs and ambiguity review surfaces (Phase 18).

</deferred>

---

*Phase: 15-reconciliation-and-commit-guardrails*
*Context gathered: 2026-03-29*
