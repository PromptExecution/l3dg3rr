# Phase 18 Validation Map

## Requirements Coverage

| Requirement | Task | Verification Command | Expected Result |
| --- | --- | --- | --- |
| TAXA-01, TAXA-03 | 18-01 Task 1 | `cargo test -p turbo-mcp --test interface -- --nocapture` | Service contracts compile and export new tax-assist interfaces |
| TAXA-01, TAXA-03 | 18-01 Task 2 (RED) | `cargo test -p turbo-mcp --test tax_assist_contract -- --nocapture` | Fails before tax-assist deterministic/ambiguity implementation |
| TAXA-01, TAXA-03 | 18-01 Task 3 (GREEN) | `cargo test -p turbo-mcp --test tax_assist_contract -- --nocapture && cargo test -p turbo-mcp --test reconciliation_contract -- --nocapture` | Passes with reconciled ontology-derived output and explicit ambiguity review payload |
| TAXA-02 | 18-02 Task 1 (RED) | `cargo test -p turbo-mcp --test tax_evidence_chain_contract -- --nocapture` | Fails before ambiguity/provenance evidence-chain links are implemented |
| TAXA-02 | 18-02 Task 2 (GREEN) | `cargo test -p turbo-mcp --test tax_evidence_chain_contract -- --nocapture && cargo test -p turbo-mcp --test events_replay_contract -- --nocapture` | Passes with deterministic source->events->current_state payload and provenance links |
| TAXA-01, TAXA-02, TAXA-03 | 18-03 Task 1 (RED) | `cargo test -p turbo-mcp --test tax_assist_mcp_e2e -- --nocapture` | Fails before MCP tax tool catalog/call wiring exists |
| TAXA-01, TAXA-02, TAXA-03 | 18-03 Task 2 (GREEN) | `cargo test -p turbo-mcp --test tax_assist_mcp_e2e -- --nocapture && cargo test -p turbo-mcp --test mcp_adapter_contract -- --nocapture` | Passes with deterministic MCP list/call transport envelopes |
| TAXA-01, TAXA-02, TAXA-03 | 18-03 Task 3 (Final) | `cargo test -p turbo-mcp --test tax_assist_contract -- --nocapture && cargo test -p turbo-mcp --test tax_evidence_chain_contract -- --nocapture && cargo test -p turbo-mcp --test tax_assist_mcp_e2e -- --nocapture && cargo test -p turbo-mcp -- --nocapture` | All passing |

## Nyquist Execution Notes

- Tax outputs are gated by reconciliation readiness and derived from ontology evidence rows only.
- Evidence chain payload keeps explicit `source`, `events`, `current_state`, and ambiguity linkage.
- MCP transport payloads are concise deterministic envelopes with explicit blocked/success states.
- No `MISSING` placeholders.
