# Phase 2: Deterministic Ingestion Pipeline - Research

**Researched:** 2026-03-28
**Domain:** Deterministic PDF ingest, replay-safe dedupe, Beancount journal persistence, MCP evidence lookup
**Confidence:** MEDIUM-HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
### Ingestion Contract
- Enforce preflight filename contract before any mutation (`VENDOR--ACCOUNT--YYYY-MM--DOCTYPE`).
- Keep transaction identity deterministic via stable canonical hashing inputs.
- Treat re-ingest as idempotent: same inputs must produce same IDs and no duplicate inserts.

### Source Evidence
- Persist and propagate source references (`.rkyv` sidecar path) with ingested transaction data.
- Keep retrieval path explicit so MCP `get_raw_context` can resolve evidence deterministically.

### the agent's Discretion
- Use `accounting-core` where it reduces custom ledger logic and preserves auditability.
- Keep adapters thin and explicit; avoid hidden mutation side effects.

### Deferred Ideas (OUT OF SCOPE)
- Full workbook write-path integration is deferred to later in Phase 2 after service-level ingest contracts are finalized.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ING-01 | User can ingest a renamed statement PDF from disk and materialize transaction rows into the corresponding `TX.<account-id>` sheet | Enforce filename preflight first; keep deterministic ID canonicalization stable; add explicit ingest orchestrator contract that emits journal + workbook projection artifacts deterministically |
| ING-02 | User can re-ingest the same statement without duplicate transactions (idempotent behavior) | Maintain deterministic hash identity + dedupe set/index as sole idempotency key; add replay tests over repeated ingest calls |
| ING-03 | User can persist parsed document context as a `.rkyv` snapshot alongside the source PDF | Add sidecar serializer/writer with explicit path derivation and non-panicking IO errors; persist emitted path in ingest output |
| ING-04 | User can trace each ingested transaction back to its source document reference | Preserve `source_ref` in transaction model and Beancount metadata (`source_ref`) and workbook projection columns |
| MCP-01 | User can call `ingest_pdf(path)` through MCP and receive deterministic transaction IDs | Keep MCP adapter thin: validate filename -> call core ingest -> return tx IDs from core output without mutation in adapter |
| MCP-05 | User can call `get_raw_context(rkyv_ref)` through MCP for source evidence lookup | Keep direct path-based evidence retrieval via `std::fs::read` with explicit error mapping and contract tests |
</phase_requirements>

## Summary

Phase 2 is already partially implemented with deterministic hashing, replay-safe in-memory dedupe, Beancount append output, and MCP stubs (`ingest_pdf`, `get_raw_context`). The remaining implementation risk is not algorithmic complexity; it is contract completeness and persistence consistency across three surfaces: journal text, workbook projection, and `.rkyv` evidence sidecars.

The safest completion strategy is to keep one canonical ingest pipeline in `ledger-core` and let MCP stay a thin adapter. Preflight filename validation must remain before all writes. Transaction IDs must continue to derive from canonical `(account_id, date, amount, description)` hashing. Evidence reference must be emitted once and reused everywhere (`source_ref` in transaction output, journal metadata, MCP evidence read path).

**Primary recommendation:** Implement a single deterministic ingest orchestrator in `ledger-core` that atomically drives dedupe decision, journal append, workbook projection, and `.rkyv` sidecar reference emission, then map MCP tools directly to that orchestrator.

## Project Constraints (from CLAUDE.md)

