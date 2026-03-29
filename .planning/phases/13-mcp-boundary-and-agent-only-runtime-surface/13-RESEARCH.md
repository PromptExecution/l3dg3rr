# Phase 13: MCP Boundary and Agent-Only Runtime Surface - Research

**Researched:** 2026-03-29
**Domain:** Enforceable MCP transport boundary for `turbo-mcp` with MCP-only agent runtime workflows
**Confidence:** HIGH

<user_constraints>
## User Constraints

### Locked Decisions
- Sandboxed agents must ONLY interact through `turbo-mcp` abstract service capabilities (MCP interface), not direct in-process Rust calls.
- Phase scope is tied to requirements `DOC-01`, `DOC-02`, `DOC-03`.
- Setup/runtime must include state-machine awareness for small-model agents, with deterministic concise status hints.

### Claude's Discretion
- Choose the concrete MCP transport/runtime topology (stdio-first or hybrid).
- Choose repo layout for server binary and adapter wiring as long as MCP boundary is enforceable.
- Choose test architecture that proves MCP-only execution path end-to-end.

### Deferred Ideas (OUT OF SCOPE)
- Implementing full ontology/reconciliation/event backbone (Phases 14-17).
- Expanding tax-assist evidence-chain endpoints beyond what is required to enforce MCP runtime boundary in this phase.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOC-01 | Ingest statement PDFs through Docling/docling-mcp to normalized candidate rows with provenance | Add MCP-facing ingest orchestration tools and enforce invocation via transport server; include provenance-bearing response schema and deterministic error surface |
| DOC-02 | Deterministic canonical mapping to transaction schema (`account,date,amount,description,currency,source_ref`) | Define strict MCP tool schemas + normalization boundary in service, emit machine-checkable structured output |
| DOC-03 | Replay same source with stable candidate IDs and no duplicates | Keep content-hash idempotency in core, verify by MCP stdio E2E replay tests (no in-process bypass path) |
</phase_requirements>

## Summary

Current `turbo-mcp` is a library service with strong request/response structs and good invariant checks, but no enforceable MCP transport boundary. Tests execute by direct Rust calls (`TurboLedgerService::from_manifest_str(...).ingest_pdf(...)`), so agent-only isolation is not currently guaranteed. This directly matches the milestone audit blocker.

The standard, low-risk implementation is to keep `turbo-mcp` business logic where it is and add a real MCP server runtime (stdio first) that exposes only approved tools. Agents then interact through MCP handshake (`initialize` -> `notifications/initialized`) and `tools/list`/`tools/call` only. Existing logic can be reused behind the adapter; the main change is boundary enforcement and test strategy.

For small-model reliability, add one deterministic status surface now (for example `get_pipeline_status`) with compact enum-like fields and short display hints. Even before full HSM in Phase 16, this gives runbook/setup state awareness and reduces agent drift.

**Primary recommendation:** Implement an RMCP-based stdio server binary in this phase, wire all ingest/classification operations through MCP tool handlers, and convert E2E verification to MCP transport tests that never instantiate `TurboLedgerService` directly.

## Project Constraints (from CLAUDE.md)

- Excel workbook is the canonical human/audit layer.
- Currency values must use `rust_decimal::Decimal`; no float money semantics.
- Transaction identity must stay content-hash based (Blake3 over canonical fields).
- Local-first, single-user operation; no mandatory cloud dependency.
- Source naming contract must remain `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE`.
- Avoid panic-prone financial pipeline paths (`unwrap`, unchecked indexing).
- Keep work aligned with GSD workflow artifacts and phase context.

## Implementation Strategy

1. Add a transport runtime boundary:
- Introduce stdio MCP server entrypoint (binary target) that owns lifecycle and tool registry.
- Keep `TurboLedgerService` as backend implementation; do not duplicate domain logic in handlers.

2. Define explicit MCP tool contract layer:
- Publish tool schemas (input/output) for current capabilities and Phase-13 required ingest flow.
- Convert domain errors to MCP tool execution errors (`isError: true`) with actionable text.

3. Enforce agent-only operation in docs and tests:
- Provide a runbook that starts server, performs handshake, lists tools, then executes ingest flow.
- Add integration tests that spawn server subprocess and use MCP calls only.
- Retain a small set of internal unit tests for service logic, but mark them as non-agent path.

