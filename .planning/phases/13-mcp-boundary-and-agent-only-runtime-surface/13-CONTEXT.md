# Phase 13: MCP Boundary and Agent-Only Runtime Surface - Context

**Gathered:** 2026-03-29  
**Status:** Ready for planning  
**Source:** User direction + milestone gap-audit

<domain>
## Phase Boundary

Phase 13 establishes an enforceable MCP-only runtime boundary for sandboxed agents.  
`l3dg3rr` must expose higher-level financial-document abstractions while leveraging existing upstream MCP interfaces rather than replacing them.

</domain>

<decisions>
## Implementation Decisions

### Locked Decisions
- Do not reinvent upstream MCP servers for ledger/doc conversion primitives.
- Use Rustledger MCP as an existing ledger-oriented capability source and expose it via passthrough/proxy in `l3dg3rr`.
- Use Docling MCP as an existing document-processing capability source and expose it via passthrough/proxy in `l3dg3rr`.
- `l3dg3rr` owns higher-level orchestration/normalization/reconciliation abstractions above these MCP backends.
- Sandboxed agents must use `l3dg3rr` MCP capabilities only for processing/understanding workflows.
- Responses must be deterministic and concise for small-model agents.

### the agent's Discretion
- Exact adapter layout and process topology (embedded client, sidecar, or delegated subprocess) as long as passthrough/proxy semantics and deterministic contracts are preserved.
- Error-shaping, retries, and timeout policies for upstream MCP calls.
- MCP tool naming and grouping for `l3dg3rr` abstraction layer.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Planning/Scope
- `.planning/ROADMAP.md` — Phase 13 goal, dependencies, and success criteria
- `.planning/REQUIREMENTS.md` — REQ IDs `DOC-01`, `DOC-02`, `DOC-03`
- `.planning/v1.1-v1.1-MILESTONE-AUDIT.md` — integration blockers and flow gaps driving this phase

### Existing Local Implementation
- `crates/turbo-mcp/src/lib.rs` — current in-process service contracts to adapt behind MCP boundary
- `crates/turbo-mcp/tests/interface.rs` — current direct-call interface behavior
- `crates/turbo-mcp/tests/e2e_bdd.rs` — current E2E behavior needing MCP-boundary enforcement
- `AGENTS.md` — project-wide operating constraints and session-learning requirements

### Upstream MCP Interfaces (external)
- `https://www.npmjs.com/package/@rustledger/mcp-server` — Rustledger MCP package
- `https://docling-project.github.io/docling/usage/mcp/` — Docling MCP usage docs

</canonical_refs>

<specifics>
## Specific Ideas

- Add proxy tools that explicitly identify backend provider (`rustledger` or `docling`) and preserve traceability.
- Include backend provenance in responses (`provider`, `backend_tool`, `backend_version`, `backend_call_id` where available).
- Add a compact status tool that reports readiness of upstream MCP backends and local session preconditions.

</specifics>

<deferred>
## Deferred Ideas

- Full HSM transition engine (Phase 16)
- Full ontology persistence/query implementation (Phase 14)
- Full disintegrate event backbone (Phase 17)

</deferred>

---

*Phase: 13-mcp-boundary-and-agent-only-runtime-surface*  
*Context gathered: 2026-03-29*
