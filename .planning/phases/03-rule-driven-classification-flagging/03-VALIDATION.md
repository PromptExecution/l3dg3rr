# Phase 03 Validation Strategy

## Scope

- Runtime Rhai classification loading/execution.
- Category + confidence assignment for transactions.
- Review-flag emission for low-confidence/policy outcomes.
- MCP rule-test and flag-query contracts.

## Validation Matrix

| Requirement | What to validate | Primary evidence |
| --- | --- | --- |
| CLSF-01 | Rule file loads/runs at runtime without recompile | `phase3_classification` + `phase3_mcp_classification` |
| CLSF-02 | Category and confidence assigned per transaction | Classification assertions in both suites |
| CLSF-03 | Low-confidence output creates open review flag | `query_flags` assertions |
| CLSF-04 | Candidate Rhai rule tests on sample tx | `run_rule_from_file` and `run_rhai_rule` assertions |
| MCP-03 | MCP `query_flags(year,status)` returns review queue | `mcp_03_query_flags_returns_review_queue_by_year_and_status` |
| MCP-07 | MCP `run_rhai_rule(rule_file,sample_tx)` validates rule output | `mcp_07_run_rhai_rule_validates_candidate_rule_on_sample_tx` |

## Execution Steps

1. `cargo test -p ledger-core --test phase3_classification -- --nocapture`
2. `cargo test -p turbo-mcp --test phase3_mcp_classification -- --nocapture`
3. `cargo test --workspace -- --nocapture`

## Pass Criteria

- All commands pass.
- Tests demonstrate deterministic classification output and correct review-queue filtering.
