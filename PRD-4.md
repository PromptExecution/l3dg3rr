# PRD-4: Ontological Traceability Graph and Phi-4 Internal Model Integration

**Status**: Specification - not yet implemented  
**Target capabilities**: deterministic artifact relationship graph, audit-first visual mapping, host-owned Phi-4 reasoning, typed model job contracts  
**Author**: Codex  
**Date**: 2026-04-27

---

## Problem

`l3dg3rr` already has deterministic financial primitives, an MCP ontology surface, a visual workflow language, and a host-owned local Phi-4 direction. These pieces are not yet unified into one internal language for traceable relationships between data artifacts.

Today, ontology facts can be upserted through MCP, but the ontology is not yet automatically produced from core ledger state. Model integration is also split across host runtime, MCP LLM extraction, verification stubs, and deterministic fallback endpoints. This makes it harder to audit why a workbook row exists, which document evidence supports it, which model call suggested a relationship, and which deterministic validator accepted or rejected that suggestion.

The next product phase must make the ontology graph the system's internal audit language and make Phi-4 a supervised local proposer of typed relationship facts, not an authority that mutates financial truth.

## Finished-State Description

Every material artifact in the bookkeeping pipeline is represented as a typed node in a deterministic relationship graph:

- source documents
- extracted document chunks
- normalized transaction rows
- content-hash transaction IDs
- classification outcomes
- validation issues
- evidence references
- workflow states and tags
- workbook projection rows
- audit log events
- Xero catalog/link entities
- model job proposals and reviews

Every relationship between those artifacts is represented as a typed edge with deterministic identity and provenance. The graph can be rebuilt from canonical state, exported through MCP, visualized in mdBook/Slint, and queried for evidence chains.

Phi-4 runs through the `ledgerr-host` model boundary and produces typed JSON proposals for tasks such as relationship extraction, classification explanation, rule repair, and evidence-chain summarization. Rust validates every proposal before it becomes ontology state.

Visual diagrams are not optional documentation. Each phase must ship with a diagram that shows the exact relationship flow it introduces, and tests must confirm the diagram source remains aligned with the implemented contract.

---

## North Star Visual

```rhai
fn source_document() -> document_record
fn document_record() -> extracted_chunk
fn extracted_chunk() -> normalized_row
fn normalized_row() -> transaction_id
fn transaction_id() -> classification_outcome
fn classification_outcome() -> validation_issue
fn validation_issue() -> evidence_reference
fn evidence_reference() -> workbook_projection
fn workbook_projection() -> audit_event
fn transaction_id() -> xero_link_candidate
fn xero_link_candidate() -> operator_review
fn operator_review() -> committed_ontology_edge
fn phi4_typed_job() -> proposed_ontology_edge
fn proposed_ontology_edge() -> rust_validation_gate
fn rust_validation_gate() -> committed_ontology_edge
fn committed_ontology_edge() -> visual_audit_graph
```

The visual audit graph is the operator-facing proof surface. If an artifact cannot be traced visually back to source evidence and forward to workbook/audit output, the phase is not complete.

---

## Design Principles

- Visual first: every new internal relationship must have a diagram representation before it is considered shippable.
- Deterministic first: Rust-generated IDs, relation kinds, sort order, and snapshots must be stable across runs.
- Model as proposer: Phi-4 can propose categories, edges, summaries, and repairs; Rust owns validation and mutation.
- Host-owned model boundary: no core crate should directly depend on provider-specific clients or raw credentials.
- Workbook remains CPA-facing truth: ontology and sidecar state explain and replay workbook output; they do not replace it.
- Provenance is mandatory: every edge must carry evidence/source metadata or an explicit reason why provenance is unavailable.
- MCP stays compact: expose ontology capabilities through `ledgerr_ontology` actions without expanding the advertised tool catalog.

---

## Phase 1 - Canonical Ontology Core

### Goal

Move the ontology language from an MCP-side implementation detail into a shared deterministic core contract.

### Scope

- Add canonical ontology types to `ledger-core`.
- Define `ArtifactKind`, `RelationKind`, `ArtifactId`, `RelationId`, `ProvenanceRef`, and `OntologySnapshot`.
- Preserve existing entity kinds: document, account, institution, transaction, tax category, evidence reference, Xero contact, Xero bank account, Xero invoice, workflow tag.
- Add artifact kinds for model job, model proposal, workbook row, audit event, validation issue, document chunk, and classification outcome.
- Replace free-form relation strings in new code with typed relation kinds.
- Keep `ledgerr-mcp/src/ontology.rs` as transport/storage adapter until it can be fully migrated.

### Required Visual

```rhai
fn ledger_core_types() -> artifact_kind
fn ledger_core_types() -> relation_kind
fn artifact_kind() -> ontology_snapshot
fn relation_kind() -> ontology_snapshot
fn ontology_snapshot() -> mcp_transport_adapter
fn ontology_snapshot() -> visual_audit_graph
```

