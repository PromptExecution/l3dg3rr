# Phase 3: Rule-Driven Classification & Flagging - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning
**Mode:** Auto-generated (autonomous continuation)

<domain>
## Phase Boundary

Implement runtime-editable classification and review-flag workflows over the deterministic ingest outputs, including MCP query and rule-test pathways.

</domain>

<decisions>
## Implementation Decisions

### Rules Runtime
- Classification and flag logic should be runtime-editable and file-backed (Rhai script direction retained).
- Rule execution must be deterministic for a given input transaction and rule set revision.
- Rule-test capability must be exposed via MCP for sample transaction validation.

### Review Queue
- Low-confidence or policy-triggered classifications must produce explicit unresolved flags.
- Flag query surface should support year and status filters for operator workflows.

### the agent's Discretion
- Internal rule-engine adapter structure and error taxonomy are at agent discretion if external behavior remains explicit, deterministic, and test-covered.

</decisions>

<code_context>
## Existing Code Insights

### Reusable Assets
- `crates/ledger-core/src/ingest.rs` provides deterministic tx identity and replay-safe ingest output.
- `crates/turbo-mcp/src/lib.rs` exposes explicit MCP tool request/response contracts.

### Established Patterns
- TDD-first implementation with requirement-linked tests.
- Contract-first boundaries; fail-fast validation before mutation.

### Integration Points
- Classification results should attach to ingested transaction representations.
- MCP tools to add/query flag state and run rule tests should compose with existing service patterns.

</code_context>

<specifics>
## Specific Ideas

- Preserve Git-friendly, auditable behavior for rule changes and classification outcomes.
- Keep query contracts clear and intentionally typed for operator tooling.

</specifics>

<deferred>
## Deferred Ideas

- Advanced analytics over flag queues remain out of phase scope.

</deferred>
