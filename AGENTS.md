## Agent Quickstart (Read This First)

This file is the persistent operator manual for future agents.  
For product scope and status, read `README.md` first, then use this file for execution rules and MCP usage patterns.

### Current Direction (2026-04-18)

Near-term product direction is shifting from pure MCP/server work toward a local desktop operator host around `l3dg3rr`.

Current planning assumptions:
- `l3dg3rr` should become the Rust host/control plane for agent execution, policy, approvals, audit, notifications, and credential access.
- Agent orchestration may run in a sidecar runtime (for example LangChain/LangGraph or another agent SDK), but secrets and process supervision should remain owned by `l3dg3rr`.
- Xero is already part of the visible `ledgerr_*` capability direction; future work should treat it as an in-flight supervised capability, not a greenfield speculative add-on.
- Xero access should be mediated through supervised MCP worker processes/tools, not by giving raw credentials directly to the model or relying on `.env` as the long-term secret model.
- Windows 11 desktop support matters: toast/app notifications, tray/menubar control surface, and persistent settings are first-class operator features.
- Slint is the preferred UI shell for the desktop window, but tray/docked menubar icon integration may use a separate Rust crate such as `tray-icon` while Slint remains the main UI/event-loop host.

Desktop control-plane milestones currently in scope:
- persistent notification settings (`enabled/disabled`, backend health, last test result),
- tray icon with quick actions (`toast enabled`, `test toast`, `status`, `show window`, `quit`),
- notifier abstraction with a practical Windows fallback path first and native Windows notifications next,
- credential abstraction with Windows Credential Manager for long-lived secrets,
- audit-friendly event flow from agent/tool execution to UI and notifications.

### Purpose (non-duplicate)

`AGENTS.md` is intentionally operational. It should not restate the full product brief from the `## Project` section below.

### Capability / Usage Notes

- Treat `Justfile` as the canonical source of build, test, run, and host-launch commands.
- When a command changes, update `Justfile` first and then reference the recipe name here instead of copying the shell line.
- Prefer `just ...` recipes over ad hoc shell invocations for repeatable work, especially for Windows-host builds and tray validation.
- Keep capability and usage notes concise and action-oriented: say what exists, how to invoke it, and what not to assume.
- If a workflow depends on a repo capability, document the recipe name, not an inline transcript of the implementation.

### MCP Capability Training (Concrete)

Use `TurboLedgerService` in `crates/ledgerr-mcp/src/lib.rs` as the canonical contract.
Use `docs/mcp-capability-contract.md` as the canonical MCP surface map (tool names, arg contracts, service mapping, contrived usage flow).

Published MCP surface rule:
- Default `tools/list` should stay collapsed to the 8 top-level `ledgerr_*` capability families: `ledgerr_documents`, `ledgerr_review`, `ledgerr_reconciliation`, `ledgerr_workflow`, `ledgerr_audit`, `ledgerr_tax`, `ledgerr_ontology`, `ledgerr_xero`.
- Use required `action` parameters to expose sub-operations while keeping major capability families visible.
- Keep any legacy `l3dg3rr_*` or proxy-style names hidden compatibility aliases only; do not advertise them in the default tool catalog.

Core methods:
- `list_accounts` / `list_accounts_tool`: enumerate account ids from manifest.
- `validate_source_filename`: enforce `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE.ext`.
- `ingest_statement_rows`: idempotent journal/workbook ingest; returns deterministic `tx_ids`.
- `ingest_pdf`: preflight filename + writes raw context bytes when missing + ingests rows.
- `get_raw_context`: read bytes from `rkyv_ref`.
- `run_rhai_rule`, `classify_ingested`, `query_flags`, `classify_transaction`, `reconcile_excel_classification`, `query_audit_log`.
- `export_cpa_workbook`, `get_schedule_summary`.

Concrete example 1 (account discovery):
```rust
let service = TurboLedgerService::from_manifest_str(manifest)?;
let response = service.list_accounts_tool(ListAccountsRequest)?;
assert_eq!(response.accounts[0].account_id, "WF-BH-CHK");
```

Concrete example 2 (idempotent ingest):
```rust
let first = service.ingest_statement_rows(IngestStatementRowsRequest {
    journal_path,
    workbook_path,
    rows,
})?;
let second = service.ingest_statement_rows(IngestStatementRowsRequest {
    journal_path,
    workbook_path,
    rows,
})?;
assert_eq!(first.inserted_count, 1);
assert_eq!(second.inserted_count, 0);
```

