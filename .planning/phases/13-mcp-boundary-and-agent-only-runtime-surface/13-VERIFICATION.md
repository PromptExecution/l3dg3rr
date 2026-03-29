---
phase: 13-mcp-boundary-and-agent-only-runtime-surface
verified: 2026-03-29T01:27:58Z
status: gaps_found
score: 6/7 must-haves verified
gaps:
  - truth: "Proxy/passthrough model to rustledger and docling is executable through MCP tool calls."
    status: failed
    reason: "The tool catalog advertises `proxy_rustledger_ingest_statement_rows`, but the stdio server does not route this tool in `tools/call` and falls back to unknown-tool error."
    artifacts:
      - path: "crates/turbo-mcp/src/mcp_adapter.rs"
        issue: "Catalog includes rustledger proxy surface name without a corresponding executable handler path."
      - path: "crates/turbo-mcp/src/bin/turbo-mcp-server.rs"
        issue: "`tools/call` match only handles `l3dg3rr_get_pipeline_status` and `proxy_docling_ingest_pdf`."
      - path: "docs/agent-mcp-runbook.md"
        issue: "Runbook states rustledger proxy surface remains available, but transport handler is missing."
    missing:
      - "Add a `tools/call` branch for `proxy_rustledger_ingest_statement_rows`."
      - "Implement adapter parsing/response shaping for rustledger statement-row ingest with deterministic/provenance fields."
      - "Add transport-level test coverage proving rustledger proxy tool is callable over MCP."
---

# Phase 13: MCP Boundary and Agent-Only Runtime Surface Verification Report

