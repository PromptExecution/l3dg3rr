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
                mcp_adapter::DOCUMENTS_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_documents_tool(global_service(), &arguments)
                }
                mcp_adapter::REVIEW_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_review_tool(global_service(), &arguments)
                }
                mcp_adapter::RECONCILIATION_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_reconciliation_tool(global_service(), &arguments)
                }
                mcp_adapter::WORKFLOW_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_workflow_tool(global_service(), &arguments)
                }
                mcp_adapter::AUDIT_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_audit_tool(global_service(), &arguments)
                }
                mcp_adapter::TAX_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_tax_tool(global_service(), &arguments)
                }
                mcp_adapter::ONTOLOGY_TOOL => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_tool(global_service(), &arguments)
                }
                "l3dg3rr_list_accounts" => mcp_adapter::handle_list_accounts(global_service()),
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
                "l3dg3rr_get_raw_context" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_get_raw_context(global_service(), &arguments)
                }
                "l3dg3rr_ontology_query_path" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_query_path(global_service(), &arguments)
                }
                "l3dg3rr_ontology_export_snapshot" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_export_snapshot(global_service(), &arguments)
                }
                "l3dg3rr_validate_reconciliation" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::dispatch_reconciliation(global_service(), "validate", &arguments)
                }
                "l3dg3rr_reconcile_postings" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::dispatch_reconciliation(global_service(), "reconcile", &arguments)
                }
                "l3dg3rr_commit_guarded" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::dispatch_reconciliation(global_service(), "commit", &arguments)
                }
                "l3dg3rr_hsm_transition" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::dispatch_hsm(global_service(), "transition", &arguments)
                }
                "l3dg3rr_hsm_status" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::dispatch_hsm(global_service(), "status", &arguments)
                }
                "l3dg3rr_hsm_resume" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::dispatch_hsm(global_service(), "resume", &arguments)
                }
                "l3dg3rr_event_history" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_event_history(global_service(), &arguments)
                }
                "l3dg3rr_event_replay" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_event_replay(global_service(), &arguments)
                }
                "l3dg3rr_tax_assist" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_tax_assist(global_service(), &arguments)
                }
                "l3dg3rr_tax_evidence_chain" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_tax_evidence_chain(global_service(), &arguments)
                }
                "l3dg3rr_tax_ambiguity_review" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_tax_ambiguity_review(global_service(), &arguments)
                }
                "l3dg3rr_classify_ingested" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_classify_ingested(global_service(), &arguments)
                }
                "l3dg3rr_query_flags" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_query_flags(global_service(), &arguments)
                }
                "l3dg3rr_query_audit_log" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_query_audit_log(global_service(), &arguments)
                }
                "l3dg3rr_classify_transaction" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_classify_transaction(global_service(), &arguments)
                }
                "l3dg3rr_reconcile_excel_classification" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_reconcile_excel_classification(global_service(), &arguments)
                }
                "l3dg3rr_get_schedule_summary" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_get_schedule_summary(global_service(), &arguments)
                }
                "l3dg3rr_export_cpa_workbook" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_export_cpa_workbook(global_service(), &arguments)
                }
                "l3dg3rr_ontology_upsert_entities" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_upsert_entities(global_service(), &arguments)
                }
                "l3dg3rr_ontology_upsert_edges" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_ontology_upsert_edges(global_service(), &arguments)
                }
                "l3dg3rr_plugin_info" => {
                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
                    mcp_adapter::handle_workflow_tool(
                        global_service(),
                        &json!({ "action": "plugin_info", "subcommand": arguments.get("subcommand").cloned().unwrap_or(Value::String("check".to_string())) }),
                    )
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
