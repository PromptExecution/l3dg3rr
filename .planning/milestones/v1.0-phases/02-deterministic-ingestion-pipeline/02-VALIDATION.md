# Phase 02 Validation Strategy

This file defines Nyquist-facing validation for Phase 2 (`02-deterministic-ingestion-pipeline`) and focuses on practical, automatable evidence.

## Scope

- Deterministic ingest path from contract-valid PDF filename to persisted outputs.
- Replay safety (idempotent re-ingest, no duplicate writes).
- `source_ref` / `.rkyv` evidence persistence and retrieval.
- MCP contracts for `ingest_pdf` and `get_raw_context`.

## Validation Matrix

| Requirement | What to validate | Primary evidence |
| --- | --- | --- |
| ING-01 | Contract-valid ingest writes Beancount entry and workbook TX projection | `phase2_ingest_pipeline_remaining` passing assertions |
| ING-02 | Re-ingest yields stable tx IDs and no duplicate journal/workbook rows | Replay/idempotency assertions in ingest tests |
| ING-03 | Ingest persists `.rkyv` source snapshot reference | `source_ref` and `.rkyv` reference assertions |
| ING-04 | Ingested transaction evidence is traceable by reference | Transaction metadata + retrievability checks |
| MCP-01 | `ingest_pdf(path)` returns deterministic tx IDs from real ingest execution | `phase2_mcp_contract_remaining` and `interface` tests |
| MCP-05 | `get_raw_context(rkyv_ref)` returns stored source bytes | MCP raw-context retrieval assertions |

## Execution Steps

1. Run core ingest requirement tests:
   - `cargo test -p ledger-core phase2_ingest -- --nocapture`
   - `cargo test -p ledger-core phase2_rustledger_journal -- --nocapture`
   - `cargo test -p ledger-core phase2_ingest_pipeline_remaining -- --nocapture`
2. Run MCP requirement tests:
   - `cargo test -p turbo-mcp phase2_mcp_contract_remaining -- --nocapture`
   - `cargo test -p turbo-mcp -- --nocapture`
3. Run workspace regression gate:
   - `cargo test --workspace -- --nocapture`
4. Confirm requirement-tagged coverage exists:
   - `rg -n "ING-01|ING-02|ING-03|ING-04" crates/ledger-core/tests/phase2_ingest_pipeline_remaining.rs`
   - `rg -n "MCP-01|MCP-05" crates/turbo-mcp/tests/phase2_mcp_contract_remaining.rs crates/turbo-mcp/tests/interface.rs`

## Pass/Fail Criteria

- Pass when all commands above exit successfully and requirement tags are present in the mapped tests.
- Fail on any non-zero command, missing requirement-tagged assertions, or evidence retrieval mismatch (`source_ref` does not resolve).

## Nyquist Compliance Notes

- Uses explicit, reproducible commands with deterministic expected behavior.
- Provides requirement-to-test traceability for ING-01..04 and MCP-01/MCP-05.
- Verifies both positive behavior (ingest + retrieval) and safety behavior (replay/idempotency).
