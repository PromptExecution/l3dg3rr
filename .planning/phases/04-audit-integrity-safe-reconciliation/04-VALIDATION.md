# Phase 04 Validation Strategy

## Scope

- Append-only audit entry creation for classification mutations.
- Excel reconciliation path recording equivalent audit evidence.
- Decimal-safe parsing for confidence and amount invariants.
- Invariant checks for tx-id determinism and schema validation.

## Validation Matrix

| Requirement | What to validate | Primary evidence |
| --- | --- | --- |
| AUD-01 | Mutation writes append-only audit entries | `aud_01_mcp_02_classify_transaction_records_append_only_audit_entries` |
| AUD-02 | Excel-style edit reconciliation records audit entries | `aud_02_excel_reconcile_path_writes_matching_audit_records` |
| AUD-03 | Decimal-safe parsing for mutation inputs | `aud_03_decimal_safe_amount_and_confidence_validation_rejects_invalid_values` |
| AUD-04 | Invariant violations are detected and rejected | `aud_04_invariant_checks_detect_schema_or_txid_violations` |
| MCP-02 | MCP classify transaction mutation path available and audited | `aud_01_*` and `phase4_keeps_phase3_open_flag_query_behavior` |

## Execution Steps

1. `cargo test -p turbo-mcp --test phase4_audit_integrity -- --nocapture`
2. `cargo test --workspace -- --nocapture`

## Pass Criteria

- Phase 4 suite passes and workspace regression remains green.