### Acceptance Criteria

**AC-4.1.1** - `ledger-core` owns the canonical ontology type definitions and deterministic ID functions.

**AC-4.1.2** - Existing MCP ontology actions continue to accept current payloads, but new internal code maps them into typed core structures.

**AC-4.1.3** - Snapshot serialization is deterministic: repeated builds from equivalent inputs produce byte-identical JSON after formatting.

**AC-4.1.4** - The phase diagram is included in mdBook and renders in both Mermaid and isometric views.

### Required Tests

- `cargo test -p ledger-core ontology_id_is_stable`
- `cargo test -p ledger-core ontology_snapshot_sorting_is_deterministic`
- `cargo test -p ledgerr-mcp ontology_legacy_payload_maps_to_core_types`
- `just docgen-check`

---

## Phase 2 - Automatic Artifact Relationship Emission

### Goal

Make the graph emerge from normal pipeline work instead of relying on manual ontology upserts.

### Scope

- Emit ontology facts during document ingest, PDF ingest, image ingest, row ingest, classification, validation, reconciliation, workbook export, tag changes, and Xero linking.
- Add a small `OntologyEmitter` trait that accepts typed artifact/edge facts without requiring callers to know storage details.
- Keep a no-op emitter for tests and CLI paths that do not need ontology output.
- Persist graph facts next to existing sidecar state where appropriate.

### Required Visual

```rhai
fn ingest_pdf() -> document_artifact
fn ingest_pdf() -> raw_context_artifact
fn document_artifact() -> extracted_row_artifact
fn extracted_row_artifact() -> transaction_artifact
fn transaction_artifact() -> classification_artifact
fn classification_artifact() -> validation_artifact
fn validation_artifact() -> workbook_row_artifact
fn workbook_row_artifact() -> audit_event_artifact
```

### Acceptance Criteria

**AC-4.2.1** - Re-ingesting the same source data does not duplicate ontology nodes or edges.

**AC-4.2.2** - Every inserted transaction has a path from source document or source reference to workbook projection row.

**AC-4.2.3** - Classification and validation edges include confidence and issue metadata.

**AC-4.2.4** - Xero link actions create both document registry links and ontology edges with shared provenance.

**AC-4.2.5** - The operator can export a graph snapshot for a single transaction and inspect it visually.

### Required Tests

- `cargo test -p ledgerr-mcp ingest_pdf_emits_document_to_transaction_edges`
- `cargo test -p ledgerr-mcp row_reingest_does_not_duplicate_ontology_edges`
- `cargo test -p ledgerr-mcp classify_transaction_emits_classification_edge`
- `cargo test -p ledgerr-mcp xero_link_entity_emits_registry_and_ontology_relation`
- `cargo test -p ledgerr-mcp tax_evidence_chain_contract`
- `just mcp-cli-basic`

---

## Phase 3 - Visual Audit Graph as a First-Class Product Surface

### Goal

Make relationship traceability inspectable through the existing visual workflow language and desktop host.

### Scope

- Add an ontology-to-diagram adapter that converts snapshots into the supported Rhai diagram DSL.
- Add graph filters: by transaction ID, document ID, workbook row, tax category, Xero entity, validation issue, and model job.
- Add visual badges for deterministic facts, model proposals, operator approvals, rejected proposals, and missing provenance.
- Keep Mermaid as the canonical 2D reference and isometric view as the operator-friendly exploration surface.
- Add at least one worked audit graph example to mdBook.

### Required Visual

```rhai
fn ontology_snapshot() -> filter_graph
match filter.kind => Transaction -> transaction_evidence_view
match filter.kind => Document -> document_lineage_view
match filter.kind => XeroEntity -> xero_reconciliation_view
match filter.kind => ModelJob -> model_proposal_view
match filter.kind => _ -> full_snapshot_view
fn transaction_evidence_view() -> mermaid_2d
fn transaction_evidence_view() -> isometric_3d
```

### Acceptance Criteria

**AC-4.3.1** - A transaction evidence graph can be rendered from a snapshot without hand-written diagram source.

**AC-4.3.2** - Visual output distinguishes deterministic Rust facts from Phi-4 proposals and operator-approved facts.

**AC-4.3.3** - Missing provenance appears as an explicit warning node, not as absent UI.

**AC-4.3.4** - The same parsed graph drives both Mermaid and isometric rendering.

**AC-4.3.5** - The visual graph has stable node ordering across repeated renders.

### Required Tests

- `cargo test -p ledger-core ontology_snapshot_to_rhai_dsl_is_deterministic`
- `cargo test -p mdbook-rhai-mermaid ontology_audit_graph_renders_match_nodes`
- `npm test --prefix book/theme`
- `just docgen-check`
- Playwright or equivalent screenshot test for desktop-width and mobile-width graph rendering once the visual route is browser-served.

