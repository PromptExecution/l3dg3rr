use std::path::PathBuf;

use ledger_core::ingest::{deterministic_tx_id, TransactionInput};
use serde::Serialize;
use serde_json::{json, Value};

use crate::{
    EventHistoryFilter, HsmResumeRequest, HsmStatusRequest, HsmTransitionRequest, IngestPdfRequest,
    IngestStatementRowsRequest, OntologyQueryPathRequest, OntologyStore, ReconciliationStageRequest,
    ReplayLifecycleRequest, ToolError, TurboLedgerService, TurboLedgerTools,
};

pub const ONTOLOGY_QUERY_PATH_TOOL: &str = "l3dg3rr_ontology_query_path";
pub const ONTOLOGY_EXPORT_SNAPSHOT_TOOL: &str = "l3dg3rr_ontology_export_snapshot";
pub const RECON_VALIDATE_TOOL: &str = "l3dg3rr_validate_reconciliation";
pub const RECON_RECONCILE_TOOL: &str = "l3dg3rr_reconcile_postings";
pub const RECON_COMMIT_TOOL: &str = "l3dg3rr_commit_guarded";
pub const HSM_TRANSITION_TOOL: &str = "l3dg3rr_hsm_transition";
pub const HSM_STATUS_TOOL: &str = "l3dg3rr_hsm_status";
pub const HSM_RESUME_TOOL: &str = "l3dg3rr_hsm_resume";
pub const EVENT_REPLAY_TOOL: &str = "l3dg3rr_event_replay";
pub const EVENT_HISTORY_TOOL: &str = "l3dg3rr_event_history";

pub const MCP_LIFECYCLE_METHODS: &[&str] = &["initialize", "tools/list", "tools/call"];

