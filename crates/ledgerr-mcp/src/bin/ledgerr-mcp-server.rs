use std::io::{self, BufRead, Write};
use std::sync::OnceLock;

use ledgerr_mcp::{mcp_adapter, TurboLedgerService};
use serde_json::{json, Value};

fn main() {
    // Serve a minimal stdio MCP transport boundary for initialize/tools/list/tools/call.
    // Stdout is reserved for protocol payloads only.
    serve(io::stdin().lock(), io::stdout());
}

fn serve<R: BufRead, W: Write>(reader: R, mut writer: W) {
    for line in reader.lines() {
        let Ok(raw) = line else { continue };
        let Ok(request) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        if let Some(response) = handle_request(request) {
            if let Ok(serialized) = serde_json::to_string(&response) {
                let _ = writeln!(writer, "{serialized}");
                let _ = writer.flush();
            }
        }
    }
}

fn handle_request(request: Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-11-25",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "ledgerr-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),
        "notifications/initialized" => None,
        "tools/list" => {
            let tools = mcp_adapter::tool_descriptors();
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": { "tools": tools }
            }))
        }
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let result = match tool_name {
                mcp_adapter::LIST_ACCOUNTS_TOOL => {
                    mcp_adapter::handle_list_accounts(global_service())
                }
                mcp_adapter::DOCUMENT_INVENTORY_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_document_inventory(global_service(), &arguments)
                }
                "l3dg3rr_get_pipeline_status" => {
                    mcp_adapter::handle_pipeline_status(true, true, true, Vec::new())
                }
                "proxy_docling_ingest_pdf" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ingest_pdf(
                        global_service(),
                        &arguments,
                        Some(format!("mcp-call-{id}")),
                    )
                }
                "proxy_rustledger_ingest_statement_rows" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ingest_statement_rows(
                        global_service(),
                        &arguments,
                        Some(format!("mcp-call-{id}")),
                    )
                }
                mcp_adapter::GET_RAW_CONTEXT_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_get_raw_context(global_service(), &arguments)
                }
                mcp_adapter::ONTOLOGY_QUERY_PATH_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_query_path(global_service(), &arguments)
                }
                mcp_adapter::ONTOLOGY_EXPORT_SNAPSHOT_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_export_snapshot(global_service(), &arguments)
                }
                mcp_adapter::RECON_VALIDATE_TOOL
                | mcp_adapter::RECON_RECONCILE_TOOL
                | mcp_adapter::RECON_COMMIT_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::dispatch_reconciliation(global_service(), tool_name, &arguments)
                }
                mcp_adapter::HSM_TRANSITION_TOOL
                | mcp_adapter::HSM_STATUS_TOOL
                | mcp_adapter::HSM_RESUME_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::dispatch_hsm(global_service(), tool_name, &arguments)
                }
                mcp_adapter::EVENT_HISTORY_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_event_history(global_service(), &arguments)
                }
                mcp_adapter::EVENT_REPLAY_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_event_replay(global_service(), &arguments)
                }
                mcp_adapter::TAX_ASSIST_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_tax_assist(global_service(), &arguments)
                }
                mcp_adapter::TAX_EVIDENCE_CHAIN_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_tax_evidence_chain(global_service(), &arguments)
                }
                mcp_adapter::TAX_AMBIGUITY_REVIEW_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_tax_ambiguity_review(global_service(), &arguments)
                }
                // P0 tools
                mcp_adapter::CLASSIFY_INGESTED_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_classify_ingested(global_service(), &arguments)
                }
                mcp_adapter::QUERY_FLAGS_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_query_flags(global_service(), &arguments)
                }
                mcp_adapter::QUERY_AUDIT_LOG_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_query_audit_log(global_service(), &arguments)
                }
                // P1 tools
                mcp_adapter::CLASSIFY_TRANSACTION_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_classify_transaction(global_service(), &arguments)
                }
                mcp_adapter::RECONCILE_EXCEL_CLASSIFICATION_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_reconcile_excel_classification(global_service(), &arguments)
                }
                mcp_adapter::GET_SCHEDULE_SUMMARY_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_get_schedule_summary(global_service(), &arguments)
                }
                // P2 tools
                mcp_adapter::EXPORT_CPA_WORKBOOK_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_export_cpa_workbook(global_service(), &arguments)
                }
                mcp_adapter::ONTOLOGY_UPSERT_ENTITIES_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_upsert_entities(global_service(), &arguments)
                }
                mcp_adapter::ONTOLOGY_UPSERT_EDGES_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_upsert_edges(global_service(), &arguments)
                }
                mcp_adapter::PLUGIN_INFO_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_plugin_info(&arguments)
                }
                _ => mcp_adapter::unknown_tool_result(tool_name),
            };
            Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": result
            }))
        }
        _ => Some(mcp_adapter::protocol_method_not_found(id, method)),
    }
}

fn global_service() -> &'static TurboLedgerService {
    static SERVICE: OnceLock<TurboLedgerService> = OnceLock::new();
    SERVICE.get_or_init(|| {
        let manifest = "[session]\nworkbook_path=\"tax-ledger.xlsx\"\nactive_year=2023\n\n[accounts]\nWF-BH-CHK = { institution = \"Wells Fargo\", type = \"checking\", currency = \"USD\" }\n";
        TurboLedgerService::from_manifest_str(manifest).expect("default manifest must parse")
    })
}
