# Phase 05 Validation Strategy

## Scope

- CPA workbook export sheet coverage for transactions, taxonomy, and review flags.
- Schedule C/D/E and FBAR aggregation outputs.
- MCP schedule summary retrieval contract.

## Validation Matrix

| Requirement | What to validate | Primary evidence |
| --- | --- | --- |
| WB-01 | Transaction sheets are exportable in workbook output | `wb_01_02_03_export_cpa_workbook_materializes_tx_and_flag_sheets` |
| WB-02 | Taxonomy/category sheet is present for classification workflow | same test (`CAT.taxonomy`) |
| WB-03 | Open/resolved flags are materialized in workbook sheets | same test (`FLAGS.open`, `FLAGS.resolved`) |
| TAX-01 | Schedule C yearly summary available | `tax_01_02_03_04_and_mcp_04_schedule_summary_are_available_by_year` |
| TAX-02 | Schedule D yearly summary available | same test |
| TAX-03 | Schedule E yearly summary available | same test |
| TAX-04 | FBAR account yearly summary available | same test |
| MCP-04 | MCP get_schedule_summary(year,schedule) provides totals | same test |

## Execution Steps

1. `cargo test -p turbo-mcp --test phase5_cpa_outputs -- --nocapture`
2. `cargo test --workspace -- --nocapture`

## Pass Criteria

- Phase 5 suite and workspace suite pass with no regressions.

