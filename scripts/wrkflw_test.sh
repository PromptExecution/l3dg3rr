#!/usr/bin/env bash
# wrkflw test harness — validate all docgen visualization pipeline stages locally.
#
# Usage:
#   ./scripts/wrkflw_test.sh               # full pipeline
#   ./scripts/wrkflw_test.sh --validate     # validate workflow YAML only
#   ./scripts/wrkflw_test.sh --stage S1     # run single stage
#   ./scripts/wrkflw_test.sh --list         # list stages
#
# Exit codes:
#   0 — all stages pass
#   1 — one or more stages fail
#   2 — usage error or missing prerequisites

set -euo pipefail
export PATH="${HOME}/.cargo/bin:${PATH}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WRKFLW="${WRKFLW:-wrkflw}"
STAGES_YAML="${ROOT}/.github/workflows/wrkflw-docgen.yml"
DEFAULT_EMULATION="secure-emulation"

# ---- Color helpers ----
green() { printf "\033[32m%s\033[0m\n" "$*"; }
red() { printf "\033[31m%s\033[0m\n" "$*"; }
yellow() { printf "\033[33m%s\033[0m\n" "$*"; }

# ---- Prerequisites ----
check_prereqs() {
    if ! command -v "$WRKFLW" >/dev/null 2>&1; then
        red "error: wrkflw not found (set WRKFLW env or install via: cargo install wrkflw)"
        exit 2
    fi
    if [ ! -f "$STAGES_YAML" ]; then
        red "error: workflow file not found: $STAGES_YAML"
        exit 2
    fi
}

# ---- List stages ----
list_stages() {
    echo "wrkflw-docgen pipeline stages:"
    echo ""
    echo "  S1  stage-1-rhai-parser-tests     — mdbook-rhai-mermaid unit tests"
    echo "  S2  stage-2-iso-lint              — ledger-core iso lint tests"
    echo "  S3  stage-3-viz-tests             — ledger-core visualization/derive tests"
    echo "  S4  stage-4-legal-z3              — legal Z3 solver integration tests"
    echo "  S5  stage-5-docgen-build          — mdBook build with rhai→mermaid injection"
    echo "  S6  stage-6-kasuari-constraints   — Kasuari constraint solver tests"
    echo "  S7  stage-7-iso-objects           — HasVisualization impl lint"
    echo "  S8  stage-8-live-editor-js        — browser-side live-editor JS tests"
    echo "  S9  stage-9-xero-mcp              — Xero MCP smoke test"
    echo ""
    echo "Commands:"
    echo "  ./scripts/wrkflw_test.sh              — full pipeline (all stages)"
    echo "  ./scripts/wrkflw_test.sh --validate   — YAML validation only"
    echo "  ./scripts/wrkflw_test.sh --stage S5   — single stage (e.g. docgen build)"
    echo "  ./scripts/wrkflw_test.sh --list       — this list"
}

# ---- Stage name mapper ----
stage_name() {
    case "$1" in
        S1|s1) echo "stage-1-rhai-parser-tests" ;;
        S2|s2) echo "stage-2-iso-lint" ;;
        S3|s3) echo "stage-3-viz-tests" ;;
        S4|s4) echo "stage-4-legal-z3" ;;
        S5|s5) echo "stage-5-docgen-build" ;;
        S6|s6) echo "stage-6-kasuari-constraints" ;;
        S7|s7) echo "stage-7-iso-objects" ;;
        S8|s8) echo "stage-8-live-editor-js" ;;
        S9|s9) echo "stage-9-xero-mcp" ;;
        *) echo "unknown-stage-$1" ;;
    esac
}

# ---- Main ----
main() {
    check_prereqs

    local mode="full"
    local stage_arg=""

    while [ $# -gt 0 ]; do
        case "$1" in
            --validate|-v)
                mode="validate"
                shift
                ;;
            --stage|-s)
                mode="stage"
                stage_arg="$(stage_name "$2")"
                shift 2
                ;;
            --list|-l)
                list_stages
                exit 0
                ;;
            --help|-h)
                list_stages
                exit 0
                ;;
            *)
                red "unknown option: $1"
                list_stages
                exit 2
                ;;
        esac
    done

    case "$mode" in
        validate)
            yellow "=== Validating workflow: $STAGES_YAML ==="
            "$WRKFLW" validate "$STAGES_YAML"
            green "✓ Workflow validates"
            ;;
        stage)
            yellow "=== Running single stage: $stage_arg ==="
            "$WRKFLW" run --job "$stage_arg" --runtime "$DEFAULT_EMULATION" "$STAGES_YAML"
            green "✓ Stage $stage_arg complete"
            ;;
        full)
            yellow "=== wrkflw-docgen pipeline (full) ==="
            yellow "Step 1: Validate workflow"
            "$WRKFLW" validate "$STAGES_YAML"
            echo ""
            yellow "Step 2: Run all stages sequentially"
            "$WRKFLW" run --runtime "$DEFAULT_EMULATION" "$STAGES_YAML"
            green "✓ wrkflw-docgen pipeline complete"
            ;;
    esac
}

main "$@"
