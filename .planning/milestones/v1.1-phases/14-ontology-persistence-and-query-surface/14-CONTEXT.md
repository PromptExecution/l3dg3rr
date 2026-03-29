# Phase 14: Ontology Persistence and Query Surface - Context

**Gathered:** 2026-03-29  
**Status:** Ready for planning  
**Source:** Roadmap + requirements + Phase 13 outcomes

<domain>
## Phase Boundary

Build ontology persistence/query/serialization for financial document knowledge while preserving the new MCP boundary established in Phase 13.

</domain>

<decisions>
## Implementation Decisions

### Locked Decisions
- Keep upstream MCP passthrough pattern: rustledger/docling remain external capability providers.
- l3dg3rr owns higher-level ontology abstractions and cross-source evidence graphing.
- Ontology outputs must be deterministic, concise, and machine-readable for small-model agents.
- Local-first and git-compatible persistence remains mandatory.

### the agent's Discretion
- Concrete storage implementation (HelixDB projection vs fallback local graph store) as long as ONTO-01/02/03 are satisfied.
- Query API surface and indexing strategy.
- Internal schema organization for entities/edges/provenance.

</decisions>

<canonical_refs>
## Canonical References

- `.planning/ROADMAP.md`
- `.planning/REQUIREMENTS.md`
- `.planning/PROJECT.md`
- `.planning/phases/13-mcp-boundary-and-agent-only-runtime-surface/13-VERIFICATION.md`
- `crates/turbo-mcp/src/mcp_adapter.rs`
- `crates/turbo-mcp/src/lib.rs`
- `AGENTS.md`

</canonical_refs>

<specifics>
## Specific Ideas

- Persist entities: document, account, institution, transaction, tax-category, evidence.
- Represent relationships with explicit provenance edges.
- Expose read/query + deterministic serialization via MCP tools.

</specifics>

<deferred>
## Deferred Ideas

- Full reconciliation-gate enforcement (Phase 15)
- Full HSM transition model (Phase 16)
- Full event-sourcing reconstruction layer (Phase 17)

</deferred>

---

*Phase: 14-ontology-persistence-and-query-surface*  
*Context gathered: 2026-03-29*