Concrete example 3 (PDF ingest with raw context fallback write):
```rust
let response = service.ingest_pdf(IngestPdfRequest {
    pdf_path: "WF--BH-CHK--2023-01--statement.pdf".to_string(),
    journal_path,
    workbook_path,
    raw_context_bytes: Some(b"ctx".to_vec()),
    extracted_rows,
})?;
assert_eq!(response.inserted_count, 1);
```

Concrete example 4 (classification edit with invariants + audit):
```rust
let updated = service.classify_transaction(ClassifyTransactionRequest {
    tx_id,
    category: "OfficeSupplies".to_string(),
    confidence: "0.93".to_string(), // must be decimal in [0,1]
    note: Some("manual correction".to_string()),
    actor: "agent".to_string(),
})?;
assert_eq!(updated.category, "OfficeSupplies");
```

### Agent-Safe Usage Rules

- Prefer Postel-style input handling at boundaries: accept practical input variance, normalize early, emit strict deterministic outputs.
- For MCP row ingest arguments, accept both `account_id` and legacy `account` keys, then normalize to canonical `account_id` internally.
- Do not bypass invariant checks (`tx_id` hash match, decimal parse, date shape, confidence range).
- Keep status/state outputs concise and obvious for small models; favor explicit fields over implicit behavior.
- Before adding new custom infrastructure, confirm an existing crate/tool already solves it acceptably.
- Distill durable session lessons back into this file when they affect future agent quality.
- Keep concerns separated within every `AGENTS.md`: product direction, capability usage, and workflow rules should each live in their own short subsection.
- Avoid mixing build commands into policy sections; route those details to `Justfile` so one file remains the executable build contract.

### Execution Loop (Successive Generations)

Future agents should follow this working loop unless the user directs otherwise:
- branch first before substantial edits;
- break work into explicit tasks and keep them small enough to verify;
- use sub-agents for bounded parallel discovery or independent validation when that reduces context load;
- add or update tests with the change whenever behavior, contracts, or workflows move;
- independently validate tests rather than assuming correctness from inspection;
- loop on fixes until tests pass;
- check in with the user after each meaningful milestone or architecture decision;
- memoize stable next steps, constraints, and unresolved risks back into this file when they matter for later sessions;
- repeat until the user is satisfied.

Practical interpretation:
- prefer one agent/sub-agent for implementation and another for targeted verification when the user explicitly wants delegation or parallel work;
- do not treat green tests as the only completion signal if the UX, notification path, tray behavior, or host integration still lacks a real validation path;
- when desktop/host features are being designed, verify the smallest executable slice first (for example, a real toast test before larger UI work).

<!-- GSD:project-start source:PROJECT.md -->
## Project

**tax-ledger**

tax-ledger is a local-first personal financial document intelligence system focused on retroactive U.S. expat tax preparation from raw PDF statements. It ingests statement PDFs, classifies transactions with agent-editable rules, and produces a CPA-auditable Excel workbook with Schedule-oriented outputs and full mutation history. It is built for an operator/agent workflow where AI performs ingestion, classification, reconciliation, and flagging while a human accountant reviews and signs off in Excel.

**Core Value:** Convert raw historical financial PDFs into accountant-usable, auditable Excel tax records without sending private data to third-party SaaS.

### Constraints