---

## Phase 4 - Host-Owned Phi-4 Typed Job Runtime

### Goal

Unify internal model calls behind the host runtime and make Phi-4 produce validated typed outputs.

### Scope

- Define typed model jobs:
  - `ClassifyTransactionJob`
  - `ProposeOntologyEdgesJob`
  - `ExplainEvidenceChainJob`
  - `RepairRuleCandidateJob`
  - `SummarizeAuditPathJob`
- Route these through `ledgerr-host/src/agent_runtime.rs`.
- Prefer `phi-4-mini-reasoning` at `http://127.0.0.1:15115/v1/chat/completions` for local mode.
- Keep deterministic fallback behavior for tests and no-model environments.
- Record model-call metadata through `ModelAuditSink` without raw prompt or response body.
- Migrate or wrap `ledgerr-llm` so extraction/classification uses the same audit and settings path.

### Required Visual

```rhai
fn typed_model_job() -> host_agent_runtime
fn host_agent_runtime() -> phi4_local_endpoint
fn phi4_local_endpoint() -> structured_json_response
fn structured_json_response() -> schema_validation
fn schema_validation() -> invariant_validation
fn invariant_validation() -> ontology_proposal
fn ontology_proposal() -> operator_or_policy_gate
fn operator_or_policy_gate() -> committed_ontology_edge
```

### Acceptance Criteria

**AC-4.4.1** - Core pipeline code depends on a model trait, not a provider-specific HTTP client.

**AC-4.4.2** - Every typed job has a JSON schema, a parser test, and a validator test.

**AC-4.4.3** - Invalid model output fails closed and produces an audit event.

**AC-4.4.4** - The local Phi-4 endpoint and deterministic fallback both satisfy the same typed job contract.

**AC-4.4.5** - Prompt and response content are not written to audit logs by default.

### Required Tests

- `cargo test -p ledgerr-host agent_runtime_parses_typed_phi4_job`
- `cargo test -p ledgerr-host model_audit_records_metadata_without_content`
- `cargo test -p ledgerr-host internal_phi_endpoint_satisfies_typed_job_contract`
- `cargo test -p ledger-core verify_phi4_profile_uses_local_model_defaults`
- `just test-phi4` when model assets are available
- `just test-phi4-mistral` when model assets and native build prerequisites are available

---

## Phase 5 - Proposal Review, Approval, and Deterministic Commit

### Goal

Let Phi-4 enrich the ontology without bypassing financial invariants, operator policy, or auditability.

### Scope

- Add proposal state: proposed, validated, rejected, approved, committed.
- Require validation before an edge can be committed.
- Require operator approval for ambiguous, low-confidence, credential-adjacent, Xero-mutating, or workbook-mutating proposals.
- Add reviewer model support through the existing multi-model verification loop, with Phi-4 as the default local proposer.
- Preserve rejected proposals as audit artifacts when useful for review, without polluting committed relationship paths.

### Required Visual

```rhai
fn phi4_proposal() -> parse_typed_output
fn parse_typed_output() -> rust_invariant_check
if confidence >= 0.90 -> policy_gate
if confidence < 0.90 -> operator_review
fn policy_gate() -> committed_edge
fn operator_review() -> approved_edge
fn operator_review() -> rejected_proposal
fn approved_edge() -> committed_edge
fn rejected_proposal() -> audit_event
```

### Acceptance Criteria

**AC-4.5.1** - No model proposal can directly mutate workbook, journal, credential, or Xero state.

**AC-4.5.2** - Rejected proposals remain queryable by audit tools but are excluded from committed evidence chains by default.

**AC-4.5.3** - Approved proposals include actor, timestamp, model metadata, validation result, and source artifact IDs.

**AC-4.5.4** - Confidence thresholds are configurable but have safe defaults.

**AC-4.5.5** - The approval path is visible in the graph.

### Required Tests

- `cargo test -p ledger-core model_proposal_cannot_commit_without_validation`
- `cargo test -p ledgerr-mcp rejected_proposal_excluded_from_default_evidence_chain`
- `cargo test -p ledgerr-mcp approved_proposal_preserves_model_and_actor_metadata`
- `cargo test -p ledger-core low_confidence_proposal_requires_operator_review`
- `just mcp-cli-spinning-wheels`

---

## Phase 6 - Semantic Retrieval and Rule Selection

### Goal

Use local model-powered retrieval to improve classification and evidence matching while keeping deterministic rules authoritative.

### Scope

- Add embeddings for transaction descriptions, document chunks, rule descriptions, prior classifications, and Xero catalog entities.
- Build a local vector index for candidate retrieval.
- Feed retrieved candidates into Phi-4 typed jobs as context.
- Keep Rhai waterfall and Rust validators as final authority.
- Record which retrieved candidates influenced a model proposal.