**Phase Goal:** Enforce MCP transport boundary so sandboxed agents can only use turbo-mcp capabilities.
**Verified:** 2026-03-29T01:27:58Z
**Status:** gaps_found
**Re-verification:** No - initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Agents can reach ingestion capabilities through a real MCP transport surface instead of direct Rust calls. | ✓ VERIFIED | Stdio server handles `initialize`, `tools/list`, `tools/call` and routes ingest via adapter/service path in [turbo-mcp-server.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/src/bin/turbo-mcp-server.rs#L32) and [mcp_adapter.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/src/mcp_adapter.rs#L144). |
| 2 | l3dg3rr MCP responses expose deterministic canonical fields and provenance metadata. | ✓ VERIFIED | Canonical+provenance shaping in [mcp_adapter.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/src/mcp_adapter.rs#L61); transport assertions in [mcp_stdio_e2e.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/tests/mcp_stdio_e2e.rs#L142). |
| 3 | A concise deterministic status tool reports upstream readiness and blockers. | ✓ VERIFIED | `status/blockers/next_hint` contract and stable values in [mcp_adapter.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/src/mcp_adapter.rs#L21). |
| 4 | DOC-01 ingestion executes end-to-end through MCP tool calls only. | ✓ VERIFIED | Subprocess MCP test lifecycle + ingest path in [mcp_stdio_e2e.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/tests/mcp_stdio_e2e.rs#L109). |
| 5 | DOC-02 canonical mapping is deterministic and validated through transport assertions. | ✓ VERIFIED | Deterministic mapping assertions in [mcp_stdio_e2e.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/tests/mcp_stdio_e2e.rs#L142). |
| 6 | DOC-03 replaying same source via MCP yields stable IDs and no duplicate insertions. | ✓ VERIFIED | Replay idempotency assertions in [mcp_stdio_e2e.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/tests/mcp_stdio_e2e.rs#L170). |
| 7 | Proxy/passthrough model to rustledger and docling is executable through MCP tool calls. | ✗ FAILED | Catalog exposes rustledger proxy in [mcp_adapter.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/src/mcp_adapter.rs#L11) but server has no `proxy_rustledger_ingest_statement_rows` handler in [turbo-mcp-server.rs](/home/brianh/promptexecution/mbse/l3dg3rr/crates/turbo-mcp/src/bin/turbo-mcp-server.rs#L59). |

**Score:** 6/7 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` | stdio MCP transport boundary | ✓ VERIFIED | Exists, substantive lifecycle handling, wired to adapter + service. |
| `crates/turbo-mcp/src/mcp_adapter.rs` | proxy/passthrough mapping + deterministic shaping | ⚠️ HOLLOW - wired but data disconnected | Docling proxy path is wired; rustledger proxy is listed but disconnected from executable `tools/call` handler. |
| `crates/turbo-mcp/tests/mcp_adapter_contract.rs` | deterministic contract tests | ✓ VERIFIED | Requirement-tagged tests for catalog/canonical/status pass. |
| `crates/turbo-mcp/tests/mcp_stdio_e2e.rs` | subprocess MCP-only DOC validation | ✓ VERIFIED | Lifecycle and DOC-01/02/03 assertions pass. |
| `scripts/mcp_e2e.sh` | reproducible MCP e2e command | ✓ VERIFIED | Script runs `mcp_stdio_e2e` suite and exits 0. |
| `docs/agent-mcp-runbook.md` | MCP-only setup/discovery/troubleshooting runbook | ⚠️ PARTIAL | Useful and aligned for docling path; rustledger surface claim does not match callable handlers. |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| `turbo-mcp stdio server` | `mcp_adapter tool handlers` | `initialize/tools/list/tools/call lifecycle` | WIRED | Server dispatch uses adapter functions for list/status/ingest. |
| `mcp_adapter passthrough handlers` | `TurboLedgerService methods` | `provider-tagged proxy mapping (rustledger/docling)` | PARTIAL | `proxy_docling_ingest_pdf` reaches `service.ingest_pdf`; rustledger proxy tool is not routed. |
| `mcp_stdio_e2e harness` | `turbo-mcp-server subprocess` | `initialize -> notifications/initialized -> tools/list -> tools/call` | WIRED | Harness spawns binary and drives full MCP lifecycle. |
| `ingest tool replay path` | `deterministic tx_ids + inserted_count` | `repeat same source input over MCP` | WIRED | E2E replay checks `inserted_count` 1->0 and identical `tx_ids`. |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| `crates/turbo-mcp/src/mcp_adapter.rs` | `canonical_rows` | `request.extracted_rows` + `service.ingest_pdf()` | Yes | ✓ FLOWING |
| `crates/turbo-mcp/src/mcp_adapter.rs` | `tx_ids` in ingest response | `service.ingest_pdf()` with deterministic fallback to `deterministic_tx_id` | Yes | ✓ FLOWING |
| `crates/turbo-mcp/src/bin/turbo-mcp-server.rs` | `proxy_rustledger_ingest_statement_rows` execution path | No handler in `tools/call` | No | ✗ DISCONNECTED |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Contract-level MCP boundary and deterministic schemas | `cargo test -p turbo-mcp --test mcp_adapter_contract -- --nocapture` | `3 passed; 0 failed` | ✓ PASS |
| Transport-only DOC-01/02/03 subprocess verification | `cargo test -p turbo-mcp --test mcp_stdio_e2e -- --nocapture` | `3 passed; 0 failed` | ✓ PASS |
| Reproducible MCP e2e wrapper | `bash scripts/mcp_e2e.sh` | exits 0; `mcp_stdio_e2e` all pass | ✓ PASS |
| Full turbo-mcp regression sanity | `cargo test -p turbo-mcp -- --nocapture` | full package suite passes | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| `DOC-01` | `13-01`, `13-02` | Ingest PDFs through Docling/docling-mcp with provenance | ✓ SATISFIED | MCP ingest callable via `proxy_docling_ingest_pdf`; DOC-01 e2e test passes. |
| `DOC-02` | `13-01`, `13-02` | Deterministic canonical schema mapping | ✓ SATISFIED | Canonical/provenance fields normalized in adapter and asserted in transport tests. |
| `DOC-03` | `13-02` | Replay same source with stable IDs and no duplicates | ✓ SATISFIED | DOC-03 replay test validates `inserted_count` and stable `tx_ids`. |

Requirement ID cross-check:
- Plan frontmatter declares `DOC-01`, `DOC-02`, `DOC-03` ([13-01-PLAN.md](/home/brianh/promptexecution/mbse/l3dg3rr/.planning/phases/13-mcp-boundary-and-agent-only-runtime-surface/13-01-PLAN.md#L14), [13-02-PLAN.md](/home/brianh/promptexecution/mbse/l3dg3rr/.planning/phases/13-mcp-boundary-and-agent-only-runtime-surface/13-02-PLAN.md#L13)).
- `REQUIREMENTS.md` defines all three and maps all three to Phase 13 ([REQUIREMENTS.md](/home/brianh/promptexecution/mbse/l3dg3rr/.planning/REQUIREMENTS.md#L10), [REQUIREMENTS.md](/home/brianh/promptexecution/mbse/l3dg3rr/.planning/REQUIREMENTS.md#L63)).
- Orphaned Phase 13 requirements: none.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| N/A | N/A | No TODO/FIXME/placeholders, empty impl stubs, or console-only handlers detected in phase files scanned. | ℹ️ Info | No blocker anti-patterns detected. |

### Human Verification Required

None required for this automated verification pass.

### Gaps Summary

Phase 13 satisfies DOC-01/02/03 and its transport-level e2e checks, but one planned passthrough boundary contract is still incomplete: `proxy_rustledger_ingest_statement_rows` is advertised as a tool surface yet not executable via `tools/call`. This leaves the rustledger half of the planned proxy model partially implemented and should be closed before treating the phase as fully complete against its own must-have wiring contract.

---

_Verified: 2026-03-29T01:27:58Z_  
_Verifier: Claude (gsd-verifier)_