- **Data Interface**: Excel workbook is the canonical human/audit layer — CPA workflow and signoff depend on it
- **Money Semantics**: `rust_decimal::Decimal` only for currency values — financial correctness and reproducibility
- **Identity Model**: Content-hash IDs only (Blake3 over account/date/amount/description) — idempotent ingest and dedup safety
- **Deployment Model**: Local-first single-user operation — no mandatory cloud services or ops-heavy infrastructure
- **Input Shape**: Source files must follow `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE` naming — deterministic ingest routing
- **Safety Bar**: No panic-prone pipeline code (`unwrap`, unchecked indexing) in financial paths — avoid silent data corruption and runtime faults
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Framework
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Rust | 1.88+ | Primary implementation language | Rust-first requirement, strong correctness/safety for financial data paths, excellent local deploy story |
| Tokio | 1.50.x | Async runtime | Standard 2025-2026 Rust async baseline for file/IO-heavy pipelines |
| Axum | 0.8.8 | Local API surface (optional UI backend) | Stable, ergonomic, integrates cleanly with `tower` middleware |
| RMCP (`modelcontextprotocol/rust-sdk`) | 0.8.x line | MCP server implementation for agent tool contract | Official Rust MCP SDK; avoids building protocol plumbing from scratch |
### Database
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Excel workbook (`.xlsx`) via `rust_xlsxwriter` + `calamine` | `rust_xlsxwriter` 0.94.0, `calamine` 0.34.0 | Canonical accountant/audit data interface | Matches CPA workflow constraint exactly; write/read in pure Rust without Excel COM dependency |
| `rkyv` sidecar archives (`.rkyv`) | 0.8.15 | Zero-copy raw extraction snapshots per source document | Fast local context recall for audit/classification without re-parsing PDFs |
| Graph projection (phase 2+) | HelixDB (current) OR `heed`+`petgraph` fallback | Relationship traversal over workbook facts | Keep Excel as truth, use graph only as query projection; keep fallback because HelixDB is newer/more volatile |
### Infrastructure
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Docker multi-stage + `cargo-chef` | Docker + `cargo-chef` 0.1.77 | Reproducible local deployment and fast rebuilds | Standard Rust container pattern in 2025-2026; dependency-layer caching reduces iteration time |
| Cocogitto | current (`cog`) | Conventional commits, changelog, version bump automation | Fits required release/versioning workflow with low process overhead |
| `tracing` + `tracing-subscriber` | 0.1.41 / 0.3.23 | Structured audit-grade operational logs | Better observability than string logs; fits async workflows |
### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `rust_decimal` | 1.40.0 | Money type | Always for monetary values; no `f64` in domain structs |
| `blake3` | 1.8.3 | Deterministic content-hash transaction IDs | Always for idempotent ingest identity |
| `rhai` | 1.24.0 | Runtime-editable classification/flag rules | Use for tax/category heuristics that need agent/human edits without recompile |
| `strum` (+ derive) | 0.27.x | Enum string roundtrip (`TaxCategory`, `Flag`) | Use for Excel validation value generation and strict parse/serialize symmetry |
| `notify` | 8.2.0 | Workbook/file change detection | Use debounce watcher (for human Excel edits + new PDFs) instead of polling-first |
| `thiserror` | 2.0.18 | Typed boundary/domain errors | Use in pipeline/services to keep failure causes explicit and auditable |
| Apache Arrow + DataFusion | DataFusion 52.3.0 | Analytics/export query path (not source of truth) | Use for year-end summaries and cross-account analytics over materialized datasets |
| Docling (Python sidecar/CLI) | 2.78.0 | Document parsing/OCR to structured markdown/json | Use as isolated local extraction service; keep Rust core clean and deterministic |
## Alternatives Considered
| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Canonical store | Excel workbook | SQLite/Postgres as system-of-record | Breaks accountant-first review/handoff requirement; adds translation friction |
| Excel integration | `rust_xlsxwriter` + `calamine` | COM automation / Office interop | Not cross-platform, brittle in containers/headless local deployments |
| Rule engine | `rhai` | Recompile-on-change Rust rules | Slows classification iteration and weakens agent-editable workflow |
| Transaction IDs | `blake3` content hash | Auto-increment IDs / random UUIDs | Breaks deterministic idempotent re-ingest behavior |
| Graph projection | HelixDB with fallback plan | Hard-coding graph traversal into relational tables only | Raises query complexity for money-flow tracing and relationship audits |
| Deployment | Docker + cargo-chef | Raw host-only toolchain installs | Harder reproducibility across machines; weaker onboarding and release confidence |
## Explicit "Do Not Use" List
## Installation
# Core runtime + API
# Ledger data model + Excel roundtrip
# File/system behavior
# Errors + observability
# Agent protocol + analytics
# Tooling
# Document extraction sidecar (local only)
# or
## Sources
- `rust_xlsxwriter` docs (0.94.0): https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/
- `rust_xlsxwriter` data validation examples: https://rustxlsxwriter.github.io/examples/data_validation.html
- `calamine` docs (0.34.0): https://docs.rs/calamine
- `rust_decimal` docs (1.40.0): https://docs.rs/rust_decimal/latest/rust_decimal/
- `rkyv` docs (0.8.15): https://docs.rs/rkyv/latest/rkyv/index.html
- `rhai` docs (1.24.0): https://docs.rs/rhai/latest/rhai/
- `blake3` docs (1.8.3): https://docs.rs/blake3/latest/blake3/
- `strum` docs (0.27): https://docs.rs/strum/latest/strum/
- `notify` docs (8.2.0): https://docs.rs/crate/notify/latest
- `axum` docs (0.8.8): https://docs.rs/axum/latest/axum/
- `tokio` docs (1.50.0): https://docs.rs/tokio/latest/tokio/
- `tracing-subscriber` docs (0.3.23): https://docs.rs/tracing-subscriber/
- DataFusion crate (52.3.0): https://docs.rs/crate/datafusion/latest
- Official MCP Rust SDK repo: https://github.com/modelcontextprotocol/rust-sdk
- HelixDB docs: https://docs.helix-db.com/
- `heed` docs (0.22.0): https://docs.rs/crate/heed/latest
- `petgraph` docs: https://docs.rs/petgraph/latest/petgraph/
- `cargo-chef` repo/docs: https://github.com/LukeMathWalker/cargo-chef
- Cocogitto docs: https://docs.cocogitto.io/
- Docling package/docs (2.78.0): https://pypi.org/project/docling/
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

