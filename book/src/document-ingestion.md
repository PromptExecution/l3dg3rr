# Document Ingestion

Document ingestion converts raw PDF statements into structured, auditable transaction records. The process is split into two phases: deterministic ingestion (fully implemented) and rule derivation from document content (partially implemented via stubs).

## Two-Phase Ingestion Model

**Phase 1 — Deterministic Ingestion** takes a PDF, extracts transactions using Blake3 content-hash IDs for deduplication, and writes raw records to the workbook and journal. This phase is fully implemented and produces stable, reproducible output regardless of processing order.

**Phase 2 — Rule Derivation** uses the extracted document structure to derive applicable classification rules. This requires the `reqif-opa-mcp` Python sidecar to build a `DocumentGraph` from the PDF, run it through an OPA policy gate, and produce `RequirementCandidate` objects. The Rust side then deserializes these into `ReqIfCandidate` structs and uses them to seed the `RuleRegistry`. This phase is currently stubbed.

## Ingestion Pipeline Diagram

```rhai
fn pdf_statement() -> filename_routing
fn filename_routing() -> blake3_ingest
fn filename_routing() -> docling_extract
fn docling_extract() -> document_graph
fn document_graph() -> requirement_candidates
fn requirement_candidates() -> opa_gate
fn opa_gate() -> reqif_baseline
fn reqif_baseline() -> rule_registry
fn rule_registry() -> classify_waterfall
fn classify_waterfall() -> workbook_commit
```

## Ingestion Flow (Rhai DSL)

The following DSL block describes the intended end-to-end ingestion flow. Each step corresponds to a node in the pipeline diagram above.

```rhai
fn ingest_artifact() -> extract_document
fn extract_document() -> build_graph
fn build_graph() -> derive_candidates
fn derive_candidates() -> opa_gate
fn opa_gate() -> emit_reqif
fn emit_reqif() -> load_rules
```

## Capability Status Table

| Capability | Status | Implementation | Notes |
|---|---|---|---|
| `ingest_artifact` | Implemented | `IngestedLedger::ingest` | Blake3 hash dedup, journal write |
| `extract_document` | Planned | Python sidecar: `extract_docling_document` | Docling 2.78 PDF parse |
| `build_graph` | Planned | Python sidecar: `DocumentGraph` assembly | `DocumentNode` tree construction |
| `derive_candidates` | Planned | Python sidecar: `RequirementCandidate` | Heuristic requirement extraction |
| `opa_gate` | Planned | Python sidecar: OPA policy evaluation | Policy: admit or reject candidates |
| `emit_reqif` | Planned | Python sidecar: `emit_reqif_xml` | ReqIF XML baseline output |
| `load_rules` | Stub | `RuleRegistry::load_from_dir` | NDJSON deserialization into `ReqIfCandidate` |

## ReqIF-OPA-MCP Integration

The Python sidecar at <https://github.com/PromptExecution/reqif-opa-mcp> implements the full `ArtifactRecord → DocumentGraph → RequirementCandidate → OPA → ReqIF` pipeline. It is invoked as a subprocess by the Rust core and communicates via NDJSON over stdout.

Key Python types and their Rust mirrors:

| Python (sidecar) | Rust (ledger-core) | Notes |
|---|---|---|
| `ArtifactRecord` | — | Source document metadata; consumed by sidecar only |
| `DocumentNode` | `DocumentChunk` | Canonical graph node with text, parent, anchors |
| `DocumentGraph` | — | Intermediate; not materialized in Rust |
| `RequirementCandidate` | `ReqIfCandidate` | Deserialized from sidecar NDJSON output |

The sidecar also exposes an MCP server interface for querying existing ReqIF baselines and evaluating new candidates against established policy. This allows the agent to ask "does this transaction match any known requirement?" before falling back to keyword-match rule selection.

The Rust bridge will call the sidecar via `std::process::Command`, read its NDJSON output, and deserialize each line into a `ReqIfCandidate`. These candidates are then stored in the `RuleRegistry` alongside their associated rule files.

## Vector Search Stub

`SemanticRuleSelector` (defined in `crates/ledger-core/src/rule_registry.rs`) will eventually use vector embeddings to rank rule files by semantic similarity to each transaction's description. The embedding space is anchored to `ReqIfCandidate` text so that rules are selected based on document-derived semantics rather than string keywords.

The interface is already defined:

```rust
pub trait SemanticRuleSelector {
    fn select_rules_semantic(&self, tx: &SampleTransaction, top_k: usize) -> Vec<PathBuf>;
    fn build_embedding_index(&mut self) -> Result<(), RuleRegistryError>;
}
```

Implementation is blocked on:

1. A local embedding model (ONNX via `ort`, or `fastembed-rs`)
2. A vector index (`usearch`, `hnsw`, or `qdrant` sidecar)
3. `ReqIfCandidate` objects being populated from the sidecar bridge

Until these are available, `select_rules_deterministic` provides a stable keyword-match fallback. See [Rule Engine](./rule-engine.md) for the full classification pipeline.
