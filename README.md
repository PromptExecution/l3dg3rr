# l3dg3rr

[![CI](https://github.com/PromptExecution/l3dg3rr/actions/workflows/ci.yml/badge.svg)](https://github.com/PromptExecution/l3dg3rr/actions/workflows/ci.yml)
[![Release](https://github.com/PromptExecution/l3dg3rr/actions/workflows/release.yml/badge.svg)](https://github.com/PromptExecution/l3dg3rr/actions/workflows/release.yml)
[![Documentation](https://img.shields.io/badge/docs-github.io-blue)](https://promptexecution.github.io/l3dg3rr/)

`l3dg3rr` is a local-first bookkeeping application for turning financial documents into an accountant-usable, CPA-auditable source of truth. Its core shape is a strongly typed, ontologically linked graph of scriptable, visual-first workflows: Rust owns financial invariants, Rhai owns editable classification and workflow rules, and AI/LLM agents drive supervised ETL through MCP tools without taking custody of credentials or approval authority.

Read the live book: <https://promptexecution.github.io/l3dg3rr/>

**Primary bookkeeping outcome:** ingest raw historical statements, classify and reconcile transactions, preserve evidence and mutation history, then export a CPA-reviewable Excel workbook without requiring private data to leave the local machine.

## System Thesis

The project is intentionally not just a PDF parser, a rules folder, or a visualization experiment. Those are subsystems of one bookkeeping control plane:

```rhai
fn source_documents() -> typed_document_graph
fn typed_document_graph() -> extraction_and_normalization
fn extraction_and_normalization() -> transaction_classification
fn transaction_classification() -> validation_and_legal_checks
fn validation_and_legal_checks() -> reconciliation
fn reconciliation() -> workbook_export
fn workbook_export() -> cpa_review
fn cpa_review() -> audit_history
```

The workbook remains the human and accounting interface. The graph, sidecar state, Rhai rules, MCP tools, and visualization layers exist to make that workbook reproducible, explainable, and agent-accessible.

## Design Lens: TRIZ + MECE

`l3dg3rr` resolves recurring product contradictions by separating concerns instead of adding one-off glue:

| Contradiction | Resolution |
|---|---|
| Accountant-readable vs machine-replayable | Excel is the CPA-facing artifact; journal and sidecar state preserve deterministic replay. |
| Runtime-editable rules vs financial correctness | Rhai handles heuristics at controlled boundaries; Rust types enforce money, identity, validation, and workbook contracts. |
| Agent autonomy vs operator control | MCP exposes capability families; host policy, audit, approvals, notifications, and credentials remain owned by `l3dg3rr`. |
| Rich visual workflows vs stable execution | A narrow Rhai diagram DSL renders Mermaid and isometric views while typed Rust workflow structs own execution contracts. |
| Xero integration vs local-first privacy | Xero is a supervised capability reached through worker tools and reconciled evidence, not raw credential leakage to a model. |

MECE module grouping keeps the logic approachable:

| Layer | Responsibility | Primary files |
|---|---|---|
| Bookkeeping truth | ingest, journal, workbook projection, audit output | `crates/ledger-core/src/ingest.rs`, `journal.rs`, `workbook.rs` |
| Typed domain model | documents, transactions, accounts, tax categories, validation state | `document.rs`, `classify.rs`, `validation.rs`, `legal.rs` |
| Ontology graph | typed links between documents, accounts, transactions, evidence, Xero entities | `crates/ledgerr-mcp/src/ontology.rs`, `crates/ledger-core/src/graph.rs`, `book/src/ontology-type-mesh.md` |
| Scriptable policy | editable Rhai classification and document-shape rules | `rules/`, `classify.rs`, `rule_registry.rs` |
| Workflow control | pipeline state, scheduled operations, approval/reversibility metadata | `pipeline.rs`, `workflow.rs`, `ledger_ops.rs`, `calendar.rs` |
| Visualization | Mermaid, isometric docs renderer, live Rhai editor | `crates/mdbook-rhai-mermaid/`, `book/theme/rhai-live-core.js`, `visualize.rs` |
| Agent boundary | published MCP capability families and deterministic argument contracts | `crates/ledgerr-mcp/src/contract.rs`, `mcp_adapter.rs`, `docs/mcp-capability-contract.md` |
| Operator host | desktop settings, notifications, local chat endpoint, tray/window control | `crates/ledgerr-host/src/` |

## Bookkeeping Flow

1. **Ingest**: accept statement files named `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE.ext`, infer document shape, and normalize rows.
2. **Identify**: generate deterministic Blake3 transaction IDs from account/date/amount/description so re-ingest is idempotent.
3. **Classify**: run a deterministic Rhai rule waterfall over each transaction and flag low-confidence results.
4. **Validate**: compound stage confidence, attach typed issues, and separate unrecoverable, recoverable, and advisory dispositions.
5. **Reconcile**: compare local facts against Xero or other external evidence through supervised capability tools.
6. **Export**: rebuild the CPA workbook projection with transactions, schedule summaries, flags, and `AUDIT.log` history.
7. **Review**: keep human/CPA signoff in Excel while preserving agent-visible evidence paths.

## Rhai Rules And Match/Switch Visualization

There are three related but distinct Rhai surfaces:

| Surface | Purpose | Supported shape |
|---|---|---|
| Transaction rules | Runtime tax/bookkeeping classification | `fn classify(tx) -> #{ category, confidence, review, reason }` |
| Workflow compiler output | Runtime state transition function | generated Rhai `switch [state, event.kind] { ... }` from `WorkflowToml` |
| Documentation diagram DSL | Visual workflow blocks in mdBook and live editor | `fn source() -> target`, `if expression -> target`, `match expr => Arm -> target` |

The match operator discussed in the docs is a switch-like visualization idiom for branch-heavy workflows. Repeated arms with the same expression collapse into one semantic match node:

```rhai
fn verify_result() -> match_result_disposition
match result.disposition => Disposition::Unrecoverable -> halt_pipeline
match result.disposition => Disposition::Recoverable -> repair_and_retry
match result.disposition => Disposition::Advisory -> record_note
match result.disposition => _ -> operator_review
fn repair_and_retry() -> requeue_validation
```

Current behavior:

| Behavior | Status |
|---|---|
| One match node per expression | Implemented in `crates/mdbook-rhai-mermaid/src/parser.rs` and mirrored in `book/theme/rhai-live-core.js` |
| Declaration-ordered arms | Implemented with `IndexMap` in Rust and ordered `Map` behavior in JS |
| Labeled outgoing edges | Implemented in Mermaid output and live editor previews |
| Default arm detection (`_`, `else`, `otherwise`, `default`) | Implemented with visual default annotation |
| Isometric lane assignment for arms | Implemented in the live docs renderer |
| Rich explicit rejoin semantics and animated lane reflow | Planned / in progress; see the match plan chapter |

Deep references:

- Live docs: <https://promptexecution.github.io/l3dg3rr/>
- [Match Visualization Plan](book/src/match-visualization-plan.md)
- [Visualization](book/src/visualize.md)
- [Workflow](book/src/workflow.md)
- [Ontology & Type Mesh](book/src/ontology-type-mesh.md)

## Current Capability Snapshot

See [Capability Map](book/src/capability-map.md) for the full component table.

| Capability | Status | Notes |
|---|---|---|
| Filename convention parser | Implemented | `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE.ext` routing |
| Blake3 transaction identity | Implemented | idempotent transaction IDs |
| Journal and workbook ingest projection | Implemented | workbook remains projection/audit artifact |
| Rhai classification engine | Implemented | strict output schema, review flags |
| Rule registry and deterministic waterfall | Implemented | semantic selector is still planned |
| Document shape classifier | Implemented | vendor/format inference for bank statements and CSVs |
| Business calendar | Implemented | US/AU tax defaults and recurring events |
| Validation disposition model | Implemented | unrecoverable/recoverable/advisory issue handling |
| Legal solver path | Partial | AU GST and US Schedule C hard predicates; native Z3 behind `ledger-core/legal-z3` |
| Workflow TOML compiler | Implemented | Mermaid, Rhai FSM, Rust enum generation |
| mdBook Rhai-to-Mermaid preprocessor | Implemented | supports `fn`, `if`, and `match` diagram DSL lines |
| Live Rhai docs editor | Implemented | synchronized isometric and Mermaid views |
| Xero capability family | In flight | supervised MCP capability, not raw credential exposure |
| Tauri desktop host | Active | primary operator host (replaces legacy Slint surface) |
| Slint desktop host | Legacy | fallback window, settings, local endpoint, notifications |
| Evidence traceability (arc-kit-au) | Implemented | petgraph-backed provenance graph with deterministic node identity |
| Docling extraction bridge | Missing | planned local extraction sidecar |
| File watcher | Missing | `notify` not yet wired as an end-to-end inbox loop |

## Documentation Map

The published book is the detailed reference and should be preferred over expanding the README indefinitely:

- <https://promptexecution.github.io/l3dg3rr/>: live GitHub Pages book
- [Introduction](book/src/intro.md): product guarantees and primary surfaces
- [MCP Surface](book/src/mcp-surface.md): published `ledgerr_*` tool families
- [Document Ingestion](book/src/document-ingestion.md): source routing and extraction assumptions
- [Rule Engine](book/src/rule-engine.md): Rhai classification model
- [Workbook & Audit](book/src/workbook-audit.md): CPA-facing artifact contract
- [Visualization](book/src/visualize.md): live editor and visual idioms
- [Match Visualization Plan](book/src/match-visualization-plan.md): match/switch branch contract

Build and validate docs locally:

```bash
just docgen
just docgen-check
just docserve
```

## Developer Quick Start

Prerequisites:

| Tool | Purpose |
|---|---|
| Rust 1.88+ | workspace build/test |
| `just` | canonical command runner |
| `mdbook` + `mdbook-mermaid` | book generation |
| `mdbook-rhai-mermaid` | Rhai diagram preprocessor |
| `cocogitto` | conventional versioning and changelog automation |

Common recipes:

```bash
# Run the full test suite plus MCP outcome smoke path.
just test

# Build the book into book/book/.
just docgen

# Validate diagrams, links, and live Rhai JS tests.
just docgen-check

# Serve the book locally with live Rhai editing.
just docserve

# Start the stdio MCP server.
just mcp-start
```

Use `Justfile` as the executable workflow contract. When a command changes, update the recipe first and reference the recipe name from docs.

## Agent And MCP Guide

- [AGENTS.md](AGENTS.md): agent-facing operating rules, product constraints, and persistent session guidance.
- [docs/mcp-capability-contract.md](docs/mcp-capability-contract.md): MCP tool matrix, argument contracts, service mappings, and usage flow.
- `crates/ledgerr-mcp/src/contract.rs`: source of truth for the published MCP surface.

The default MCP catalog should stay collapsed to the top-level `ledgerr_*` capability families. Add sub-operations through required `action` parameters instead of expanding the default tool list.

## Release and Versioning Policy

l3dg3rr follows an **odd/even minor version** convention, similar to the Ubuntu LTS model.

| Minor version | Series | Characteristics |
|---|---|---|
| Even (`1.0`, `1.2`, `1.4`, `1.8`, …) | **Stable** | Long-term supported. Full test gate including local Phi-4 model-inference tests. GitHub release published. Suitable for production operator use. |
| Odd (`1.1`, `1.3`, `1.5`, `1.7`, …) | **Dev / Experimental** | Fast-moving. Breaking changes within a major series are permitted. Model-inference tests may be skipped. No GitHub release created. No LTS support. |

### Release commands

```sh
# Stable release (even minor) — full test gate including phi4 inference, GitHub release created
just release minor   # or: just release major / just release patch

# Fast test gate only (excludes phi4 GGUF inference, ~seconds)
just test-fast
```

### What the `release` recipe does

1. Detects the next version's minor parity and selects the appropriate test gate:
   - **Even minor (stable)** — full `cargo test` suite including phi4 GGUF inference
   - **Odd minor (dev)** — fast gate only (`--skip phi4_produces_output --skip phi4_mistral_produces_output`)
2. Runs `./scripts/e2e_mvp.sh` end-to-end smoke path
3. Calls `cog bump --<version>` — sets version in all `Cargo.toml` files, creates a conventional-commit bump commit and a semver git tag
4. Pushes branch and tags to origin with `git push --follow-tags`
5. **Even minor** — creates a stable GitHub release (`gh release create --latest`)
6. **Odd minor** — skips GitHub release; CI creates a pre-release from the pushed tag

Pushing the tag triggers `.github/workflows/docs.yml`, which redeploys GitHub Pages regardless of minor parity.

## Docker

```bash
docker build -t l3dg3rr:dev .
docker run --rm -i \
  -v "$PWD/data:/data" \
  l3dg3rr:dev
```

The container runs `ledgerr-mcp-server` over stdio. Mount `/data` for local workbook and document inputs.