4. Add deterministic setup/status hints:
- Add a compact status tool to expose ready/not-ready states for manifest/workbook/rules/session.
- Keep responses strict and concise for small-model reasoning.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `rmcp` | 1.2.0 (`latest` on docs.rs as of 2026-03-29) | Official Rust MCP server/client SDK | Avoids hand-rolled JSON-RPC/lifecycle bugs and aligns with MCP spec |
| `tokio` | 1.50.0 (published 2026-03-03) | Async runtime for stdio server transport | Standard runtime for MCP server process orchestration |
| `serde` / `serde_json` | 1.0.228 / 1.0.149 | Tool argument/result encoding | Required for stable schema-driven tool I/O |
| `thiserror` | 2.0.18 | Typed boundary errors for adapter layer | Keeps deterministic and auditable error mapping |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tracing` + `tracing-subscriber` | 0.1.x / 0.3.x (project guidance) | Structured logs to stderr (not stdout) | Server runtime diagnostics without corrupting stdio MCP stream |
| `tempfile` | 3.x | Isolated MCP subprocess E2E tests | Required for replay/idempotency transport tests |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| RMCP stdio server | Hand-rolled JSON-RPC over stdin/stdout | Higher protocol-risk; violates "don't hand-roll" for complex protocol |
| Stdio-only transport now | Streamable HTTP now | Adds auth/origin/session complexity too early; local sandbox use-case prefers stdio |
| Single crate with bin target | Separate `turbo-mcp-server` crate | Separate crate gives cleaner boundary but adds workspace overhead; both viable |

**Installation (expected):**
```bash
cargo add rmcp tokio serde serde_json thiserror tracing tracing-subscriber --package turbo-mcp
```

Version verification note: for this Rust phase, use `cargo search` / docs.rs / crates.io before pinning in plan execution.

## Architecture Options

### Option A (Recommended): Same crate + binary target (`turbo-mcp`)
**What:** Keep `crates/turbo-mcp/src/lib.rs` as domain adapter and add `src/bin/turbo-mcp-server.rs` for MCP transport runtime.
**Pros:**
- Minimal refactor.
- Reuses all existing tests with incremental migration.
- Fastest path to enforceable boundary this phase.
**Cons:**
- Library and server concerns stay co-located.

### Option B: Split runtime crate (`turbo-mcp-server`)
**What:** Keep `turbo-mcp` as pure backend contract crate; create dedicated server crate.
**Pros:**
- Strong compile-time boundary between service core and transport runtime.
- Cleaner long-term for multiple transports.
**Cons:**
- More workspace churn and migration complexity in Phase 13.

### Option C: Streamable HTTP MCP server now
**What:** Implement remote-style HTTP MCP endpoint in this phase.
**Pros:**
- Future-ready for remote clients.
**Cons:**
- Requires origin/auth/session hardening immediately; unnecessary for local sandbox boundary milestone.

**Recommendation:** Option A in Phase 13, with an explicit follow-up decision gate for Option B if transport complexity grows by Phase 15+.

## Architecture Patterns

### Recommended Project Structure
```text
crates/turbo-mcp/
├── src/lib.rs                    # existing service + request/response contracts
├── src/bin/turbo-mcp-server.rs   # MCP stdio runtime entrypoint (new)
├── src/mcp_adapter.rs            # RMCP handler <-> TurboLedgerService mapping (new)
└── tests/
    ├── mcp_stdio_e2e.rs          # subprocess MCP transport tests (new)
    └── ...                       # existing domain tests
