# Phase 17 Validation Map

## Requirements Coverage

| Requirement | Task | Verification Command | Expected Result |
| --- | --- | --- | --- |
| EVT-01 | 17-01 Task 1 (RED) | `cargo test -p turbo-mcp --test events_contract -- --nocapture` | Fails before event contracts exist |
| EVT-01 | 17-01 Task 2 (GREEN-incremental) | `cargo test -p turbo-mcp --test events_contract -- --nocapture` | Compiles with failures limited to service wiring |
| EVT-01 | 17-01 Task 3 (GREEN) | `cargo test -p turbo-mcp --test events_contract -- --nocapture && cargo test -p turbo-mcp --test phase4_audit_integrity -- --nocapture && cargo test -p turbo-mcp --test hsm_contract -- --nocapture` | All passing |
| EVT-02 | 17-02 Task 1 (RED) | `cargo test -p turbo-mcp --test events_replay_contract -- --nocapture` | Fails before replay contracts exist |
| EVT-02 | 17-02 Task 2 (GREEN-incremental) | `cargo test -p turbo-mcp --test events_replay_contract -- --nocapture` | Projector contracts pass; service wiring still fails |
| EVT-02 | 17-02 Task 3 (GREEN) | `cargo test -p turbo-mcp --test events_contract -- --nocapture && cargo test -p turbo-mcp --test events_replay_contract -- --nocapture && cargo test -p turbo-mcp --test hsm_resume_contract -- --nocapture` | All passing |
| EVT-03 | 17-03 Task 1 (RED) | `cargo test -p turbo-mcp --test events_mcp_e2e -- --nocapture` | Fails before MCP event tool wiring exists |
| EVT-03 | 17-03 Task 2 (GREEN) | `cargo test -p turbo-mcp --test events_mcp_e2e -- --nocapture && cargo test -p turbo-mcp --test mcp_adapter_contract -- --nocapture` | All passing |
| EVT-03 | 17-03 Task 3 (Final verification) | `cargo test -p turbo-mcp --test events_contract -- --nocapture && cargo test -p turbo-mcp --test events_replay_contract -- --nocapture && cargo test -p turbo-mcp --test events_mcp_e2e -- --nocapture && cargo test -p turbo-mcp -- --nocapture` | All passing |

## Nyquist Execution Notes

- Deterministic reconstruction and filtered query behavior are covered at service and transport layers.
- Blocked query semantics are verified over MCP transport (`EventHistoryBlocked` + `time_range_invalid`).
- No `MISSING` entries.