## Session Learning Capture (Mandatory)

All future agents working in this repository must consider whether the session produced reusable guidance, tradeoff decisions, constraints, or lessons learned.

When meaningful new guidance appears, agents must distill it into concise, durable entries in `AGENTS.md` so it persists across sessions.

Capture should focus on:
- User-stated preferences that affect implementation or process
- Architectural or workflow decisions with lasting impact
- Pitfalls discovered and the preferred resolution pattern

Avoid noisy transcript-style notes. Record only stable guidance that improves future execution quality.

## Standing Task Hook (Post-Commit)

After every commit, validate that Claude plugin/skill usage documentation is current and aligned with recommended patterns from:
- https://code.claude.com/docs/en/plugins

Minimum requirement:
- Confirm the repository's Claude-facing docs still reflect the currently exposed MCP tools, expected arguments, and practical usage flow.
- If code changed MCP behavior, update docs in the same branch before opening or updating a PR.

Preferred implementation ("extra points"):
- Keep runnable documentation flows in `Justfile` target `test` that executes an MCP CLI path against sample data.
- Maintain two documented modes:
  - simple/basic happy-path usage
  - "spinning wheels" troubleshooting/diagnostic usage (intentional blocked or recovery-oriented flow)

Treat this as a standing operational gate, not a one-time migration task.

### Validation Memo

- 2026-04-02: executed post-commit plugin-doc validation against `https://code.claude.com/docs/en/plugins`.
  - Updated stale tool examples from `l3dg3rr_context_summary` to then-live MCP tools (`l3dg3rr_get_pipeline_status`, `l3dg3rr_list_accounts`, `l3dg3rr_get_raw_context`).
  - Added plugin skill frontmatter `name` for plugin-doc compatibility.
  - Added runnable `just test` outcome flow (Rust executable) with both simple and blocked-diagnostics scenarios.
- 2026-04-17: reduced the default MCP catalog to 7 top-level `ledgerr_*` tools and relocated plugin info under `ledgerr_workflow`.
  - Keep docs/examples aligned to the reduced surface; `tools/list` is now intended to be a trustworthy small catalog for agents.
  - Legacy `l3dg3rr_*` and proxy tool names remain compatibility aliases only and should not be reintroduced into the advertised catalog.
- 2026-04-21: Xero is now part of the advertised MCP catalog as `ledgerr_xero`, making the default published surface 8 top-level `ledgerr_*` tools.
  - Keep generated docs and AGENTS guidance aligned to `crates/ledgerr-mcp/src/contract.rs`; older references to a 7-tool surface are stale.
  - Documentation hierarchy should lead with operator capabilities first, then application structure, then visualization internals.
  - Use Z3 for hard satisfiability/proof obligations and Kasuari for soft plausibility/layout constraints.
  - `ledger-core` keeps native Z3 behind the `legal-z3` feature because default local builds may not have `libz3` installed.
- 2026-04-21: Rig integration boundary is host-owned.
  - `crates/ledgerr-host/src/agent_runtime.rs` is the current Rig-backed text/structured-output adapter and implements `ledger_core::verify::ModelClient` for validation/review flows.
  - Keep `ledger-core` deterministic and provider-agnostic; do not add direct Rig dependencies there.
  - Model-call audit hooks should record metadata and outcomes, not raw prompt or response content.
