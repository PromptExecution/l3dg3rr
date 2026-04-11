#!/usr/bin/env bash
set -euo pipefail

echo "[e2e] running behavior-driven MVP flow tests"
for test in e2e_bdd e2e_mvp_flow phase3_mcp_classification phase4_audit_integrity phase5_cpa_outputs; do
    cargo test -p ledgerr-mcp --test "$test" -- --nocapture
done

echo "[e2e] all flow tests passed"

