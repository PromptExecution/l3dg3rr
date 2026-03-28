# Architecture Patterns

**Domain:** Excel-as-truth tax ledger service with MCP interface  
**Researched:** 2026-03-28

## Recommended Architecture

Use a **layered, local-first core** with strict boundary ownership:

1. `ledger-core` (domain and invariants) is the center.
2. `ledger-io` (Excel + PDF snapshot ingest) is the only path that mutates persisted truth.
3. `ledger-rules` (Rhai classification/flag rules) is deterministic, auditable compute over domain records.
4. `ledger-audit` (append-only mutation history) records all changes in both workbook and machine log.
5. `ledger-projection` (HelixDB graph projection) is a rebuildable read-model, never source of truth.
6. `ledger-mcp` (tool interface) is an adapter over `LedgerService`, not business logic.
7. `ledger-api-ui` (Axum/Leptos) is optional convenience, behind stable service interfaces.

The source-of-truth contract is:
- Human/audit truth: `tax-ledger.xlsx` (`TX.*`, `AUDIT.log`, schedules, flags)
- Machine truth for retrieval: `.rkyv` sidecars for parsed document context
- Query acceleration: HelixDB projection derived from workbook rows

## Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `ledger-core` | Domain types (`Amount`, `TxId`, `TaxCategory`, `Flag`), traits (`ExcelRecord`, `Auditable`, `GraphNode`), validation rules | All layers depend on this only |
| `ledger-io` | Workbook init, sheet/table schema, calamine read, rust_xlsxwriter write, PDF naming parse, rkyv sidecar persistence | `ledger-core`, filesystem |
| `ledger-rules` | Load/execute `rules/*.rhai`, produce category/confidence/flags from transactions | `ledger-core`, `ledger-io` |
| `ledger-audit` | Append-only mutation journal, field-level diffs, actor/session metadata | `ledger-core`, `ledger-io` |
| `ledger-service` | Orchestration use-cases: ingest, classify, resolve flags, summaries, rebuild projections | `ledger-core`, `ledger-io`, `ledger-rules`, `ledger-audit`, `ledger-projection` |
| `ledger-projection` | Graph materialization from workbook rows, traversal/query APIs, full rebuild on startup or drift | `ledger-core`, HelixDB backend |
| `ledger-mcp` | Stable MCP tool contract and argument/response schemas; no direct storage logic | `ledger-service` |
| `ledger-api-ui` | HTTP endpoints + dashboard for operator convenience; never bypasses service | `ledger-service` |
| `manifest` module | Lightweight session/account/progress index for bounded startup context | `ledger-io`, `ledger-service`, `ledger-mcp` |

## Data Flow

1. **Discovery:** Agent reads `manifest.toml` to identify workbook path, accounts, years, ingest/classification progress.
2. **Ingest:** MCP `ingest_pdf(path)` -> parse filename -> doc extraction -> persist `.rkyv` snapshot -> normalize tx rows -> deterministic `TxId` hash.
3. **Classification:** `ledger-rules` evaluates Rhai scripts and emits `category`, `confidence`, `flags`, `rule_version`.
4. **Validation:** Domain constructors enforce money scale, enum validity, required refs.
5. **Commit to truth:** `ledger-io` writes transaction row updates to correct `TX.<account-id>` table and appends `AUDIT.log` row.
6. **Projection sync:** `ledger-projection` applies incremental sync or rebuild from workbook.
7. **Query:** MCP/API calls read from workbook-backed service state; graph-heavy flows route to projection.
8. **Human edits:** Excel save event (file watcher + debounce) triggers diff reload, validation, audit append, projection re-sync.

## Patterns to Follow

### Pattern 1: Ports and Adapters around `LedgerService`
**What:** Keep domain and use-cases independent from MCP, HTTP, or DB adapters.  
**When:** Always; every external system is an adapter.

### Pattern 2: Deterministic Identity + Idempotent Ingest
**What:** `TxId = blake3(account_id || date || amount || description)` and upsert-by-id.  
**When:** Every ingest/re-ingest path.

### Pattern 3: Workbook-First Write, Projection-Second
**What:** Write canonical row + audit first, then update Helix projection.  
**When:** Any mutation or classification action.

### Pattern 4: Explicit MCP Contract Versioning
**What:** MCP tools expose `contract_version`, strict request schemas, typed error codes.  
**When:** From first tool release; avoid silent interface drift.