### Required Visual

```rhai
fn document_chunk() -> embedding_record
fn transaction_description() -> embedding_record
fn rule_registry() -> embedding_record
fn embedding_record() -> local_vector_index
fn local_vector_index() -> candidate_context
fn candidate_context() -> phi4_typed_job
fn phi4_typed_job() -> validated_classification
fn validated_classification() -> ontology_edge
```

### Acceptance Criteria

**AC-4.6.1** - Retrieval is local-first and does not require cloud services.

**AC-4.6.2** - The semantic selector never bypasses Rhai classification invariants or validation gates.

**AC-4.6.3** - Candidate context is traceable: retrieved item IDs are attached to proposal provenance.

**AC-4.6.4** - Rebuilding the index from the same inputs yields stable item IDs and stable top-k ordering for equal scores.

### Required Tests

- `cargo test -p ledger-core semantic_candidate_ids_are_stable`
- `cargo test -p ledger-core semantic_selector_preserves_deterministic_fallback`
- `cargo test -p ledgerr-mcp semantic_context_refs_are_added_to_model_provenance`
- `cargo test -p ledger-core rule_registry_waterfall_remains_authoritative`

---

## Phase 7 - End-to-End Audit Playbook

### Goal

Ship a complete operator-facing audit path from document ingest to visual evidence graph to workbook export.

### Scope

- Add a sample dataset that exercises document ingest, row normalization, classification, validation issue, Xero candidate link, model proposal, approval/rejection, workbook projection, and audit graph export.
- Add mdBook playbook chapter for the scenario.
- Add MCP CLI flow for both happy path and blocked diagnostic path.
- Add Slint host route or local docs route that opens the visual graph for the sample transaction.

### Required Visual

```rhai
fn sample_statement() -> ingest_rows
fn ingest_rows() -> classify_transactions
fn classify_transactions() -> phi4_edge_proposals
fn phi4_edge_proposals() -> operator_review
fn operator_review() -> workbook_export
fn workbook_export() -> evidence_chain
fn evidence_chain() -> visual_audit_graph
fn visual_audit_graph() -> cpa_review
```

### Acceptance Criteria

**AC-4.7.1** - A fresh checkout can run the basic audit playbook without a real model by using deterministic fallback responses.

**AC-4.7.2** - The same playbook can run with real Phi-4 when model assets are present.

**AC-4.7.3** - The output workbook, audit log, ontology snapshot, and visual graph all refer to the same transaction IDs.

**AC-4.7.4** - The blocked diagnostic path demonstrates a failed or low-confidence proposal and shows how the operator resolves it.

**AC-4.7.5** - Documentation links the playbook to the MCP surface, ontology type mesh, workflow, and workbook audit chapters.

### Required Tests

- `just test`
- `just mcp-cli-basic`
- `just mcp-cli-spinning-wheels`
- `just docgen-check`
- `cargo test -p ledgerr-mcp audit_playbook_ids_match_across_workbook_ontology_and_events`
- `cargo test -p ledgerr-host internal_phi_fallback_runs_audit_playbook_prompt`

---

## Cross-Phase Test Gates

Every phase must complete these checks before being considered done:

- Unit tests for newly introduced types and validators.
- Integration test proving deterministic replay for the touched artifact path.
- MCP contract test if any transport payload changes.
- Visual diagram test or docgen test for every new relationship flow.
- Audit test proving provenance is present or an explicit missing-provenance warning is emitted.
- `just docgen-check` when docs or visual DSL examples change.
- `just test` before merging implementation work.

When model behavior is involved, both paths must be tested:

- deterministic fallback path, always required in CI;
- real Phi-4 path, required on machines with local model assets.

---

## Non-Goals

- Replacing Excel as the CPA-facing artifact.
- Giving Phi-4 direct write access to workbook, journal, credentials, or Xero.
- Expanding the default MCP catalog beyond the collapsed `ledgerr_*` capability families.
- Storing raw prompts or raw model responses in audit logs by default.
- Requiring cloud services for the core audit path.
- Building a generic knowledge graph product outside bookkeeping and audit traceability.

---

## Open Questions

- Should the canonical ontology snapshot live beside the manifest workbook, inside the existing deterministic sidecar, or both?
- Should model proposals be retained forever as audit artifacts, or pruned after workbook export with a digest retained?
- Which local embedding backend should be preferred for Phase 6: fastembed/ONNX, Phi-derived embeddings, or a simpler lexical baseline first?
- Should Slint show the graph through native Rust rendering, the existing docs browser route, or both?
- Which ontology relations should be hard-coded typed enums immediately, and which should remain extension points for imported Xero/accounting concepts?