- 2026-04-21: Use `uv` for Python package and tool workflows.
  - Do not document `pipx` or direct `pip install` as the preferred path; use `uv tool install ...` for Python CLIs and `uv pip ...` for environment-scoped installs.
- 2026-04-17: issue `#22` established a code-first MCP contract path.
  - The published MCP surface now lives in `crates/ledgerr-mcp/src/contract.rs`; treat it as the only source of truth for parser shapes, generated JSON Schema, and checked-in operator docs/examples.
  - Regenerate `docs/mcp-capability-contract.md`, `docs/agent-mcp-runbook.md`, and `scripts/mcp_cli_demo.sh` via `cargo run -p xtask-mcpb -- generate-mcp-artifacts` after changing the published MCP surface.
  - Drift between `contract.rs` and those generated artifacts is a test failure, not a documentation chore.
- 2026-04-17: CPA workbook export is now explicitly projection-only.
  - Treat `ledger_core::workbook::REQUIRED_SHEETS` as the canonical base workbook contract for export paths.
  - `export_cpa_workbook` should rebuild the full workbook from canonical service state on each export, including `META.config`, `ACCT.registry`, schedule sheets, flag sheets, transaction sheets, and `AUDIT.log`.
  - Tests should assert representative workbook contents, not just that a file was written.
- 2026-04-17: restart-visible MCP operational state now persists as a deterministic sidecar next to the manifest workbook path.
  - Persist ingest idempotency state, transaction row cache, audit log, lifecycle event history, and HSM checkpoint together as one snapshot.
  - Keep the workbook as the human/accountant artifact; do not overload it as the only machine recovery mechanism for agent queues and replay state.
  - If the sidecar exists but cannot be parsed or its version is unsupported, fail closed instead of silently resetting state.
- 2026-04-19: P1 validation framework adds verb-centric pipeline with carry-forward confidence model.
  - Disposition (Unrecoverable/Recoverable/Advisory) on every Issue for clear signal on what action to take.
  - MetaCtx.accumulated_confidence compounds multiplicatively across stages.
  - LegalSolver verifies transactions against tax rules (AU GST Act s38-190, US Schedule C).
  - VendorConstraintSet evaluates data plausibility using Kasuari strengths.
  - WorkflowToml is single source of truth — compiles to Mermaid diagram for operator and Rhai FSM for execution.
  - VerbDef captures reversibility and access criteria (Commit/Reverse require Tray approval).
  - Multi-model verification loop: LLM proposes, second model reviews, operator approves.
  - Multi-jurisdiction: US/AU/UK with rules keyed by Jurisdiction.