### Pattern 5: Evented Excel Reconciliation
**What:** Watch workbook file changes with debounce, re-read changed sheets, diff against prior snapshot, append auditable change set.  
**When:** Always if human accountant edits workbook directly.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Business Logic in MCP Handlers
**Why bad:** Duplicated rules and divergent behavior between MCP/UI/CLI paths.  
**Instead:** MCP handler only validates transport schema and calls `LedgerService`.

### Anti-Pattern 2: Projection as Truth
**Why bad:** Drift, non-auditable states, accountant mismatch.  
**Instead:** Rebuildable projection from workbook, with checksum/drift checks.

### Anti-Pattern 3: Freeform Cell Parsing in Domain Layer
**Why bad:** Runtime panics and silent coercion from Excel values.  
**Instead:** Centralized boundary mappers `Cell -> Validated Domain Type`.

### Anti-Pattern 4: Combining HelixDB and UI in Early MVP
**Why bad:** Two independent risk fronts delay usable tooling.  
**Instead:** Phase-separate Helix projection from web dashboard delivery.

## Dependency-Driven Build Sequence

Build order should minimize unknowns and lock contracts early:

1. **Phase 0: Contract + Skeleton (must happen first)**
   - Finalize workbook sheet/table schema and column contracts.
   - Define MCP tool signatures and response/error envelopes.
   - Define `manifest.toml` schema and lifecycle.
   - Create crate boundaries: `ledger-core`, `ledger-service`, `ledger-io`, adapters.

2. **Phase 1: Core Correctness + Excel Roundtrip (MVP usable)**
   - Implement domain types/invariants (`Amount`, `TxId`, enums, validation).
   - Workbook initializer and read/write roundtrip.
   - Ingest pipeline with deterministic IDs + `.rkyv` sidecars.
   - Rhai classification execution.
   - Append-only audit writes.
   - MVP MCP tools: ingest, classify, flags, summaries, raw context.

3. **Phase 2: Human-in-the-Loop Reconciliation + Stability**
   - File watcher debounce loop for workbook edits.
   - Diff/audit reconciliation on human edits.
   - Idempotent re-ingest and conflict policy tests.
   - Containerization (Docker) + release/versioning flow (cocogitto).

4. **Phase 3: Graph Projection**
   - HelixDB adapter + `GraphStore` trait.
   - Startup rebuild + incremental sync pipeline.
   - MCP graph query tools.

5. **Phase 4: Convenience Interfaces + Analytics**
   - Axum HTTP API over existing service contracts.
   - Leptos dashboard (flags queue, rule testing UI, schedule summaries).
   - Arrow/DataFusion export path for analytics.

This sequencing makes MVP available at Phase 1 and avoids coupling graph/UI complexity to correctness-critical ingest and audit paths.

## Scalability Considerations

| Concern | At 100 users (single operator reality) | At 10K workbooks | At 1M transactions/workbook |
|---------|-----------------------------------------|------------------|-----------------------------|
| Workbook I/O | Single-file reads/writes are acceptable | Partition by workbook per client and batch writes | Keep workbook as audit/handoff; move heavy analytics to Arrow/DataFusion snapshots |
| Reconciliation | Full-sheet diff acceptable | Incremental changed-row tracking required | Hash-index rows and sheet-level checkpoints |
| Projection | Full rebuild on startup acceptable | Incremental projection preferred | Async projection worker + resumable checkpoints |
| MCP latency | Sub-second expected | Per-workbook process isolation | Cache manifest + selective sheet loading + mmap rkyv lookups |

## Sources

- PRD architecture brief and constraints (local file): `/home/brianh/promptexecution/mbse/l3dg3rr/prd.md` (HIGH)
- Project requirements and decisions (local file): `/home/brianh/promptexecution/mbse/l3dg3rr/.planning/PROJECT.md` (HIGH)
- Model Context Protocol overview/spec references: https://modelcontextprotocol.io (MEDIUM)
- HelixDB MCP guide (stateful MCP traversal): https://docs.helix-db.com/guides/mcp-guide (MEDIUM)
- `calamine` crate documentation: https://docs.rs/calamine (MEDIUM)
- `rust_xlsxwriter` crate documentation: https://docs.rs/rust_xlsxwriter (MEDIUM)
- Cocogitto documentation (release/versioning workflow): https://docs.cocogitto.io (MEDIUM)
