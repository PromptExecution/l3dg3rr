#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

MODE="${1:-basic}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

SOURCE_REF="$TMP_DIR/WF--BH-CHK--2023-01--statement.rkyv"
JOURNAL_PATH="$TMP_DIR/ledger.beancount"
WORKBOOK_PATH="$TMP_DIR/tax-ledger.xlsx"

run_basic() {
  cat <<EOF | cargo run -q -p ledgerr-mcp --bin ledgerr-mcp-server
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"mcp-cli-basic","version":"0.1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ledgerr_documents","arguments":{"action":"pipeline_status"}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"ledgerr_documents","arguments":{"action":"list_accounts"}}}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"ledgerr_documents","arguments":{"action":"ingest_pdf","pdf_path":"WF--BH-CHK--2023-01--statement.pdf","journal_path":"$JOURNAL_PATH","workbook_path":"$WORKBOOK_PATH","raw_context_bytes":[99,116,120],"extracted_rows":[{"account_id":"WF-BH-CHK","date":"2023-01-15","amount":"-42.11","description":"Coffee Shop","source_ref":"$SOURCE_REF"}]}}}
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"ledgerr_documents","arguments":{"action":"get_raw_context","rkyv_ref":"$SOURCE_REF"}}}
EOF
}

run_spinning_wheels() {
  cat <<EOF | cargo run -q -p ledgerr-mcp --bin ledgerr-mcp-server
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"mcp-cli-spinning","version":"0.1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ledgerr_workflow","arguments":{"action":"resume","state_marker":"invalid-checkpoint"}}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ledgerr_reconciliation","arguments":{"action":"commit","source_total":"100.00","extracted_total":"95.00","posting_amounts":["-95.00","95.00"]}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"ledgerr_audit","arguments":{"action":"event_history","time_start":"2026-12-31","time_end":"2026-01-01"}}}
EOF
}

case "$MODE" in
  basic)
    run_basic
    ;;
  spinning-wheels)
    run_spinning_wheels
    ;;
  *)
    echo "usage: $0 [basic|spinning-wheels]" >&2
    exit 2
    ;;
esac