- 2026-04-20: mdbook documentation with executable code examples.
  - Live at https://promptexecution.github.io/l3dg3rr/
  - Every chapter includes executable Rust code examples that can run as integration tests.
  - Include rhai code blocks that parse to Mermaid diagrams: ` ```rhai ` code fences.
  - Keep auto-diagram Rhai blocks to the supported mini-DSL only: `fn source() -> target`, `if expr -> target`, and `match expr => Arm -> target`. Do not drop general imperative Rhai examples into diagram sections unless they are fenced with another language or explicitly meant to render no diagram.
  - Cross-reference chapters using relative links (e.g., `[Graph](./graph.md)`).
  - Include "Related Chapters" section in each chapter for navigation.
  - Use Option/Result/Either monadic patterns in code examples to reflect real API style.
  - Theory of Operation chapter documents Novel Theory of Tool (NTTP) pattern.
  - CI runs `just docgen-check` as part of docs job; validates generated Mermaid blocks, cross-references, live-editor JS syntax, and live-editor unit tests. Static mdBook output contains `<pre class="mermaid">`; SVG rendering happens in the browser/live editor.
  - `just docgen` - build local docs, `just docgen-check` - validate diagrams and links, `just docserve` - serve the built book locally with the live Rhai diagram editor.
  - Windows/WSL startup for live docs is memoized in `scripts/docserve-live.pwsh` and invoked via `just wsl2-pwsh-docserve`.
  - The live docs editor now has two synchronized render modes for supported Rhai diagram blocks: `isometric-3d` and `mermaid-2d`.
  - Treat the shared browser core in `book/theme/rhai-live-core.js` as the contract for parser output, layout metadata, icon inference, and render-failure messaging; add tests there when the live docs behavior changes.
  - The isometric docs renderer currently uses deterministic layered placement with animated SVG reflow and autogenerated glTF data URIs per node as a fallback model path; Mermaid remains the canonical 2D reference view and failure fallback.
  - Prefer worked sample blocks in visualization-heavy chapters over abstract prose. At minimum, include one happy-path sample and one branch-heavy sample that can be pasted into the live editor.
  - The future `match` operator contract is documented in `book/src/match-visualization-plan.md`; keep Mermaid and isometric semantics aligned to that plan instead of inventing per-view behavior.
  - When adding new modules, add corresponding chapter in `book/src/` and update `book/src/SUMMARY.md`.
- 2026-04-27: PRD-4 Phase 1 established `ledger-core::ontology` as the canonical ontology primitive layer while preserving MCP legacy storage shape.
  - Keep `ledgerr-mcp` ontology files backward-compatible with `{ entities, edges }` and legacy `entity|...` / `edge|...` Blake3 ID prefixes unless a deliberate migration is planned.
  - PRD-4 Phase 2 ingest ontology emission is opt-in through `ontology_path` on ingest requests; do not write ontology sidecars implicitly from workbook paths unless product policy changes.
  - PRD-4 Phase 4 typed Phi-4 jobs start in `crates/ledgerr-host/src/agent_runtime.rs`; the local fallback in `internal_openai.rs` must satisfy the same JSON-only schema contract as a real model.
  - PRD-4 Phase 5 proposal lifecycle lives in `ledger-core::proposal`; model proposals must be validated before commit and low-confidence or mutating relations require explicit operator approval.
  - PRD-4 Phase 6 semantic retrieval starts as a deterministic local lexical index in `RuleRegistry`; future embedding backends must preserve stable candidate IDs and keep `classify_waterfall` authoritative.
  - PRD-4 Phase 7 audit playbook must keep workbook rows, ontology facts, lifecycle events, and visual graph examples tied to the same deterministic transaction IDs.
  - `book/src/SUMMARY.md` must not list the same chapter file twice; mdBook fails closed on duplicate paths before diagram checks run.
  - Current validated docs toolchain is `mdbook 0.4.52`, `mdbook-mermaid 0.16.0`, and `mdbook-admonish 1.20.0` with admonish assets version `3.1.0`.
- 2026-04-22: docs Rhai mutation playground is model-prompt-first.
  - The browser-side mdBook playground prepares constrained prompts and deterministic example drafts; it does not call an LLM directly from the browser.
  - Keep the prompt contract limited to supported Rhai diagram DSL lines (`fn`, `if`, `match`) plus concise explanation text.
  - Default examples should use the tool-tray local provider label (`phi-4-mini-reasoning`) unless a specific external model is configured in the host settings.
- 2026-04-22: Slint host UI needs a tested Rust state seam.
  - Keep chat transcript rendering, Rhai prompt seeding, and review diffset logging in `crates/ledgerr-host/src/chat.rs` so Linux tests exercise the UX behavior without launching Slint.
  - `ledgerr-host` uses `unsafe_code = "deny"` instead of inheriting workspace `forbid` because Slint macro-generated code emits `allow(unsafe_code)` attributes; keep direct unsafe out of host code and keep `ledger-core` under the stricter workspace policy.
- 2026-04-22: tool-tray internal webserver owns local chat and docs playbook routes.
  - Use `crates/ledgerr-host/src/internal_openai.rs` for the localhost OpenAI-compatible contract: `/v1/models`, `/v1/chat/completions`, and `/docs/`.
  - The Slint window should switch providers by setting tested `ChatSettings`: `phi-4-mini-reasoning` on `http://127.0.0.1:15115/v1/chat/completions` for local mode, or the cloud OpenAI-compatible URL for remote mode.
  - Build mdBook assets before expecting `/docs/` to serve useful content; use `just host-playbook-window` for the packaged playbook launch path.
- 2026-04-24: README/product framing is bookkeeping-first with visual workflow graph as the organizing model.
  - Describe `l3dg3rr` as a strongly typed, ontologically linked graph of scriptable visual-first workflows for supervised AI/LLM ETL into CPA-auditable bookkeeping artifacts.
  - Keep README structure MECE: bookkeeping truth, typed domain model, ontology graph, scriptable policy, workflow control, visualization, MCP/agent boundary, and operator host.
  - Clarify Rhai surfaces separately: transaction rules use `fn classify(tx)`, workflow compiler output may emit Rhai `switch`, and docs visualization uses the narrow `match expr => Arm -> target` DSL.


<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
