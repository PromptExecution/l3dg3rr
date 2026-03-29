use std::path::PathBuf;

use ledger_core::ingest::{deterministic_tx_id, TransactionInput};
use serde::Serialize;
use serde_json::{json, Value};

use crate::{IngestPdfRequest, ToolError, TurboLedgerTools};

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

pub fn parse_ingest_pdf_request(arguments: &Value) -> Result<IngestPdfRequest, ToolError> {
    let pdf_path = required_str(arguments, "pdf_path")?.to_string();
    let journal_path = PathBuf::from(required_str(arguments, "journal_path")?);
    let workbook_path = PathBuf::from(required_str(arguments, "workbook_path")?);
    let raw_context_bytes = parse_optional_bytes(arguments.get("raw_context_bytes"))?;
    let extracted_rows = parse_rows(arguments.get("extracted_rows"))?;

    Ok(IngestPdfRequest {
        pdf_path,
        journal_path,
        workbook_path,
        raw_context_bytes,
        extracted_rows,
    })
}

pub fn ingest_pdf_tool_result<T: TurboLedgerTools>(
    service: &T,
    arguments: &Value,
    backend_call_id: Option<String>,
) -> Value {
    let request = match parse_ingest_pdf_request(arguments) {
        Ok(request) => request,
        Err(err) => {
            return json!({
                "content": [{
                    "type": "json",
                    "json": map_tool_error(&err)
                }],
                "isError": true
            });
        }
    };

    let canonical_rows = normalize_rows_with_provenance(
        "docling",
        "ingest_pdf",
        Some(env!("CARGO_PKG_VERSION")),
        backend_call_id.as_deref(),
        request.extracted_rows.clone(),
    );

    match service.ingest_pdf(request.clone()) {
        Ok(response) => {
            let tx_ids = if response.tx_ids.is_empty() {
                request
                    .extracted_rows
                    .iter()
                    .map(|row| deterministic_tx_id(row))
                    .collect::<Vec<_>>()
            } else {
                response.tx_ids
            };
            json!({
                "content": [{
                    "type": "json",
                    "json": {
                        "inserted_count": response.inserted_count,
                        "tx_ids": tx_ids,
                        "canonical_rows": canonical_rows,
                    }
                }],
                "isError": false
            })
        }
        Err(err) => json!({
            "content": [{
                "type": "json",
                "json": map_tool_error(&err)
            }],
            "isError": true
        }),
    }
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

fn required_str<'a>(obj: &'a Value, key: &str) -> Result<&'a str, ToolError> {
    obj.get(key).and_then(Value::as_str).ok_or_else(|| {
        ToolError::InvalidInput(format!("missing or invalid `{key}` in tool arguments"))
    })
}

fn parse_optional_bytes(value: Option<&Value>) -> Result<Option<Vec<u8>>, ToolError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                let num = item.as_u64().ok_or_else(|| {
                    ToolError::InvalidInput("raw_context_bytes must be an array of bytes".to_string())
                })?;
                u8::try_from(num).map_err(|_| {
                    ToolError::InvalidInput(
                        "raw_context_bytes values must be in range 0..=255".to_string(),
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        _ => Err(ToolError::InvalidInput(
            "raw_context_bytes must be null or an array of bytes".to_string(),
        )),
    }
}

fn parse_rows(value: Option<&Value>) -> Result<Vec<TransactionInput>, ToolError> {
    let rows = value
        .and_then(Value::as_array)
        .ok_or_else(|| ToolError::InvalidInput("missing or invalid `extracted_rows`".to_string()))?;

    rows.iter()
        .map(|row| {
            Ok(TransactionInput {
                account_id: required_str(row, "account_id")?.to_string(),
                date: required_str(row, "date")?.to_string(),
                amount: required_str(row, "amount")?.to_string(),
                description: required_str(row, "description")?.to_string(),
                source_ref: required_str(row, "source_ref")?.to_string(),
            })
        })
        .collect()
}
