use ledger_core::ingest::TransactionInput;
use serde::Serialize;
use serde_json::{json, Value};

use crate::{ToolError, TurboLedgerTools};

pub const MCP_LIFECYCLE_METHODS: &[&str] = &["initialize", "tools/list", "tools/call"];

pub fn tool_catalog() -> Vec<String> {
    vec![
        "proxy_docling_ingest_pdf".to_string(),
        "proxy_rustledger_ingest_statement_rows".to_string(),
        "l3dg3rr_get_pipeline_status".to_string(),
        "tools/list".to_string(),
        "tools/call".to_string(),
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PipelineStatusResponse {
    pub status: String,
    pub blockers: Vec<String>,
    pub next_hint: String,
}

pub fn get_pipeline_status(
    manifest_ready: bool,
    rustledger_ready: bool,
    docling_ready: bool,
    mut blockers: Vec<String>,
) -> PipelineStatusResponse {
    if !manifest_ready {
        blockers.push("manifest_unavailable".to_string());
    }
    if !rustledger_ready {
        blockers.push("rustledger_unreachable".to_string());
    }
    if !docling_ready {
        blockers.push("docling_unreachable".to_string());
    }
    blockers.sort();
    blockers.dedup();

    if blockers.is_empty() {
        PipelineStatusResponse {
            status: "ready".to_string(),
            blockers,
            next_hint: "call_proxy_ingest_pdf".to_string(),
        }
    } else {
        PipelineStatusResponse {
            status: "blocked".to_string(),
            blockers,
            next_hint: "resolve_blockers_then_retry".to_string(),
        }
    }
}

pub fn normalize_rows_with_provenance(
    provider: &str,
    backend_tool: &str,
    backend_version: Option<&str>,
    backend_call_id: Option<&str>,
    rows: Vec<TransactionInput>,
) -> Vec<Value> {
    rows.into_iter()
        .map(|row| {
            let account_id = row.account_id;
            let currency = infer_currency(&account_id);
            json!({
                "account": account_id,
                "date": row.date,
                "amount": row.amount,
                "description": row.description,
                "currency": currency,
                "source_ref": row.source_ref,
                "provider": provider,
                "backend_tool": backend_tool,
                "backend_version": backend_version,
                "backend_call_id": backend_call_id,
            })
        })
        .collect()
}

pub fn map_tool_error(error: &ToolError) -> Value {
    match error {
        ToolError::InvalidInput(message) => json!({
            "isError": true,
            "error_type": "InvalidInput",
            "message": message,
        }),
        ToolError::Internal(message) => json!({
            "isError": true,
            "error_type": "Internal",
            "message": message,
        }),
    }
}

pub fn unknown_tool_result(tool_name: &str) -> Value {
    json!({
        "content": [{
            "type": "json",
            "json": {
                "isError": true,
                "error_type": "InvalidInput",
                "message": format!("unknown tool: {tool_name}")
            }
        }],
        "isError": true
    })
}

pub fn protocol_method_not_found(id: Value, method: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32601,
            "message": format!("method not found: {method}")
        }
    })
}

pub struct McpAdapter<'a, T: TurboLedgerTools> {
    service: &'a T,
}

impl<'a, T: TurboLedgerTools> McpAdapter<'a, T> {
    pub fn new(service: &'a T) -> Self {
        Self { service }
    }

    pub fn provider_passthrough_ping(&self) -> Value {
        let _ = &self.service;
        json!({
            "provider": "rustledger",
            "backend_tool": "list_accounts",
            "backend_version": serde_json::Value::Null,
            "backend_call_id": serde_json::Value::Null,
            "status": "ok",
        })
    }
}

fn infer_currency(account_id: &str) -> String {
    let upper = account_id.to_ascii_uppercase();
    if upper.contains("BTC") {
        "BTC".to_string()
    } else {
        "USD".to_string()
    }
}
