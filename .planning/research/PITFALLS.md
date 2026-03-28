# Domain Pitfalls

**Domain:** Local-first tax ledger for retroactive U.S. expat tax prep from PDFs to Excel
**Researched:** 2026-03-28

## Critical Pitfalls

### Pitfall 1: Decimal drift at Excel boundaries
**What goes wrong:** Amounts are parsed from spreadsheet cells through float-like paths, then re-rounded differently across ingest, classification, and schedule aggregation.
**Why it happens:** `calamine` exposes float/int/string/date cell variants and `rust_decimal` supports multiple rounding strategies; teams often rely on defaults.
**Consequences:** Schedule totals differ from source transactions; hard-to-explain penny deltas; CPA confidence drops.
**Prevention:**
- Enforce one canonical money parser: `DataType -> String normalization -> Amount::new(Decimal)` only.
- Ban `as_f64` and any float money conversion in code review/lints.
- Declare one explicit rounding policy per tax computation type and encode it in tests.
- Add golden roundtrip tests: PDF -> workbook -> reload -> schedule totals stable.
**Detection:**
- Any non-zero diff between recomputed `SCHED.*` totals and stored sheet totals.
- Re-ingesting unchanged source files changes cents-level totals.

### Pitfall 2: Non-deterministic transaction identity (false duplicates / missed duplicates)
**What goes wrong:** Same statement line hashes to different `TxId` across runs because canonicalization (whitespace/date/amount format) differs.
**Why it happens:** Hashing raw user-facing strings instead of canonical normalized fields.
**Consequences:** Duplicate rows, broken idempotent ingest, corrupted audit history.
**Prevention:**
- Define `TxId v1` canonical input contract (trimmed description, ISO date, fixed-scale amount string, normalized account id) and freeze it.
- Persist `txid_version` in `META.config`.
- Add property tests for deterministic hash output under formatting variants.
**Detection:**
- Re-running ingest on unchanged files produces new `TxId`s.
- Growth in TX rows without new source documents.

### Pitfall 3: Workbook contract drift (sheet/table/validation mismatch)
**What goes wrong:** Excel workbook no longer matches code assumptions (sheet rename, table rename, broken category validation list).
**Why it happens:** Excel is edited manually and schema invariants are not validated before write/read operations.
**Consequences:** Failed ingest writes, silent category corruption, broken pivots.
**Prevention:**
- Treat workbook schema as API: validate required sheet names/table names/named ranges at startup.
- Fail fast if contract check fails; provide one-click repair/migration command.
- Keep taxonomy source in enum + generated validation range, never ad-hoc text lists.
**Detection:**
- Startup schema validator failures.
- Categories appearing in TX rows that are absent from `CAT.taxonomy`.

### Pitfall 4: Audit trail that can be rewritten or becomes non-replayable
**What goes wrong:** `AUDIT.log` is edited, reordered, or lacks enough context to replay “who changed what and why.”
**Why it happens:** Append-only discipline is assumed but not cryptographically or operationally enforced.
**Consequences:** Auditability collapse during CPA review or dispute resolution.
**Prevention:**
- Append-only write path only; no update/delete operations for audit rows.
- Include session id, actor, reason/note, old/new value, and monotonic sequence.
- Add chained hash per audit row (`entry_hash` includes prior hash) and verify on load.
- Snapshot signed release metadata (git tag/version) for rule and schema state.
**Detection:**
- Sequence gaps, hash-chain breaks, or out-of-order timestamps.
- Mutations with missing actor or missing prior value.

### Pitfall 5: Human edit race conditions (Excel save vs ingest/classify write)
**What goes wrong:** Operator edits category/notes in Excel while service is writing, causing lost updates or stale overwrites.
**Why it happens:** No optimistic concurrency/version check on row updates.
**Consequences:** User distrust, rework, unresolved flags reappearing.
**Prevention:**
- Add row version/etag column; reject stale writes with explicit conflict status.
- Watch file changes (debounced) and run merge-aware diff before writes.
- Prefer command queue: one writer process owns workbook mutations.
**Detection:**
- Same row changed multiple times within seconds by different actors.
- User reports “my edit disappeared.”

### Pitfall 6: Rule evolution without reproducibility
**What goes wrong:** Rhai classification rules change, but prior classifications cannot be reproduced or compared against the old rule set.
**Why it happens:** Rule file version isn’t captured with each classification decision.
**Consequences:** Inconsistent year-over-year categorization and untraceable reclassifications.
**Prevention:**
- Store `rule_hash` and `rule_version` per classification event.
- Require dry-run diff before applying new rules to historical rows.
- Block production classification if rule file is dirty/uncommitted in git.
**Detection:**
- Reclassifying same tx under “same” environment yields different category/confidence.
- Missing rule metadata on audit entries.