- Excel workbook remains the canonical human/audit interface.
- Money values must use `rust_decimal::Decimal` (no float-backed money).
- Transaction identity must be content-hash based (Blake3 over account/date/amount/description).
- Local-first single-user operation; avoid cloud/ops-heavy dependencies.
- Source filenames must follow `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE`.
- No panic-prone pipeline paths (`unwrap`, unchecked indexing) in financial flows.
- Use existing GSD workflow conventions for edits/planning context.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `blake3` | 1.8.3 | Deterministic transaction identity | Fast cryptographic hash; currently used in `deterministic_tx_id`; aligns with locked idempotency decision |
| `rust_xlsxwriter` | 0.94.0 (project currently 0.79.4) | Workbook write path (`TX.<account-id>` and required sheets) | Canonical Rust Excel writer in project architecture; supports accountant-first workflow |
| `calamine` | 0.34.0 (project currently 0.26.1) | Workbook read path/roundtrip | Canonical pure-Rust spreadsheet reader paired with `rust_xlsxwriter` |
| `rkyv` | 0.8.15 (not yet wired in current code) | `.rkyv` sidecar context snapshots | Zero-copy archive format for local evidence snapshots |
| `thiserror` | 2.0.18 (project currently 1.0.69 + 2.0.18 in lockfile) | Typed boundary errors | Keeps ingest/MCP failures explicit and non-panicking |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `rust_decimal` | 1.41.0 (constraint baseline 1.40.x) | Money-safe parsing/normalization | For all amount parsing and canonical formatting in ingest, never `f64` |
| `tempfile` | 3.x | deterministic integration tests | For replay/idempotency and file side-effect tests |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Beancount journal append | DB-first ingest ledger | Violates current locked Git-friendly plain-text direction |
| `blake3` deterministic IDs | UUID/random IDs | Breaks replay determinism and dedupe guarantees |
| `.rkyv` sidecar evidence | ad-hoc JSON blobs | More fragile schema/perf story for binary context snapshots |

**Installation:**
```bash
cargo add blake3 thiserror rkyv rust_decimal rust_xlsxwriter calamine
```

**Version verification (crates.io API, 2026-03-28):**
- `blake3`: 1.8.3 (2026-01-08)
- `rust_xlsxwriter`: 0.94.0 (2026-02-28)
- `calamine`: 0.34.0 (2026-03-07)
- `rkyv`: 0.8.15 (2026-02-10)
- `thiserror`: 2.0.18 (2026-01-18)
- `rust_decimal`: 1.41.0 (2026-03-27)

## Architecture Patterns

### Recommended Project Structure
```
crates/ledger-core/src/
├── filename.rs      # preflight contract parser
├── ingest.rs        # deterministic ingest orchestration + dedupe
├── journal.rs       # Beancount append + metadata
├── workbook.rs      # TX sheet materialization path
└── evidence.rs      # .rkyv sidecar encode/decode + path derivation (add)
```

### Pattern 1: Boundary-First Preflight
**What:** Parse/validate source filename before any state mutation.
**When to use:** At entry to `ingest_pdf` and any path-based ingest API.
**Example:**
```rust
let file_name = std::path::Path::new(&request.pdf_path)
    .file_name()
    .and_then(|name| name.to_str())
    .ok_or_else(|| ToolError::InvalidInput("pdf_path must have a valid filename".to_string()))?;
let _parsed = self.validate_source_filename(file_name)?;
```

### Pattern 2: Deterministic ID as Single Idempotency Key
**What:** Canonicalize fields and hash exactly once with Blake3.
**When to use:** Before dedupe insert and before persistence fan-out.
**Example:**
```rust
let canonical = format!(
    "{}|{}|{}|{}",
    row.account_id.trim().to_ascii_uppercase(),
    row.date.trim(),
    row.amount.trim(),
    row.description.trim().to_ascii_lowercase(),
);
let tx_id = blake3::hash(canonical.as_bytes()).to_hex().to_string();
```

### Pattern 3: Thin MCP Adapter, Fat Core
**What:** Keep `turbo-mcp` as request/response translation only.
**When to use:** All tool methods (`ingest_pdf`, `get_raw_context`).
**Example:** `ingest_pdf` in `turbo-mcp` should call one core ingest function and return its outputs without extra business logic branches.

### Anti-Patterns to Avoid
- **Duplicate ingest logic in MCP and core:** causes drift and nondeterministic behavior.
- **Per-surface dedupe decisions:** dedupe must happen once before journal/workbook writes.
- **Implicit `source_ref` derivation in multiple places:** compute once, propagate explicitly.
- **`unwrap` in test-promoted code paths:** violates safety constraint and can hide data corruption faults.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Transaction identity | custom ad-hoc string IDs | `blake3` canonical hash | Proven deterministic identity and stable idempotency behavior |
| Money parsing/normalization | float parsing/formatting | `rust_decimal` | Prevents rounding drift and reproducibility errors |
| Excel OOXML writing | manual zip/xml generation | `rust_xlsxwriter` | Avoids high-complexity format bugs |
| Evidence serialization | bespoke binary format | `rkyv` | Better performance and lower maintenance risk for snapshot format |
| Error taxonomy | stringly-typed errors only | `thiserror` enums | Preserves auditable causal error boundaries |

