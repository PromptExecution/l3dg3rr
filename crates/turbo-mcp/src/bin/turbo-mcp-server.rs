use std::io::{self, BufRead, Write};
use std::sync::OnceLock;

use serde_json::{json, Value};
use turbo_mcp::{mcp_adapter, TurboLedgerService};

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
                    "name": "turbo-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })),
        "notifications/initialized" => None,
        "tools/list" => {
            let tools: Vec<Value> = mcp_adapter::tool_catalog()
                .into_iter()
                .map(|name| json!({ "name": name }))
                .collect();
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
                "l3dg3rr_get_pipeline_status" => {
                    let status = mcp_adapter::get_pipeline_status(true, true, true, Vec::new());
                    json!({
                        "content": [{
                            "type": "json",
                            "json": status
                        }],
                        "isError": false
                    })
                }
                "proxy_docling_ingest_pdf" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::ingest_pdf_tool_result(
                        global_service(),
                        &arguments,
                        Some(format!("mcp-call-{id}")),
                    )
                }
                "proxy_rustledger_ingest_statement_rows" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::ingest_statement_rows_tool_result(
                        global_service(),
                        &arguments,
                        Some(format!("mcp-call-{id}")),
                    )
                }
                mcp_adapter::ONTOLOGY_QUERY_PATH_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::ontology_query_path_tool_result(global_service(), &arguments)
                }
                mcp_adapter::ONTOLOGY_EXPORT_SNAPSHOT_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::ontology_export_snapshot_tool_result(&arguments)
                }
                mcp_adapter::RECON_VALIDATE_TOOL
                | mcp_adapter::RECON_RECONCILE_TOOL
                | mcp_adapter::RECON_COMMIT_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::reconciliation_tool_result(global_service(), tool_name, &arguments)
                }
                mcp_adapter::HSM_TRANSITION_TOOL
                | mcp_adapter::HSM_STATUS_TOOL
                | mcp_adapter::HSM_RESUME_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::hsm_tool_result(global_service(), tool_name, &arguments)
                }
                mcp_adapter::EVENT_HISTORY_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::event_history_tool_result(global_service(), &arguments)
                }
                mcp_adapter::EVENT_REPLAY_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::event_replay_tool_result(global_service(), &arguments)
                }
                mcp_adapter::TAX_ASSIST_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::tax_assist_tool_result(global_service(), &arguments)
                }
                mcp_adapter::TAX_EVIDENCE_CHAIN_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::tax_evidence_chain_tool_result(global_service(), &arguments)
                }
                mcp_adapter::TAX_AMBIGUITY_REVIEW_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::tax_ambiguity_review_tool_result(global_service(), &arguments)
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
        let manifest = "[session]\nworkbook_path=\"tax-ledger.xlsx\"\nactive_year=2023\n";
        TurboLedgerService::from_manifest_str(manifest).expect("default manifest must parse")
    })
}
