# Phase 14: Ontology Persistence and Query Surface - Research

**Researched:** 2026-03-29  
**Domain:** Ontology persistence/query/serialization for agent-facing FDKMS  
**Confidence:** Medium-High

## Summary

Phase 14 should introduce a deterministic ontology projection layer above current MCP ingest flows without replacing upstream rustledger/docling interfaces. The safest incremental architecture is:
1. define a strict ontology schema for entities + relations + provenance,
2. persist ontology records locally in a git-compatible form,
3. expose MCP query/serialization tools that return stable machine-readable payloads.

This phase should not implement reconciliation gates, HSM transition control, or event-sourcing replay logic; those remain in Phases 15-17.

## Requirements Mapping

| Requirement | Implementation Direction |
|-------------|--------------------------|
| ONTO-01 | Add ontology entity store with referential integrity checks on insert/update. |
| ONTO-02 | Add relationship query endpoints (document -> extracted tx -> reconciliation placeholder -> tax treatment placeholder). |
| ONTO-03 | Add deterministic serialization output (stable ordering, explicit IDs, explicit edge types) for MCP consumers. |

## Recommended Architecture

### Data Model
- `OntologyEntity`: `id`, `kind`, `attrs`, `source_refs`
- `OntologyEdge`: `id`, `from`, `to`, `relation`, `provenance`
- `OntologySnapshot`: ordered entity/edge collections for serialization output

### Persistence
- Use local file-backed store (JSONL or compact structured files) with deterministic sort/ID generation.
- Keep IDs content-hash-based where possible to align with existing deterministic behavior.
- Enforce referential checks (`from`/`to` existence) before edge commit.

### MCP Surface
- `ontology_upsert_entities`
- `ontology_upsert_edges`
- `ontology_query_path`
- `ontology_export_snapshot`

All responses should include concise status fields and deterministic ordering.

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Over-coupling ontology with reconciliation/HSM/event phases | Scope spill and slow delivery | Keep placeholders for later lifecycle states, avoid enforcing downstream semantics now. |
| Non-deterministic output ordering | Agent instability, flaky tests | Canonical sort by `(kind,id)` and `(relation,from,to,id)` before response/serialization. |
| Interface drift from Phase 13 MCP boundary | Regression in agent-only workflows | Route ontology operations through existing MCP adapter/server pattern and add transport tests. |

## Validation Architecture

| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` integration + contract tests |
| Quick run | `cargo test -p turbo-mcp --test ontology_contract -- --nocapture` |
| Full suite | `cargo test --workspace -- --nocapture` |

Per-requirement checks:
- ONTO-01: persistence integrity tests (`missing_ref` rejects, valid graph persists)
- ONTO-02: query traversal tests across entity/edge chains
- ONTO-03: stable serialization snapshot test with fixed expected output

Wave-0 expected test files:
- `crates/turbo-mcp/tests/ontology_contract.rs`
- `crates/turbo-mcp/tests/ontology_mcp_e2e.rs`

## Proposed Plan Shape

Two-wave execution is sufficient:
- Wave 1: ontology data model + persistence integrity + contract tests
- Wave 2: MCP query/serialization tools + transport e2e + docs

---

*Phase: 14-ontology-persistence-and-query-surface*  
*Research date: 2026-03-29*