**Key insight:** Deterministic ingest fails most often at boundary semantics and fan-out consistency, not parsing complexity. Reuse proven libraries and keep one canonical pipeline.

## Common Pitfalls

### Pitfall 1: Hash/Input Canonicalization Drift
**What goes wrong:** Same logical transaction hashes differently across code paths.
**Why it happens:** Inconsistent trim/case/amount normalization before hashing.
**How to avoid:** Centralize canonicalization in one function (`deterministic_tx_id`) and forbid alternate implementations.
**Warning signs:** Re-ingest returns new tx IDs for same source rows.

### Pitfall 2: Journal Append Interleaving/Partial Write Assumptions
**What goes wrong:** Corrupted or interleaved journal entries under repeated writes.
**Why it happens:** Assuming append means whole-entry atomicity.
**How to avoid:** Build full entry strings and write each logical entry in a tightly controlled sequence; keep single-writer pattern in service.
**Warning signs:** Broken Beancount stanza boundaries in generated journal.

### Pitfall 3: Metadata Key Incompatibility in Beancount
**What goes wrong:** Beancount parser/tooling rejects metadata.
**Why it happens:** Invalid metadata key naming or malformed quoting.
**How to avoid:** Use lower-case metadata keys (`txid`, `source_ref`) and sanitize quotes in narration/source values.
**Warning signs:** Downstream Beancount parse failures on generated entries.

### Pitfall 4: MCP Adapter Side Effects
**What goes wrong:** MCP behavior differs from direct core ingest behavior.
**Why it happens:** Business logic duplicated in adapter methods.
**How to avoid:** Adapter only validates request shape + delegates to core pipeline.
**Warning signs:** Core tests pass but MCP contract tests fail on replay determinism.

## Code Examples

Verified patterns from current code and official docs:

### Deterministic ID Generation
```rust
pub fn deterministic_tx_id(row: &TransactionInput) -> String {
    let canonical = format!(
        "{}|{}|{}|{}",
        row.account_id.trim().to_ascii_uppercase(),
        row.date.trim(),
        row.amount.trim(),
        row.description.trim().to_ascii_lowercase(),
    );
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}
```

### Beancount Metadata for Evidence Traceability
```beancount
2023-01-15 * "Imported" "Coffee Shop"
  txid: "..."
  source_ref: "2023-taxes/WF--BH-CHK--2023-01--statement.rkyv"
```