### Pitfall 7: Unsafe access to rkyv sidecars
**What goes wrong:** Corrupt or malicious `.rkyv` sidecar data causes invalid reads or crashes when accessed without validation.
**Why it happens:** Fast path used without `bytecheck` validation for untrusted bytes.
**Consequences:** Pipeline instability; potential integrity/security risk in operator session.
**Prevention:**
- Enable `bytecheck` and validate sidecars on first access and at ingest.
- Store source PDF hash + sidecar hash and verify pair integrity.
- Quarantine invalid sidecars and continue ingest with flagged status.
**Detection:**
- rkyv validation failures or mismatch between source hash and sidecar hash.
- Repeated context retrieval errors for same document.

### Pitfall 8: Tax-domain evidence gaps (basis, FX, FBAR max logic)
**What goes wrong:** System computes tax summaries but lacks evidence required for CPA substantiation (crypto basis provenance, wallet transfer linkage, FBAR max conversion method).
**Why it happens:** Focus on classification output, not on evidence fields required by IRS/FinCEN rules.
**Consequences:** Manual rework, filing delays, or incorrect filings.
**Prevention:**
- Require provenance fields for taxable dispositions: acquisition lot id, basis source, disposal proceeds source.
- Separate transfer detection from taxable disposal classification.
- For FBAR, record max account value method and year-end Treasury FX source used.
- Fail export readiness checks if required evidence fields are missing.
**Detection:**
- Rows in SCHED.D without basis provenance.
- FBAR rows with missing FX source metadata.

## Moderate Pitfalls

### Pitfall 1: Over-reliance on HelixDB projection semantics
**What goes wrong:** Graph projection answers differ from workbook truth due to stale sync or mapping bugs.
**Prevention:** Rebuild projection from workbook on startup; include consistency checks (`row_count`, hash totals) before serving graph queries.

### Pitfall 2: Excel scale ceilings reached unexpectedly
**What goes wrong:** Large multi-year imports approach worksheet limits and degrade usability.
**Prevention:** Capacity checks before ingest; shard by account/year if projected row count nears Excel limits.

## Minor Pitfalls

### Pitfall 1: Container non-reproducibility
**What goes wrong:** Different builds produce behavior drift (base image updates, dependency variance).
**Prevention:** Multi-stage Dockerfile, pinned base image digests, CI build/test before tagging release.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Phase 0: Input readiness | PDFs not normalized to naming convention; missing manifest metadata | Add preflight scanner that rejects non-conforming filenames and generates actionable rename report |
| Phase 1: Ledger core + Excel roundtrip | Decimal drift, TxId nondeterminism, schema drift, audit gaps | Ship invariant test suite first (money, hash determinism, schema validator, audit chain validator) |
| Phase 1: MCP tool surface | Tool signatures drift from agent workflow | Freeze MCP contract early; version tool schema and add compatibility tests |
| Phase 2: Helix projection | Graph answers stale/inconsistent with workbook | Projection rebuild + parity checks on every startup and before graph query responses |
| Phase 3: API/UI + ops | Concurrent edits and stale writes via UI/API | Enforce single-writer queue and optimistic concurrency conflict handling |
| Release/versioning | Rule/schema changes shipped without auditable change narrative | Use cocogitto conventional commits + automated bump/changelog; require release note sections for tax logic/rule/schema changes |

## Sources

- calamine `DataType` and cell-type handling (docs.rs): https://docs.rs/calamine/latest/calamine/trait.DataType.html
- rust_decimal precision and rounding strategy (docs.rs): https://docs.rs/rust_decimal/latest/rust_decimal/enum.RoundingStrategy.html
- rust_xlsxwriter capabilities/constraints (write-only, validations) (docs.rs): https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/
- rust_xlsxwriter validation list limit (255 chars) (docs.rs): https://docs.rs/rust_xlsxwriter/latest/rust_xlsxwriter/struct.DataValidation.html
- Excel worksheet limits (Microsoft): https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3
- Structured references behavior for Excel tables (Microsoft): https://support.microsoft.com/en-au/office/using-structured-references-with-excel-tables-f5ed2452-2337-4f71-bed3-c8ae6d2b276e
- rkyv validation guidance (`bytecheck`) (official docs): https://rkyv.org/validation.html
- Rhai runtime limits for sandboxing scripts (`set_max_operations`, `set_max_call_levels`) (docs.rs): https://docs.rs/rhai/latest/rhai/struct.Engine.html
- IRS Publication 583 recordkeeping expectations: https://www.irs.gov/publications/p583
- IRS digital asset FAQ (basis, wallet transfer, recordkeeping): https://www.irs.gov/individuals/international-taxpayers/frequently-asked-questions-on-virtual-currency-transactions
- FinCEN FBAR filing page (includes maximum account value + exchange rate guidance links): https://www.fincen.gov/report-foreign-bank-and-financial-accounts
- Cocogitto automatic versioning/changelog for auditable releases: https://docs.cocogitto.io/guide/bump