pub fn tool_catalog() -> Vec<String> {
    vec![
        "proxy_docling_ingest_pdf".to_string(),
        "proxy_rustledger_ingest_statement_rows".to_string(),
        "l3dg3rr_get_pipeline_status".to_string(),
        ONTOLOGY_QUERY_PATH_TOOL.to_string(),
        ONTOLOGY_EXPORT_SNAPSHOT_TOOL.to_string(),
        RECON_VALIDATE_TOOL.to_string(),
        RECON_RECONCILE_TOOL.to_string(),
        RECON_COMMIT_TOOL.to_string(),
        HSM_TRANSITION_TOOL.to_string(),
        HSM_STATUS_TOOL.to_string(),
        HSM_RESUME_TOOL.to_string(),
        EVENT_REPLAY_TOOL.to_string(),
        EVENT_HISTORY_TOOL.to_string(),
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
    let extracted_rows = parse_rows(arguments.get("extracted_rows"), "extracted_rows")?;

    Ok(IngestPdfRequest {
        pdf_path,
        journal_path,
        workbook_path,
        raw_context_bytes,
        extracted_rows,
    })
}

pub fn parse_ingest_statement_rows_request(
    arguments: &Value,
) -> Result<IngestStatementRowsRequest, ToolError> {
    let journal_path = PathBuf::from(required_str(arguments, "journal_path")?);
    let workbook_path = PathBuf::from(required_str(arguments, "workbook_path")?);
    let rows = parse_rows(arguments.get("rows"), "rows")?;

    Ok(IngestStatementRowsRequest {
        journal_path,
        workbook_path,
        rows,
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

pub fn ingest_statement_rows_tool_result<T: TurboLedgerTools>(
    service: &T,
    arguments: &Value,
    backend_call_id: Option<String>,
) -> Value {
    let request = match parse_ingest_statement_rows_request(arguments) {
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
        "rustledger",
        "ingest_statement_rows",
        Some(env!("CARGO_PKG_VERSION")),
        backend_call_id.as_deref(),
        request.rows.clone(),
    );

    match service.ingest_statement_rows(request.clone()) {
        Ok(response) => {
            let tx_ids = if response.tx_ids.is_empty() {
                request
                    .rows
                    .iter()
                    .map(deterministic_tx_id)
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
                        "provider": "rustledger",
                        "backend_tool": "ingest_statement_rows",
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

pub fn ontology_query_path_tool_result(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_ontology_query_path_request(arguments) {
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

    match service.ontology_query_path_tool(request) {
        Ok(response) => json!({
            "content": [{
                "type": "json",
                "json": {
                    "nodes": response.nodes,
                    "edges": response.edges,
                }
            }],
            "isError": false
        }),
        Err(err) => json!({
            "content": [{
                "type": "json",
                "json": map_tool_error(&err)
            }],
            "isError": true
        }),
    }
}

pub fn ontology_export_snapshot_tool_result(arguments: &Value) -> Value {
    let ontology_path = match parse_ontology_path(arguments) {
        Ok(path) => path,
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

    match OntologyStore::load(&ontology_path) {
        Ok(store) => json!({
            "content": [{
                "type": "json",
                "json": {
                    "entities": store.entities,
                    "edges": store.edges,
                    "snapshot": {
                        "entity_count": store.entities.len(),
                        "edge_count": store.edges.len(),
                    }
                }
            }],
            "isError": false
        }),
        Err(err) => json!({
            "content": [{
                "type": "json",
                "json": map_tool_error(&err)
            }],
            "isError": true
        }),
    }
}

pub fn reconciliation_tool_result(
    service: &TurboLedgerService,
    tool_name: &str,
    arguments: &Value,
) -> Value {
    let request = match parse_reconciliation_stage_request(arguments) {
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

    let response = match tool_name {
        RECON_VALIDATE_TOOL => service.validate_reconciliation_stage_tool(request),
        RECON_RECONCILE_TOOL => service.reconcile_reconciliation_stage_tool(request),
        RECON_COMMIT_TOOL => service.commit_reconciliation_stage_tool(request),
        _ => {
            return unknown_tool_result(tool_name);
        }
    };

    match response {
        Ok(stage_response) => {
            let blocked = stage_response.status == "blocked";
            let stage = stage_response.stage;
            let status = stage_response.status;
            let stage_marker = stage_response.stage_marker;
            let blocked_reasons = stage_response.blocked_reasons;
            let diagnostics = stage_response
                .diagnostics
                .into_iter()
                .map(|diag| json!({ "key": diag.key, "message": diag.message }))
                .collect::<Vec<_>>();

            let payload = if blocked {
                json!({
                    "isError": true,
                    "error_type": "ReconciliationBlocked",
                    "message": format!("{stage} blocked by reconciliation guardrails"),
                    "stage": stage,
                    "status": status,
                    "stage_marker": stage_marker,
                    "blocked_reasons": blocked_reasons,
                    "diagnostics": diagnostics,
                })
            } else {
                json!({
                    "stage": stage,
                    "status": status,
                    "stage_marker": stage_marker,
                    "blocked_reasons": blocked_reasons,
                    "diagnostics": diagnostics,
                })
            };

            json!({
                "content": [{
                    "type": "json",
                    "json": payload
                }],
                "isError": blocked
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

pub fn hsm_tool_result(service: &TurboLedgerService, tool_name: &str, arguments: &Value) -> Value {
    match tool_name {
        HSM_TRANSITION_TOOL => {
            let request = match parse_hsm_transition_request(arguments) {
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

            match service.hsm_transition_tool(request) {
                Ok(response) => {
                    let blocked = response.status == "blocked";
                    let payload = if blocked {
                        json!({
                            "isError": true,
                            "error_type": "HsmTransitionBlocked",
                            "message": "hsm transition blocked by lifecycle guard",
                            "state": response.state,
                            "substate": response.substate,
                            "status": response.status,
                            "guard_reason": response.guard_reason,
                            "transition_evidence": response.transition_evidence,
                            "state_marker": response.state_marker,
                        })
                    } else {
                        json!({
                            "state": response.state,
                            "substate": response.substate,
                            "status": response.status,
                            "guard_reason": response.guard_reason,
                            "transition_evidence": response.transition_evidence,
                            "state_marker": response.state_marker,
                        })
                    };
                    json!({
                        "content": [{
                            "type": "json",
                            "json": payload
                        }],
                        "isError": blocked
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
        HSM_STATUS_TOOL => match service.hsm_status_tool(HsmStatusRequest) {
            Ok(response) => json!({
                "content": [{
                    "type": "json",
                    "json": {
                        "state": response.state,
                        "substate": response.substate,
                        "display_state": response.display_state,
                        "next_hint": response.next_hint,
                        "resume_hint": response.resume_hint,
                        "blockers": response.blockers,
                    }
                }],
                "isError": false
            }),
            Err(err) => json!({
                "content": [{
                    "type": "json",
                    "json": map_tool_error(&err)
                }],
                "isError": true
            }),
        },
        HSM_RESUME_TOOL => {
            let request = match parse_hsm_resume_request(arguments) {
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

            match service.hsm_resume_tool(request) {
                Ok(response) => {
                    let blocked = !response.resumed;
                    let payload = if blocked {
                        json!({
                            "isError": true,
                            "error_type": "HsmResumeBlocked",
                            "message": "hsm resume blocked by checkpoint guard",
                            "resumed": response.resumed,
                            "resume_from": response.resume_from,
                            "resume_hint": response.resume_hint,
                            "blockers": response.blockers,
                        })
                    } else {
                        json!({
                            "resumed": response.resumed,
                            "resume_from": response.resume_from,
                            "resume_hint": response.resume_hint,
                            "blockers": response.blockers,
                        })
                    };
                    json!({
                        "content": [{
                            "type": "json",
                            "json": payload
                        }],
                        "isError": blocked
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
        _ => unknown_tool_result(tool_name),
    }
}

pub fn event_history_tool_result(service: &TurboLedgerService, arguments: &Value) -> Value {
    let filter = match parse_event_history_filter(arguments) {
        Ok(filter) => filter,
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

    match service.event_history(filter.clone()) {
        Ok(response) => {
            let events = response
                .events
                .into_iter()
                .map(|event| {
                    json!({
                        "event_id": event.event_id,
                        "sequence": event.sequence,
                        "event_type": event.event_type,
                        "tx_id": event.tx_id,
                        "document_ref": event.document_ref,
                        "occurred_at": event.occurred_at,
                        "payload": event.payload,
                        "identity_inputs": event.identity_inputs,
                    })
                })
                .collect::<Vec<_>>();

            json!({
                "content": [{
                    "type": "json",
                    "json": {
                        "filter": {
                            "tx_id": filter.tx_id,
                            "document_ref": filter.document_ref,
                            "time_start": filter.time_start,
                            "time_end": filter.time_end,
                        },
                        "events": events,
                    }
                }],
                "isError": false
            })
        }
        Err(ToolError::InvalidInput(message)) if message.contains("time_start must be <= time_end") => json!({
            "content": [{
                "type": "json",
                "json": {
                    "isError": true,
                    "error_type": "EventHistoryBlocked",
                    "reason": "time_range_invalid",
                    "message": message,
                }
            }],
            "isError": true
        }),
        Err(err) => json!({
            "content": [{
                "type": "json",
                "json": map_tool_error(&err)
            }],
            "isError": true
        }),
    }
}

pub fn event_replay_tool_result(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_replay_lifecycle_request(arguments) {
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

    match service.replay_lifecycle(request) {
        Ok(response) => json!({
            "content": [{
                "type": "json",
                "json": {
                    "reconstructed_state": response.reconstructed_state,
                    "event_count": response.event_count,
                    "diagnostics": response.diagnostics,
                    "filter": {
                        "tx_id": response.filter.tx_id,
                        "document_ref": response.filter.document_ref,
                        "time_start": response.filter.time_start,
                        "time_end": response.filter.time_end,
                    }
                }
            }],
            "isError": false
        }),
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

fn parse_ontology_path(arguments: &Value) -> Result<PathBuf, ToolError> {
    Ok(PathBuf::from(required_str(arguments, "ontology_path")?))
}

fn parse_ontology_query_path_request(arguments: &Value) -> Result<OntologyQueryPathRequest, ToolError> {
    let ontology_path = parse_ontology_path(arguments)?;
    let from_entity_id = required_str(arguments, "from_entity_id")?.to_string();
    let max_depth = match arguments.get("max_depth") {
        None | Some(Value::Null) => None,
        Some(Value::Number(num)) => {
            let raw = num.as_u64().ok_or_else(|| {
                ToolError::InvalidInput("`max_depth` must be a non-negative integer".to_string())
            })?;
            Some(usize::try_from(raw).map_err(|_| {
                ToolError::InvalidInput("`max_depth` is too large for this platform".to_string())
            })?)
        }
        _ => {
            return Err(ToolError::InvalidInput(
                "`max_depth` must be null or a non-negative integer".to_string(),
            ))
        }
    };

    Ok(OntologyQueryPathRequest {
        ontology_path,
        from_entity_id,
        max_depth,
    })
}

fn parse_reconciliation_stage_request(arguments: &Value) -> Result<ReconciliationStageRequest, ToolError> {
    let source_total = required_str(arguments, "source_total")?.to_string();
    let extracted_total = required_str(arguments, "extracted_total")?.to_string();
    let posting_amounts = parse_string_array(arguments.get("posting_amounts"), "posting_amounts")?;
    Ok(ReconciliationStageRequest {
        source_total,
        extracted_total,
        posting_amounts,
    })
}

fn parse_hsm_transition_request(arguments: &Value) -> Result<HsmTransitionRequest, ToolError> {
    let target_state = required_str(arguments, "target_state")?.to_string();
    let target_substate = required_str(arguments, "target_substate")?.to_string();
    Ok(HsmTransitionRequest {
        target_state,
        target_substate,
    })
}

fn parse_hsm_resume_request(arguments: &Value) -> Result<HsmResumeRequest, ToolError> {
    let state_marker = required_str(arguments, "state_marker")?.to_string();
    Ok(HsmResumeRequest { state_marker })
}

fn parse_event_history_filter(arguments: &Value) -> Result<EventHistoryFilter, ToolError> {
    Ok(EventHistoryFilter {
        tx_id: optional_str(arguments, "tx_id"),
        document_ref: optional_str(arguments, "document_ref"),
        time_start: optional_str(arguments, "time_start"),
        time_end: optional_str(arguments, "time_end"),
    })
}

fn parse_replay_lifecycle_request(arguments: &Value) -> Result<ReplayLifecycleRequest, ToolError> {
    Ok(ReplayLifecycleRequest {
        tx_id: optional_str(arguments, "tx_id"),
        document_ref: optional_str(arguments, "document_ref"),
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

fn parse_rows(value: Option<&Value>, field_name: &str) -> Result<Vec<TransactionInput>, ToolError> {
    let rows = value
        .and_then(Value::as_array)
        .ok_or_else(|| ToolError::InvalidInput(format!("missing or invalid `{field_name}`")))?;

    rows.iter()
        .map(|row| {
            Ok(TransactionInput {
                account_id: row
                    .get("account_id")
                    .and_then(Value::as_str)
                    .or_else(|| row.get("account").and_then(Value::as_str))
                    .ok_or_else(|| {
                        ToolError::InvalidInput(
                            "missing or invalid `account_id` in tool arguments".to_string(),
                        )
                    })?
                    .to_string(),
                date: required_str(row, "date")?.to_string(),
                amount: required_str(row, "amount")?.to_string(),
                description: required_str(row, "description")?.to_string(),
                source_ref: required_str(row, "source_ref")?.to_string(),
            })
        })
        .collect()
}

fn optional_str(obj: &Value, key: &str) -> Option<String> {
    obj.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn parse_string_array(value: Option<&Value>, field_name: &str) -> Result<Vec<String>, ToolError> {
    let items = value
        .and_then(Value::as_array)
        .ok_or_else(|| ToolError::InvalidInput(format!("missing or invalid `{field_name}`")))?;

    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| ToolError::InvalidInput(format!("`{field_name}` must contain strings")))
        })
        .collect()
}