### Replay-Safe Ingest Test Pattern
```rust
let first = ledger.ingest(&[tx.clone()]);
let second = ledger.ingest(&[tx]);
assert_eq!(first.len(), 1);
assert_eq!(second.len(), 0);
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Workbook-first Phase 2 ingest direction | Rustledger-compatible plain-text Beancount + deterministic tx metadata | 2026-03-28 (`STATE.md`) | Improves Git diffability and audit history readability while keeping deterministic semantics |
| Loose adapter behavior | Explicit MCP tool contracts (`ingest_pdf`, `get_raw_context`) | Phase 1-2 | Enables stable agent orchestration with testable request/response boundaries |

**Deprecated/outdated for this phase:**
- DB-first ingest state as source-of-truth: conflicts with locked Excel/journal local-first workflow.

## Open Questions

1. **Canonical amount formatting for hash input**
   - What we know: Hash currently uses `amount` string verbatim after trim.
   - What's unclear: Whether equivalent textual forms (e.g., `-42.1` vs `-42.10`) should hash identically.
   - Recommendation: Decide one canonical decimal formatting policy before finalizing ING-02 regression tests.

2. **`.rkyv` payload schema contract**
   - What we know: Requirement asks snapshot persistence + MCP retrieval by ref.
   - What's unclear: Exact archive schema/versioning for future compatibility.
   - Recommendation: Add versioned archive envelope now (e.g., `schema_version`, `source_pdf`, payload bytes).

3. **Workbook materialization scope in this phase**
   - What we know: `02-CONTEXT.md` defers full workbook integration detail.
   - What's unclear: Minimum viable `TX.<account-id>` columns required for phase-complete acceptance.
   - Recommendation: Lock minimal Phase 2 TX schema in tests before implementation.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `rustc` | core implementation/tests | ✓ | 1.88.0 | — |
| `cargo` | build/test | ✓ | 1.88.0 | — |
| `python3` | optional sidecar tooling/scripts | ✓ | 3.10.12 | not required for core ingest |
| `node` | GSD orchestration scripts | ✓ | 22.15.0 | — |
| `docker` | containerized run path (not required for phase coding) | ⚠ partial/error in this environment | error from runtime-dir permissions | run tests natively with cargo |

**Missing dependencies with no fallback:**
- None for Phase 2 core implementation and tests.

**Missing dependencies with fallback:**
- Docker unavailable in this shell context; fallback is native cargo workflow.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness via `cargo test` |
| Config file | none (workspace Cargo test defaults) |
| Quick run command | `cargo test -p ledger-core phase2_ingest -- --nocapture` |
| Full suite command | `cargo test --workspace -- --nocapture` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ING-01 | ingest contract-valid PDF into deterministic artifacts and TX projection | integration | `cargo test -p ledger-core phase2_ingest_pipeline_remaining -- --nocapture` | ❌ Wave 0 |
| ING-02 | replay-safe re-ingest with no duplicates | integration | `cargo test -p ledger-core phase2_ingest -- --nocapture` | ✅ |
| ING-03 | persist `.rkyv` context sidecar | integration | `cargo test -p ledger-core phase2_ingest_pipeline_remaining -- --nocapture` | ❌ Wave 0 |
| ING-04 | trace tx to source evidence reference | integration | `cargo test -p ledger-core phase2_rustledger_journal -- --nocapture` | ✅ |
| MCP-01 | `ingest_pdf(path)` returns deterministic tx IDs | integration | `cargo test -p turbo-mcp ingest_pdf_validates_filename_and_ingests_rows -- --nocapture` | ✅ |
| MCP-05 | `get_raw_context(rkyv_ref)` reads source bytes | integration | `cargo test -p turbo-mcp get_raw_context_reads_rkyv_reference_bytes -- --nocapture` | ✅ |

### Sampling Rate
- **Per task commit:** `cargo test -p ledger-core phase2_ingest -- --nocapture && cargo test -p turbo-mcp -- --nocapture`
- **Per wave merge:** `cargo test --workspace -- --nocapture`
- **Phase gate:** Full workspace green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `crates/ledger-core/tests/phase2_ingest_pipeline_remaining.rs` — explicit ING-01/03 coverage
- [ ] `crates/turbo-mcp/tests/phase2_mcp_contract_remaining.rs` — end-to-end MCP-01/05 replay and evidence contract

## Sources

### Primary (HIGH confidence)
- `02-CONTEXT.md`: locked decisions and scope boundaries  
  https://github.com (local file source: `.planning/phases/02-deterministic-ingestion-pipeline/02-CONTEXT.md`)
- `REQUIREMENTS.md`: requirement definitions and traceability  
  https://github.com (local file source: `.planning/REQUIREMENTS.md`)
- Existing implementation:
  - `crates/ledger-core/src/ingest.rs`
  - `crates/ledger-core/src/journal.rs`
  - `crates/turbo-mcp/src/lib.rs`
- Beancount language syntax (metadata rules and examples): https://beancount.github.io/docs/beancount_language_syntax.html
- Rust `OpenOptions` append semantics: https://doc.rust-lang.org/nightly/std/fs/struct.OpenOptions.html
- crates.io metadata (versions + publish dates):
  - https://crates.io/crates/blake3
  - https://crates.io/crates/rust_xlsxwriter
  - https://crates.io/crates/calamine
  - https://crates.io/crates/rkyv
  - https://crates.io/crates/thiserror
  - https://crates.io/crates/rust_decimal

### Secondary (MEDIUM confidence)
- Project stack baseline from `CLAUDE.md` (version intent, architecture direction)

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM-HIGH - versions verified from crates.io; some are project-intent vs currently pinned dependencies.
- Architecture: HIGH - based on locked decisions plus existing code/tests.
- Pitfalls: MEDIUM - supported by docs and current implementation patterns; some failure modes inferred from system design.

**Research date:** 2026-03-28
**Valid until:** 2026-04-27
