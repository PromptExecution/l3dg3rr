#!/usr/bin/env bash
set -euo pipefail

JOURNAL_PATH="${JOURNAL_PATH:-/tmp/demo.beancount}"
WORKBOOK_PATH="${WORKBOOK_PATH:-/tmp/demo.xlsx}"
SOURCE_REF="${SOURCE_REF:-wf-2023-01.rkyv}"

cargo run -q -p ledgerr-mcp --bin ledgerr-mcp-server <<EOF
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"demo","version":"0.1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ledgerr_documents","arguments":{"action":"pipeline_status"}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"ledgerr_documents","arguments":{"action":"list_accounts"}}}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"ledgerr_documents","arguments":{"action":"ingest_pdf","pdf_path":"WF--BH-CHK--2023-01--statement.pdf","journal_path":"$JOURNAL_PATH","workbook_path":"$WORKBOOK_PATH","raw_context_bytes":[99,116,120],"extracted_rows":[{"account_id":"WF-BH-CHK","date":"2023-01-15","amount":"-42.11","description":"Coffee Shop","source_ref":"$SOURCE_REF"}]}}}
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"ledgerr_documents","arguments":{"action":"get_raw_context","rkyv_ref":"$SOURCE_REF"}}}
EOF

# Troubleshooting path
cat <<'EOF'
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ledgerr_workflow","arguments":{"action":"resume","state_marker":"invalid-checkpoint"}}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ledgerr_reconciliation","arguments":{"action":"commit","source_total":"100.00","extracted_total":"95.00","posting_amounts":["-95.00","95.00"]}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"ledgerr_audit","arguments":{"action":"event_history","time_start":"2026-12-31","time_end":"2026-01-01"}}}
EOF