```

### Pattern 1: Thin Transport Adapter, Strict Domain Invariants
**What:** `call_tool` decodes args, delegates to existing service methods, returns structured content.
**When to use:** For every MCP tool in this phase.
**Example:**
```rust
// Source: https://github.com/modelcontextprotocol/rust-sdk
let transport = (tokio::io::stdin(), tokio::io::stdout());
let server = my_handler.serve(transport).await?;
```

### Pattern 2: MCP Lifecycle-First Boot
**What:** Require initialize/initialized before normal tool operation.
**When to use:** All E2E harnesses and runbooks.
**Example:** Client performs `initialize`, then sends `notifications/initialized`, then `tools/list` and `tools/call`.

### Pattern 3: Deterministic Status Tool for Small Models
**What:** Add a compact status tool with fixed enums and short hints.
**When to use:** Setup/bootstrap and preflight checks.
**Example response shape:**
```json
{
  "phase": "ingest_ready",
  "ready": true,
  "blockers": [],
  "hint": "Run tools/call ingest_pdf next."
}
```

### Anti-Patterns to Avoid
- **Writing logs to stdout:** corrupts stdio MCP message stream.
- **Dual execution paths (direct + MCP) for agent workflow:** makes boundary unenforceable.
- **Embedding business logic inside tool handlers:** causes drift from existing invariants.
- **Free-form status text:** harms small-model determinism and machine-checkability.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| MCP protocol handshake | custom JSON-RPC lifecycle code | `rmcp` server/client flow | Avoids subtle `initialize`/`initialized` sequencing bugs |
| Tool schema negotiation | ad hoc argument parsing protocol | MCP tool schema (`inputSchema`/`outputSchema`) | Standardized discoverability and safer agent invocation |
| Transport framing | custom newline protocol parser | RMCP stdio transport primitives | Prevents stdout framing and parsing errors |
| Error taxonomy in tool responses | opaque string failures only | MCP protocol errors + `isError` tool results | Enables LLM self-correction loops |

**Key insight:** The hard part is boundary correctness, not business logic. Keep domain logic where it is; standardize transport and contract semantics.

## Common Pitfalls

### Pitfall 1: In-Process Tests Masquerading as MCP E2E
**What goes wrong:** Tests pass while boundary remains bypassable.
**Why it happens:** Calling `TurboLedgerService` directly in integration tests.
**How to avoid:** Require at least one gate suite that spawns server subprocess and uses MCP calls only.
**Warning signs:** No tests exercising `initialize`, `tools/list`, `tools/call`.

### Pitfall 2: Stdout Contamination
**What goes wrong:** MCP JSON stream becomes invalid.
**Why it happens:** Debug prints/logs to stdout in server mode.
**How to avoid:** Route all logs to stderr; reserve stdout exclusively for protocol messages.
**Warning signs:** Random parse errors despite valid tool code.

### Pitfall 3: Spec-Invalid Tool Error Mapping
**What goes wrong:** Clients cannot recover or retry correctly.
**Why it happens:** Mixing protocol errors and execution errors arbitrarily.
**How to avoid:** Use JSON-RPC errors for malformed/unknown tool; use `isError: true` for domain/tool execution failures.
**Warning signs:** Client retries malformed payloads or fails to surface actionable correction hints.

### Pitfall 4: Non-Deterministic Status Hints
**What goes wrong:** Small models choose wrong next steps.
**Why it happens:** Free-form text with hidden state assumptions.
**How to avoid:** Fixed enum states + concise deterministic hint field + blocker list.
**Warning signs:** Same state yields different wording across runs.

## Code Examples

Verified patterns from official and current code:

### MCP stdio server bootstrap (RMCP)
```rust
// Source: https://github.com/modelcontextprotocol/rust-sdk
use tokio::io::{stdin, stdout};
let transport = (stdin(), stdout());
let server = service.serve(transport).await?;
```

### MCP tool discovery/call requirements
```json
{ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }
{ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": { "name": "ingest_pdf", "arguments": { "...": "..." } } }
```

### Existing deterministic idempotent core behavior to preserve
```rust
let first = service.ingest_statement_rows(req.clone())?;
let second = service.ingest_statement_rows(req)?;
assert_eq!(first.inserted_count, 1);
assert_eq!(second.inserted_count, 0);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| HTTP+SSE transport expectation | Streamable HTTP replaces legacy HTTP+SSE in modern spec | 2025 spec evolution | New remote servers should target Streamable HTTP; local servers still strongly support stdio |
| Ad hoc tool outputs | Structured tool schemas (`inputSchema` + optional `outputSchema`) | Current MCP spec | Better validation/discoverability and safer client automation |

**Deprecated/outdated:**
- Treating in-process API invocation as sufficient proof of MCP compliance.

## Open Questions

1. **Boundary hardening level for direct Rust callers**
   - What we know: Existing tests and examples call service directly.
   - What's unclear: Whether Phase 13 should forbid this in CI or only for agent/runtime paths.
   - Recommendation: Keep direct unit tests for logic, but require MCP-only suites for all DOC requirement acceptance.

2. **Status tool scope in this phase vs Phase 16 HSM**
   - What we know: User requires deterministic concise status hints now.
   - What's unclear: Whether full transition guards ship now.
   - Recommendation: Ship minimal deterministic status surface now; reserve guarded transition engine for Phase 16.

3. **Docling integration level in Phase 13**
   - What we know: DOC-01 references Docling/docling-mcp.
   - What's unclear: Whether to add live Docling dependency in this phase or keep extracted-row contract with MCP-only path.
   - Recommendation: Keep Docling adapter behind MCP contract and phase-gate live extraction integration by deterministic fixture tests first.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | Build/test server and crates | ✓ | 1.88.0 | — |
| `rustc` | Compile MCP runtime | ✓ | 1.88.0 | — |
| `node` | GSD phase tooling/scripts | ✓ | v22.15.0 | — |
| `python3` | Optional helper tooling only | ✓ | 3.10.12 | not required for boundary work |

**Missing dependencies with no fallback:**
- None identified for Phase 13 implementation and validation.

**Missing dependencies with fallback:**
- None.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` integration tests |
| Config file | none (workspace default) |
| Quick run command | `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture` |
| Full suite command | `cargo test --workspace -- --nocapture` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DOC-01 | MCP-only ingest flow returns normalized candidates with provenance via tool calls | integration (subprocess stdio MCP) | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_01_mcp_only_ingest -- --nocapture` | ❌ Wave 0 |
| DOC-02 | Canonical schema mapping deterministic via MCP tool contract | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_02_canonical_mapping -- --nocapture` | ❌ Wave 0 |
| DOC-03 | Replay via MCP yields stable IDs and zero duplicates | integration | `cargo test -p turbo-mcp --test mcp_stdio_e2e doc_03_replay_idempotent -- --nocapture` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture`
- **Per wave merge:** `cargo test -p turbo-mcp -- --nocapture`
- **Phase gate:** `cargo test --workspace -- --nocapture`

### Wave 0 Gaps
- [ ] `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` — real MCP runtime boundary entrypoint
- [ ] `crates/turbo-mcp/src/mcp_adapter.rs` — tool schema + handler mapping layer
- [ ] `crates/turbo-mcp/tests/mcp_stdio_e2e.rs` — subprocess MCP-only DOC requirement tests
- [ ] `scripts/mcp_e2e.sh` (optional) — reproducible runbook command wrapper

## Phased Tasks (Execution-Ready)

1. **Task A: Add transport runtime**
- Create stdio MCP server binary and wire lifecycle-compliant startup.
- Ensure no stdout logging.

2. **Task B: Register tools and schemas**
- Map existing service capabilities to MCP tools with explicit input/output schemas.
- Add deterministic status tool for setup/state awareness.

3. **Task C: Error and status normalization**
- Convert domain errors to MCP execution errors with actionable messages.
- Keep concise deterministic status hints.

4. **Task D: MCP-only E2E tests**
- Build subprocess client harness running initialize/list/call flow.
- Implement DOC-01/02/03 tests through transport only.

5. **Task E: Runbook and operator docs**
- Document bootstrap, handshake expectations, tool discovery, and troubleshooting.

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Boundary regression from new direct-call tests | Medium | High | Add CI check that DOC acceptance tests use subprocess MCP only |
| Protocol mismatch across client/server versions | Medium | High | Pin negotiated version and assert handshake fields in tests |
| Stdout contamination by logs | Medium | High | Enforce stderr logging and add parser-failure regression test |
| Over-scoping into Phase 16 HSM | Medium | Medium | Limit Phase 13 to deterministic status hints, not full guarded transitions |
| Tool-schema drift vs backend structs | Low | Medium | Keep schema generation colocated with adapter and validate with contract tests |

## Sources

### Primary (HIGH confidence)
- MCP Lifecycle spec (initialize/initialized/version/capabilities): https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle
- MCP Transports spec (stdio + Streamable HTTP requirements): https://modelcontextprotocol.io/specification/2025-11-25/basic/transports
- MCP Tools spec (`tools/list`, `tools/call`, schemas, error handling): https://modelcontextprotocol.io/specification/2025-11-25/server/tools
- Official Rust SDK (`rmcp`) examples (`serve`, stdio transport, handlers): https://github.com/modelcontextprotocol/rust-sdk
- Current codebase contracts and tests:
  - `crates/turbo-mcp/src/lib.rs`
  - `crates/turbo-mcp/tests/interface.rs`
  - `crates/turbo-mcp/tests/e2e_bdd.rs`
  - `crates/turbo-mcp/tests/e2e_mvp_flow.rs`
  - `.planning/v1.1-v1.1-MILESTONE-AUDIT.md`

### Secondary (MEDIUM confidence)
- `rmcp` crate latest docs.rs landing page (version visibility): https://docs.rs/crate/rmcp/latest/source/
- `tokio` crate version listing: https://docs.rs/crate/tokio/1.50.0
- `serde_json` latest docs.rs: https://docs.rs/crate/serde_json/latest
- `thiserror` latest docs.rs: https://docs.rs/crate/thiserror/latest

### Tertiary (LOW confidence)
- None.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - validated against official MCP spec, Rust SDK, and docs.rs crate metadata.
- Architecture: HIGH - directly derived from current repository structure + audit blockers + spec constraints.
- Pitfalls: HIGH - observed in current code/tests and documented in protocol requirements.

**Research date:** 2026-03-29
**Valid until:** 2026-04-28
