#!/usr/bin/env bash
set -euo pipefail

echo "[e2e] running behavior-driven MVP flow tests"
cargo test -p turbo-mcp --test e2e_bdd -- --nocapture
cargo test -p turbo-mcp --test e2e_mvp_flow -- --nocapture
cargo test -p turbo-mcp --test phase3_mcp_classification -- --nocapture
cargo test -p turbo-mcp --test phase4_audit_integrity -- --nocapture
cargo test -p turbo-mcp --test phase5_cpa_outputs -- --nocapture

echo "[e2e] all flow tests passed"

