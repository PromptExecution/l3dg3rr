# Capability Map

This chapter provides a complete status map of the l3dg3rr system — every major capability, its current implementation state, and what is needed to complete it.

## System Architecture Diagram

```rhai
fn filename_routing() -> blake3_ids
fn filename_routing() -> docling_bridge
fn docling_bridge() -> document_graph
fn document_graph() -> reqif_candidates
fn reqif_candidates() -> rule_registry
fn rule_files() -> rule_registry
fn rule_registry() -> keyword_selector
fn rule_registry() -> semantic_selector
fn keyword_selector() -> classify_waterfall
fn semantic_selector() -> classify_waterfall
fn classify_waterfall() -> classification_engine
fn classification_engine() -> review_flags
fn classification_engine() -> legal_solver
fn legal_solver() -> workbook_output
fn review_flags() -> workbook_output
fn workbook_output() -> audit_trail
fn audit_trail() -> sidecar_state
fn workflow_toml() -> mermaid_generation
fn pipeline_hsm() -> agent_runtime
fn issue_source() -> agent_runtime
fn reqif_opa_bridge() -> agent_runtime
fn xero_catalog() -> reconciliation_candidates
fn ledgerr_mcp_contract() -> agent_runtime
```

## Component Status Table

| Component | Module | Status | Notes |
|---|---|---|---|
| Filename routing | `filename.rs` | Implemented | `VENDOR--ACCT--YYYY-MM--DOCTYPE` parser |
| Blake3 hash IDs | `ingest.rs` | Implemented | `deterministic_tx_id`, idempotent dedup |
| IngestedLedger | `ingest.rs` | Implemented | Journal + workbook ingest pipeline |
| DocType enum | `document.rs` | Implemented | Document type classification |
| DocumentGraph types | `document.rs` | Implemented | Graph node/edge types defined |
| Pipeline HSM | `pipeline.rs` | Implemented | Type-state + statig state machine |
| Verb trait | `pipeline.rs` | Implemented | DetectVerb, ValidateVerb |
| ClassificationEngine | `classify.rs` | Implemented | Rhai rule execution |
| ClassificationOutcome | `classify.rs` | Implemented | category, confidence, reason |
| ReviewFlag | `classify.rs` | Implemented | Flag upsert, query by year/status |
| Rhai rule files | `rules/` | Implemented | foreign_income, self_employment, fallback |
| Jurisdiction enum | `legal.rs` | Implemented | US, AU, UK |
| LegalRule + Z3 formulas | `legal.rs` | Implemented | Hard predicate checks for AU GST and US Schedule C |
| LegalSolver | `legal.rs` | Implemented | Uses pinned `z3 = 0.8` for violation satisfiability checks behind `legal-z3`; default builds use the same deterministic result semantics without native Z3 |
| Proposer/Reviewer LLM | `verify.rs` | Partial | Pattern defined, no real LLM calls |
| WorkflowToml DSL | `workflow.rs` | Implemented | TOML → Rhai FSM + Mermaid |
| IssueSource::RhaiRule | `validation.rs` | Implemented | Validation layer with rule source |
| Workbook write | `workbook.rs` | Implemented | `rust_xlsxwriter` tx projection |
| Workbook read-back | `workbook.rs` | Implemented | `calamine` round-trip |
| Journal | `journal.rs` | Implemented | NDJSON append and replay |
| Audit trail (MetaCtx) | `pipeline.rs` | Implemented | Mutation log per pipeline state |
| MCP contract | `ledgerr-mcp/src/contract.rs` | Implemented | 8 advertised `ledgerr_*` capability families |
| MCP adapter | `ledgerr-mcp/src/mcp_adapter.rs` | Implemented | Dispatches contract actions to `TurboLedgerService` |
| Ontology store | `ledgerr-mcp/src/ontology.rs` | Implemented | Entity/edge upsert and path query surface |
| Xero service | `ledgerr-mcp/src/xero_service.rs` | Partial | Supervised catalog/link actions; credentials remain host-owned |
| Mermaid auto-generation | `workflow.rs` | Implemented | rhai DSL → diagram blocks |
| Slint desktop UI | `slint_viz.rs` | Partial | Stub, not wired to window system |
| RuleRegistry | `rule_registry.rs` | Implemented | Loads transaction `.rhai` rules and optional ReqIF sidecars |
| Keyword rule selection | `rule_registry.rs` | Implemented | Deterministic keyword fallback; semantic selector remains planned |
| Waterfall orchestration | `rule_registry.rs` | Implemented | First non-`Unclassified` result wins; fallback outcome is preserved |
| ReqIfCandidate (Rust) | `rule_registry.rs` | Stub | Type defined, sidecar bridge missing |
| DocumentChunk | `rule_registry.rs` | Stub | Type defined, bridge missing |
| SemanticRuleSelector | `rule_registry.rs` | Stub | Trait defined, embeddings not wired |
| Docling extraction bridge | — | Missing | Python subprocess call not written |
| reqif-opa-mcp MCP wiring | — | Missing | No Rust MCP client for sidecar |
| Vector embedding index | — | Missing | No embedding model or HNSW index |
| File watcher (notify) | — | Missing | `notify` crate not yet wired |

## North Star Pipeline (Rhai DSL)

The following DSL block describes the full intended end-to-end system flow — the "north star" that all stub work is building toward.

```rhai
fn document_ingest() -> reqif_extract
fn reqif_extract() -> opa_gate
fn opa_gate() -> rule_registry
fn rule_registry() -> classify_waterfall
fn classify_waterfall() -> legal_verify
fn legal_verify() -> workbook_commit
fn workbook_commit() -> audit_trail
```

Each step in this pipeline has a corresponding Rust type or trait. The deterministic rule-registry waterfall is now implemented; the remaining ingestion-side gap is the `reqif-opa-mcp` bridge and semantic rule-selection infrastructure. Z3 is wired for the first hard legal predicates; broader solver coverage is still a roadmap item.

## Next Steps

The five highest-value missing capabilities to implement, in priority order:

1. **Docling extraction bridge** — Write the Rust `std::process::Command` call to invoke the Python sidecar, parse its NDJSON stdout, and deserialize into `DocumentChunk` / `ReqIfCandidate`. This is the critical path for Phase 2 document intelligence. Estimated scope: new `sidecar.rs` module, ~100 lines.

2. **Wire `ClassifyTransactionsOp` to `RuleRegistry`** — Replace the current operation stub with registry loading, transaction iteration, waterfall classification, and review-flag emission.

3. **Expand `LegalSolver` coverage** — Add hard Z3 checks for FBAR/FATCA thresholds, mutually exclusive categories, and reconciliation/workbook invariants. The initial Z3 integration covers AU GST and US Schedule C predicates.

4. **File watcher via `notify`** — Add a debounced `notify` watcher on the workbook path and the `rules/` directory. This enables live rule reloading and human Excel-edit detection without polling. Estimated scope: new `watcher.rs` module, ~60 lines.

5. **`SemanticRuleSelector` embedding index** — Wire a local `fastembed-rs` or ONNX embedding model to encode transaction descriptions and `ReqIfCandidate` texts. Build an HNSW index over candidate embeddings for cosine-similarity rule selection. Estimated scope: ~200 lines, depends on item 2.
