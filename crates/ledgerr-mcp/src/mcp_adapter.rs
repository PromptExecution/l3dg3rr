use std::path::PathBuf;

use ledger_core::ingest::{deterministic_tx_id, TransactionInput};
use serde::Serialize;
use serde_json::{json, Value};

use crate::{
    ClassifyIngestedRequest, ClassifyTransactionRequest, EventHistoryFilter,
    ExportCpaWorkbookRequest, FlagStatusRequest, GetRawContextRequest, GetScheduleSummaryRequest,
    HsmResumeRequest, HsmStatusRequest, HsmTransitionRequest, IngestPdfRequest,
    IngestStatementRowsRequest, ListAccountsRequest, OntologyExportSnapshotRequest,
    OntologyQueryPathRequest, OntologyUpsertEdgesRequest, OntologyUpsertEntitiesRequest,
    QueryAuditLogRequest, QueryFlagsRequest, ReconcileExcelClassificationRequest,
    ReconciliationStageRequest, ReplayLifecycleRequest, ScheduleKindRequest,
    TaxAmbiguityReviewRequest, TaxAssistRequest, TaxEvidenceChainRequest, ToolError,
    TurboLedgerService, TurboLedgerTools,
};

pub const LIST_ACCOUNTS_TOOL: &str = "l3dg3rr_list_accounts";
pub const GET_RAW_CONTEXT_TOOL: &str = "l3dg3rr_get_raw_context";
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
pub const TAX_ASSIST_TOOL: &str = "l3dg3rr_tax_assist";
pub const TAX_EVIDENCE_CHAIN_TOOL: &str = "l3dg3rr_tax_evidence_chain";
pub const TAX_AMBIGUITY_REVIEW_TOOL: &str = "l3dg3rr_tax_ambiguity_review";

// P0 tools
pub const CLASSIFY_INGESTED_TOOL: &str = "l3dg3rr_classify_ingested";
pub const QUERY_FLAGS_TOOL: &str = "l3dg3rr_query_flags";
pub const QUERY_AUDIT_LOG_TOOL: &str = "l3dg3rr_query_audit_log";

// P1 tools
pub const CLASSIFY_TRANSACTION_TOOL: &str = "l3dg3rr_classify_transaction";
pub const RECONCILE_EXCEL_CLASSIFICATION_TOOL: &str = "l3dg3rr_reconcile_excel_classification";
pub const GET_SCHEDULE_SUMMARY_TOOL: &str = "l3dg3rr_get_schedule_summary";

// P2 tools
pub const EXPORT_CPA_WORKBOOK_TOOL: &str = "l3dg3rr_export_cpa_workbook";
pub const ONTOLOGY_UPSERT_ENTITIES_TOOL: &str = "l3dg3rr_ontology_upsert_entities";
pub const ONTOLOGY_UPSERT_EDGES_TOOL: &str = "l3dg3rr_ontology_upsert_edges";

// Meta / self-management (always included regardless of feature flags)
pub use crate::plugin_info::PLUGIN_INFO_TOOL;

pub const TOOL_GROUP_CORE: &[&str] = &[
    LIST_ACCOUNTS_TOOL,
    GET_RAW_CONTEXT_TOOL,
    "proxy_docling_ingest_pdf",
    "proxy_rustledger_ingest_statement_rows",
    "l3dg3rr_get_pipeline_status",
];
pub const TOOL_GROUP_ONTOLOGY: &[&str] = &[ONTOLOGY_QUERY_PATH_TOOL, ONTOLOGY_EXPORT_SNAPSHOT_TOOL];
pub const TOOL_GROUP_RECONCILIATION: &[&str] =
    &[RECON_VALIDATE_TOOL, RECON_RECONCILE_TOOL, RECON_COMMIT_TOOL];
pub const TOOL_GROUP_HSM: &[&str] = &[HSM_TRANSITION_TOOL, HSM_STATUS_TOOL, HSM_RESUME_TOOL];
pub const TOOL_GROUP_EVENTS: &[&str] = &[EVENT_REPLAY_TOOL, EVENT_HISTORY_TOOL];
pub const TOOL_GROUP_CLASSIFICATION: &[&str] = &[
    CLASSIFY_INGESTED_TOOL,
    QUERY_FLAGS_TOOL,
    CLASSIFY_TRANSACTION_TOOL,
    RECONCILE_EXCEL_CLASSIFICATION_TOOL,
];
pub const TOOL_GROUP_TAX: &[&str] = &[
    TAX_ASSIST_TOOL,
    TAX_EVIDENCE_CHAIN_TOOL,
    TAX_AMBIGUITY_REVIEW_TOOL,
    GET_SCHEDULE_SUMMARY_TOOL,
    EXPORT_CPA_WORKBOOK_TOOL,
];
pub const TOOL_GROUP_AUDIT: &[&str] = &[QUERY_AUDIT_LOG_TOOL];
pub const TOOL_GROUP_ONTOLOGY_WRITE: &[&str] =
    &[ONTOLOGY_UPSERT_ENTITIES_TOOL, ONTOLOGY_UPSERT_EDGES_TOOL];

pub fn tool_names() -> Vec<String> {
    let mut features = Vec::new();

    #[cfg(feature = "core")]
    features.push("core");
    #[cfg(feature = "events")]
    features.push("events");
    #[cfg(feature = "reconciliation")]
    features.push("reconciliation");
    #[cfg(feature = "hsm")]
    features.push("hsm");
    #[cfg(feature = "ontology")]
    features.push("ontology");
    #[cfg(feature = "classification")]
    features.push("classification");
    #[cfg(feature = "audit")]
    features.push("audit");
    #[cfg(feature = "tax")]
    features.push("tax");

    if features.is_empty() {
        features.push("core");
    }

    tool_names_for(&features)
}

pub fn tool_names_for(features: &[&str]) -> Vec<String> {
    let mut tools = Vec::new();

    if features.iter().any(|f| *f == "core") {
        tools.extend(TOOL_GROUP_CORE.iter().map(|s| s.to_string()));
    }
    if features.iter().any(|f| *f == "ontology") || features.iter().any(|f| *f == "tax") {
        tools.extend(TOOL_GROUP_ONTOLOGY.iter().map(|s| s.to_string()));
    }
    if features.iter().any(|f| *f == "reconciliation") || features.iter().any(|f| *f == "tax") {
        tools.extend(TOOL_GROUP_RECONCILIATION.iter().map(|s| s.to_string()));
    }
    if features.iter().any(|f| *f == "hsm") || features.iter().any(|f| *f == "tax") {
        tools.extend(TOOL_GROUP_HSM.iter().map(|s| s.to_string()));
    }
    if features.iter().any(|f| *f == "events") || features.iter().any(|f| *f == "tax") {
        tools.extend(TOOL_GROUP_EVENTS.iter().map(|s| s.to_string()));
    }
    if features.iter().any(|f| *f == "classification") {
        tools.extend(TOOL_GROUP_CLASSIFICATION.iter().map(|s| s.to_string()));
    }
    if features.iter().any(|f| *f == "tax") {
        tools.extend(TOOL_GROUP_TAX.iter().map(|s| s.to_string()));
    }
    if features.iter().any(|f| *f == "audit") {
        tools.extend(TOOL_GROUP_AUDIT.iter().map(|s| s.to_string()));
    }
    if features.iter().any(|f| *f == "ontology") {
        tools.extend(TOOL_GROUP_ONTOLOGY_WRITE.iter().map(|s| s.to_string()));
    }

    // plugin_info is always available — not gated by any feature flag.
    tools.push(PLUGIN_INFO_TOOL.to_string());

    tools
}

/// Return the full MCP-spec tool objects (name + inputSchema) for all enabled tools.
/// Use this in tools/list responses — do NOT use tool_names() directly for that.
pub fn tool_descriptors() -> Vec<Value> {
    tool_names()
        .into_iter()
        .map(|name| {
            let schema = tool_input_schema(&name);
            json!({ "name": name, "inputSchema": schema })
        })
        .collect()
}

/// Returns the JSON Schema for the input arguments of a named tool.
pub fn tool_input_schema(name: &str) -> Value {
    match name {
        // ── no-argument tools ──────────────────────────────────────────────
        LIST_ACCOUNTS_TOOL
        | "l3dg3rr_get_pipeline_status"
        | HSM_STATUS_TOOL
        | QUERY_AUDIT_LOG_TOOL => json!({ "type": "object", "properties": {} }),

        // ── ontology_export_snapshot ───────────────────────────────────────
        ONTOLOGY_EXPORT_SNAPSHOT_TOOL => json!({
            "type": "object",
            "required": ["ontology_path"],
            "properties": {
                "ontology_path": { "type": "string", "description": "Path to the ontology file" }
            }
        }),

        // ── get_raw_context ───────────────────────────────────────────────
        GET_RAW_CONTEXT_TOOL => json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": { "type": "string", "description": "Path to the rkyv context file" }
            }
        }),

        // ── proxy_docling_ingest_pdf ───────────────────────────────────────
        "proxy_docling_ingest_pdf" => json!({
            "type": "object",
            "required": ["pdf_path", "journal_path", "workbook_path"],
            "properties": {
                "pdf_path": { "type": "string", "description": "Path to the PDF file (VENDOR--ACCOUNT--YYYY-MM--DOCTYPE naming required)" },
                "journal_path": { "type": "string", "description": "Path to the journal file" },
                "workbook_path": { "type": "string", "description": "Path to the Excel workbook" },
                "raw_context_bytes": {
                    "type": "array", "items": { "type": "integer", "minimum": 0, "maximum": 255 },
                    "description": "Raw PDF bytes as a byte array (required when source file does not yet exist on disk)"
                },
                "extracted_rows": {
                    "type": "array", "items": { "type": "object" },
                    "description": "Pre-extracted transaction rows from Docling (optional; omit or pass [] to ingest without rows)"
                }
            }
        }),

        // ── proxy_rustledger_ingest_statement_rows ────────────────────────
        "proxy_rustledger_ingest_statement_rows" => json!({
            "type": "object",
            "required": ["rows"],
            "properties": {
                "rows": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["account_id", "date", "amount", "description", "source_ref"],
                        "properties": {
                            "account_id": { "type": "string" },
                            "date": { "type": "string", "format": "date" },
                            "amount": { "type": "string", "description": "Decimal string, e.g. \"-42.11\"" },
                            "description": { "type": "string" },
                            "source_ref": { "type": "string" }
                        }
                    }
                }
            }
        }),

        // ── ontology_query_path ───────────────────────────────────────────
        ONTOLOGY_QUERY_PATH_TOOL => json!({
            "type": "object",
            "required": ["path"],
            "properties": {
                "path": { "type": "string", "description": "Filesystem path to the ontology file" },
                "max_depth": { "type": "integer", "minimum": 0, "description": "Maximum traversal depth (optional)" }
            }
        }),

        // ── reconciliation tools (validate / reconcile / commit) ──────────
        RECON_VALIDATE_TOOL | RECON_RECONCILE_TOOL | RECON_COMMIT_TOOL => json!({
            "type": "object",
            "required": ["source_total", "extracted_total", "posting_amounts"],
            "properties": {
                "source_total": { "type": "string", "description": "Total from source document (decimal string)" },
                "extracted_total": { "type": "string", "description": "Total of extracted rows (decimal string)" },
                "posting_amounts": {
                    "type": "array", "items": { "type": "string" },
                    "description": "Individual posting amounts as decimal strings"
                }
            }
        }),

        // ── hsm_transition ────────────────────────────────────────────────
        HSM_TRANSITION_TOOL => json!({
            "type": "object",
            "required": ["target_state", "target_substate"],
            "properties": {
                "target_state": { "type": "string" },
                "target_substate": { "type": "string" }
            }
        }),

        // ── hsm_resume ────────────────────────────────────────────────────
        HSM_RESUME_TOOL => json!({
            "type": "object",
            "required": ["state_marker"],
            "properties": {
                "state_marker": { "type": "string" }
            }
        }),

        // ── event_history ─────────────────────────────────────────────────
        EVENT_HISTORY_TOOL => json!({
            "type": "object",
            "properties": {
                "tx_id": { "type": "string" },
                "document_ref": { "type": "string" },
                "time_start": { "type": "string", "format": "date-time" },
                "time_end": { "type": "string", "format": "date-time" }
            }
        }),

        // ── event_replay ──────────────────────────────────────────────────
        EVENT_REPLAY_TOOL => json!({
            "type": "object",
            "properties": {
                "tx_id": { "type": "string" },
                "document_ref": { "type": "string" }
            }
        }),

        // ── classify_ingested ─────────────────────────────────────────────
        CLASSIFY_INGESTED_TOOL => json!({
            "type": "object",
            "required": ["rule_file", "review_threshold"],
            "properties": {
                "rule_file": { "type": "string", "description": "Path to the Rhai rule file" },
                "review_threshold": { "type": "number", "description": "Confidence threshold below which transactions are flagged for review" }
            }
        }),

        // ── query_flags ───────────────────────────────────────────────────
        QUERY_FLAGS_TOOL => json!({
            "type": "object",
            "required": ["year", "status"],
            "properties": {
                "year": { "type": "integer" },
                "status": { "type": "string", "enum": ["open", "resolved"] }
            }
        }),

        // ── classify_transaction / reconcile_excel_classification ─────────
        CLASSIFY_TRANSACTION_TOOL | RECONCILE_EXCEL_CLASSIFICATION_TOOL => json!({
            "type": "object",
            "required": ["tx_id", "category", "confidence", "actor"],
            "properties": {
                "tx_id": { "type": "string" },
                "category": { "type": "string" },
                "confidence": { "type": "string" },
                "note": { "type": "string" },
                "actor": { "type": "string" }
            }
        }),

        // ── tax_assist / tax_ambiguity_review ─────────────────────────────
        TAX_ASSIST_TOOL | TAX_AMBIGUITY_REVIEW_TOOL => json!({
            "type": "object",
            "required": ["ontology_path", "from_entity_id", "reconciliation"],
            "properties": {
                "ontology_path": { "type": "string" },
                "from_entity_id": { "type": "string" },
                "max_depth": { "type": "integer", "minimum": 0 },
                "reconciliation": {
                    "type": "object",
                    "required": ["source_total", "extracted_total", "posting_amounts"],
                    "properties": {
                        "source_total": { "type": "string" },
                        "extracted_total": { "type": "string" },
                        "posting_amounts": { "type": "array", "items": { "type": "string" } }
                    }
                }
            }
        }),

        // ── tax_evidence_chain ────────────────────────────────────────────
        TAX_EVIDENCE_CHAIN_TOOL => json!({
            "type": "object",
            "required": ["ontology_path", "from_entity_id"],
            "properties": {
                "ontology_path": { "type": "string" },
                "from_entity_id": { "type": "string" },
                "tx_id": { "type": "string" },
                "document_ref": { "type": "string" }
            }
        }),

        // ── get_schedule_summary ──────────────────────────────────────────
        GET_SCHEDULE_SUMMARY_TOOL => json!({
            "type": "object",
            "required": ["year", "schedule"],
            "properties": {
                "year": { "type": "integer" },
                "schedule": { "type": "string", "enum": ["ScheduleC", "ScheduleD", "ScheduleE", "Fbar"] }
            }
        }),

        // ── export_cpa_workbook ───────────────────────────────────────────
        EXPORT_CPA_WORKBOOK_TOOL => json!({
            "type": "object",
            "required": ["workbook_path"],
            "properties": {
                "workbook_path": { "type": "string", "description": "Output path for the Excel workbook" }
            }
        }),

        // ── ontology_upsert_entities ──────────────────────────────────────
        ONTOLOGY_UPSERT_ENTITIES_TOOL => json!({
            "type": "object",
            "required": ["path", "entities"],
            "properties": {
                "path": { "type": "string" },
                "entities": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["kind"],
                        "properties": {
                            "kind": { "type": "string", "enum": ["Document", "Account", "Institution", "Transaction", "TaxCategory", "EvidenceReference"] },
                            "id": { "type": "string" },
                            "label": { "type": "string" },
                            "properties": { "type": "object" }
                        }
                    }
                }
            }
        }),

        // ── ontology_upsert_edges ─────────────────────────────────────────
        ONTOLOGY_UPSERT_EDGES_TOOL => json!({
            "type": "object",
            "required": ["path", "edges"],
            "properties": {
                "path": { "type": "string" },
                "edges": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["from_id", "to_id", "relation"],
                        "properties": {
                            "from_id": { "type": "string" },
                            "to_id": { "type": "string" },
                            "relation": { "type": "string" },
                            "provenance": { "type": "object" }
                        }
                    }
                }
            }
        }),

        // ── plugin_info ───────────────────────────────────────────────────
        PLUGIN_INFO_TOOL => crate::plugin_info::input_schema(),

        // ── unknown / future tools ────────────────────────────────────────
        _ => json!({ "type": "object" }),
    }
}

pub fn handle_list_accounts(service: &TurboLedgerService) -> Value {
    match service.list_accounts_tool(ListAccountsRequest) {
        Ok(response) => {
            let accounts = response
                .accounts
                .into_iter()
                .map(|account| json!({ "account_id": account.account_id }))
                .collect::<Vec<_>>();
            json!({
                "content": [text_content(json!({ "accounts": accounts }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_get_raw_context(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_get_raw_context_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.get_raw_context(request) {
        Ok(response) => json!({
            "content": [text_content(json!({ "bytes": response.bytes }))],
            "isError": false
        }),
        Err(err) => error_envelope(&err),
    }
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

pub fn handle_pipeline_status(
    manifest_ready: bool,
    rustledger_ready: bool,
    docling_ready: bool,
    blockers: Vec<String>,
) -> Value {
    let status = get_pipeline_status(manifest_ready, rustledger_ready, docling_ready, blockers);
    json!({
        "content": [text_content(json!({
            "status": status.status,
            "blockers": status.blockers,
            "next_hint": status.next_hint,
        }))],
        "isError": false
    })
}

pub fn rows_to_json_with_provenance(
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

fn text_content(payload: Value) -> Value {
    json!({ "type": "text", "text": payload.to_string() })
}

fn error_envelope(err: &ToolError) -> Value {
    json!({
        "content": [text_content(error_payload(err))],
        "isError": true
    })
}

pub fn error_payload(error: &ToolError) -> Value {
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
        "content": [text_content(json!({
                "isError": true,
                "error_type": "InvalidInput",
                "message": format!("unknown tool: {tool_name}")
            }))],
        "isError": true
    })
}

/// Handle a `l3dg3rr_plugin_info` tool call.
/// Wraps `crate::plugin_info::handle` in the standard MCP content envelope.
pub fn handle_plugin_info(arguments: &Value) -> Value {
    let payload = crate::plugin_info::handle(arguments);
    json!({
        "content": [text_content(payload)],
        "isError": false
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

fn parse_ingest_pdf_request(arguments: &Value) -> Result<IngestPdfRequest, ToolError> {
    let pdf_path = required_str(arguments, "pdf_path")?.to_string();
    let journal_path = PathBuf::from(required_str(arguments, "journal_path")?);
    let workbook_path = PathBuf::from(required_str(arguments, "workbook_path")?);
    let raw_context_bytes = parse_optional_bytes(arguments.get("raw_context_bytes"))?;
    let extracted_rows = match arguments.get("extracted_rows") {
        None | Some(Value::Null) => Vec::new(),
        some => parse_rows(some, "extracted_rows")?,
    };

    Ok(IngestPdfRequest {
        pdf_path,
        journal_path,
        workbook_path,
        raw_context_bytes,
        extracted_rows,
    })
}

fn parse_ingest_statement_rows_request(
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

pub fn handle_ingest_pdf<T: TurboLedgerTools>(
    service: &T,
    arguments: &Value,
    backend_call_id: Option<String>,
) -> Value {
    let request = match parse_ingest_pdf_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    let canonical_rows = rows_to_json_with_provenance(
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
                "content": [text_content(json!({
                        "inserted_count": response.inserted_count,
                        "tx_ids": tx_ids,
                        "canonical_rows": canonical_rows,
                    }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_ingest_statement_rows<T: TurboLedgerTools>(
    service: &T,
    arguments: &Value,
    backend_call_id: Option<String>,
) -> Value {
    let request = match parse_ingest_statement_rows_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    let canonical_rows = rows_to_json_with_provenance(
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
                "content": [text_content(json!({
                        "inserted_count": response.inserted_count,
                        "tx_ids": tx_ids,
                        "canonical_rows": canonical_rows,
                        "provider": "rustledger",
                        "backend_tool": "ingest_statement_rows",
                    }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_ontology_query_path(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_ontology_query_path_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.ontology_query_path_tool(request) {
        Ok(response) => json!({
            "content": [text_content(json!({
                    "nodes": response.nodes,
                    "edges": response.edges,
                }))],
            "isError": false
        }),
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_ontology_export_snapshot(service: &TurboLedgerService, arguments: &Value) -> Value {
    let ontology_path = match parse_ontology_path(arguments) {
        Ok(path) => path,
        Err(err) => return error_envelope(&err),
    };

    match service.ontology_export_snapshot(OntologyExportSnapshotRequest { ontology_path }) {
        Ok(response) => json!({
            "content": [text_content(json!({
                    "entities": response.entities,
                    "edges": response.edges,
                    "snapshot": {
                        "entity_count": response.entity_count,
                        "edge_count": response.edge_count,
                    }
                }))],
            "isError": false
        }),
        Err(err) => error_envelope(&err),
    }
}

pub fn dispatch_reconciliation(
    service: &TurboLedgerService,
    tool_name: &str,
    arguments: &Value,
) -> Value {
    let request = match parse_reconciliation_stage_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
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
                "content": [text_content(payload)],
                "isError": blocked
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn dispatch_hsm(service: &TurboLedgerService, tool_name: &str, arguments: &Value) -> Value {
    match tool_name {
        HSM_TRANSITION_TOOL => {
            let request = match parse_hsm_transition_request(arguments) {
                Ok(request) => request,
                Err(err) => return error_envelope(&err),
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
                        "content": [text_content(payload)],
                        "isError": blocked
                    })
                }
                Err(err) => error_envelope(&err),
            }
        }
        HSM_STATUS_TOOL => match service.hsm_status_tool(HsmStatusRequest) {
            Ok(response) => json!({
                "content": [text_content(json!({
                        "state": response.state,
                        "substate": response.substate,
                        "display_state": response.display_state,
                        "next_hint": response.next_hint,
                        "resume_hint": response.resume_hint,
                        "blockers": response.blockers,
                    }))],
                "isError": false
            }),
            Err(err) => error_envelope(&err),
        },
        HSM_RESUME_TOOL => {
            let request = match parse_hsm_resume_request(arguments) {
                Ok(request) => request,
                Err(err) => return error_envelope(&err),
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
                        "content": [text_content(payload)],
                        "isError": blocked
                    })
                }
                Err(err) => error_envelope(&err),
            }
        }
        _ => unknown_tool_result(tool_name),
    }
}

pub fn handle_event_history(service: &TurboLedgerService, arguments: &Value) -> Value {
    let filter = match parse_event_history_filter(arguments) {
        Ok(filter) => filter,
        Err(err) => return error_envelope(&err),
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
                "content": [text_content(json!({
                        "filter": {
                            "tx_id": filter.tx_id,
                            "document_ref": filter.document_ref,
                            "time_start": filter.time_start,
                            "time_end": filter.time_end,
                        },
                        "events": events,
                    }))],
                "isError": false
            })
        }
        Err(ToolError::InvalidInput(message))
            if message.contains("time_start must be <= time_end") =>
        {
            json!({
                "content": [text_content(json!({
                        "isError": true,
                        "error_type": "EventHistoryBlocked",
                        "reason": "time_range_invalid",
                        "message": message,
                    }))],
                "isError": true
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_event_replay(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_replay_lifecycle_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.replay_lifecycle(request) {
        Ok(response) => json!({
            "content": [text_content(json!({
                    "reconstructed_state": response.reconstructed_state,
                    "event_count": response.event_count,
                    "diagnostics": response.diagnostics,
                    "filter": {
                        "tx_id": response.filter.tx_id,
                        "document_ref": response.filter.document_ref,
                        "time_start": response.filter.time_start,
                        "time_end": response.filter.time_end,
                    }
                }))],
            "isError": false
        }),
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_tax_assist(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_tax_assist_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.tax_assist_tool(request) {
        Ok(response) => {
            let blocked = response.status == "blocked";
            let payload = if blocked {
                json!({
                    "isError": true,
                    "error_type": "TaxAssistBlocked",
                    "reason": response.blocked_reasons.first().cloned().unwrap_or_default(),
                    "status": response.status,
                    "stage_marker": response.stage_marker,
                    "blocked_reasons": response.blocked_reasons,
                    "summary": response.summary,
                    "schedule_rows": response.schedule_rows,
                    "fbar_rows": response.fbar_rows,
                    "ambiguity": response.ambiguity,
                })
            } else {
                json!({
                    "status": response.status,
                    "stage_marker": response.stage_marker,
                    "blocked_reasons": response.blocked_reasons,
                    "summary": response.summary,
                    "schedule_rows": response.schedule_rows,
                    "fbar_rows": response.fbar_rows,
                    "ambiguity": response.ambiguity,
                })
            };
            json!({
                "content": [text_content(payload)],
                "isError": blocked
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_tax_evidence_chain(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_tax_evidence_chain_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.tax_evidence_chain_tool(request) {
        Ok(response) => json!({
            "content": [text_content(json!({
                    "source": response.source,
                    "events": response.events,
                    "current_state": response.current_state,
                    "ambiguity": response.ambiguity,
                }))],
            "isError": false
        }),
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_tax_ambiguity_review(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_tax_ambiguity_review_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.tax_ambiguity_review_tool(request) {
        Ok(response) => {
            let blocked = response.status == "blocked";
            let payload = if blocked {
                json!({
                    "isError": true,
                    "error_type": "TaxAmbiguityReviewBlocked",
                    "reason": response.blocked_reasons.first().cloned().unwrap_or_default(),
                    "status": response.status,
                    "stage_marker": response.stage_marker,
                    "blocked_reasons": response.blocked_reasons,
                    "ambiguity": response.ambiguity,
                })
            } else {
                json!({
                    "status": response.status,
                    "stage_marker": response.stage_marker,
                    "blocked_reasons": response.blocked_reasons,
                    "ambiguity": response.ambiguity,
                })
            };
            json!({
                "content": [text_content(payload)],
                "isError": blocked
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_classify_ingested(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_classify_ingested_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.classify_ingested(request) {
        Ok(response) => {
            let classifications = response
                .classifications
                .into_iter()
                .map(|c| {
                    json!({
                        "tx_id": c.tx_id,
                        "category": c.category,
                        "confidence": c.confidence,
                        "needs_review": c.needs_review,
                        "reason": c.reason,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "content": [text_content(json!({ "classifications": classifications }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_query_flags(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_query_flags_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.query_flags(request) {
        Ok(response) => {
            let flags = response
                .flags
                .into_iter()
                .map(|f| {
                    json!({
                        "tx_id": f.tx_id,
                        "year": f.year,
                        "status": match f.status {
                            FlagStatusRequest::Open => "open",
                            FlagStatusRequest::Resolved => "resolved",
                        },
                        "reason": f.reason,
                        "category": f.category,
                        "confidence": f.confidence,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "content": [text_content(json!({ "flags": flags }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_query_audit_log(service: &TurboLedgerService, arguments: &Value) -> Value {
    let _request = match parse_query_audit_log_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.query_audit_log(QueryAuditLogRequest) {
        Ok(response) => {
            let entries = response
                .entries
                .into_iter()
                .map(|e| {
                    json!({
                        "timestamp": e.timestamp,
                        "actor": e.actor,
                        "tx_id": e.tx_id,
                        "field": e.field,
                        "old_value": e.old_value,
                        "new_value": e.new_value,
                        "note": e.note,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "content": [text_content(json!({ "entries": entries }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_classify_transaction(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_classify_transaction_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.classify_transaction(request) {
        Ok(response) => {
            let audit_entries = response
                .audit_entries
                .into_iter()
                .map(|e| {
                    json!({
                        "timestamp": e.timestamp,
                        "actor": e.actor,
                        "tx_id": e.tx_id,
                        "field": e.field,
                        "old_value": e.old_value,
                        "new_value": e.new_value,
                        "note": e.note,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "content": [text_content(json!({
                        "tx_id": response.tx_id,
                        "category": response.category,
                        "confidence": response.confidence,
                        "audit_entries": audit_entries,
                    }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_reconcile_excel_classification(
    service: &TurboLedgerService,
    arguments: &Value,
) -> Value {
    let request = match parse_reconcile_excel_classification_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.reconcile_excel_classification(request) {
        Ok(response) => {
            let audit_entries = response
                .audit_entries
                .into_iter()
                .map(|e| {
                    json!({
                        "timestamp": e.timestamp,
                        "actor": e.actor,
                        "tx_id": e.tx_id,
                        "field": e.field,
                        "old_value": e.old_value,
                        "new_value": e.new_value,
                        "note": e.note,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "content": [text_content(json!({
                        "tx_id": response.tx_id,
                        "category": response.category,
                        "confidence": response.confidence,
                        "audit_entries": audit_entries,
                    }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_get_schedule_summary(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_get_schedule_summary_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.get_schedule_summary(request) {
        Ok(response) => {
            let schedule_str = match response.schedule {
                ScheduleKindRequest::ScheduleC => "ScheduleC",
                ScheduleKindRequest::ScheduleD => "ScheduleD",
                ScheduleKindRequest::ScheduleE => "ScheduleE",
                ScheduleKindRequest::Fbar => "Fbar",
            };
            let lines = response
                .lines
                .into_iter()
                .map(|l| {
                    json!({
                        "key": l.key,
                        "total": l.total,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "content": [text_content(json!({
                        "year": response.year,
                        "schedule": schedule_str,
                        "total": response.total,
                        "lines": lines,
                    }))],
                "isError": false
            })
        }
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_export_cpa_workbook(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_export_cpa_workbook_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.export_cpa_workbook(request) {
        Ok(response) => json!({
            "content": [text_content(json!({ "sheets_written": response.sheets_written }))],
            "isError": false
        }),
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_ontology_upsert_entities(
    service: &TurboLedgerService,
    arguments: &Value,
) -> Value {
    let request = match parse_ontology_upsert_entities_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.ontology_upsert_entities_tool(request) {
        Ok(response) => json!({
            "content": [text_content(json!({ "upserted": response.inserted_count }))],
            "isError": false
        }),
        Err(err) => error_envelope(&err),
    }
}

pub fn handle_ontology_upsert_edges(service: &TurboLedgerService, arguments: &Value) -> Value {
    let request = match parse_ontology_upsert_edges_request(arguments) {
        Ok(request) => request,
        Err(err) => return error_envelope(&err),
    };

    match service.ontology_upsert_edges_tool(request) {
        Ok(response) => json!({
            "content": [text_content(json!({ "upserted": response.inserted_count }))],
            "isError": false
        }),
        Err(err) => error_envelope(&err),
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

fn parse_get_raw_context_request(arguments: &Value) -> Result<GetRawContextRequest, ToolError> {
    Ok(GetRawContextRequest {
        rkyv_ref: PathBuf::from(required_str(arguments, "rkyv_ref")?),
    })
}

fn parse_ontology_query_path_request(
    arguments: &Value,
) -> Result<OntologyQueryPathRequest, ToolError> {
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

fn parse_reconciliation_stage_request(
    arguments: &Value,
) -> Result<ReconciliationStageRequest, ToolError> {
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

fn parse_tax_assist_request(arguments: &Value) -> Result<TaxAssistRequest, ToolError> {
    let ontology_path = PathBuf::from(required_str(arguments, "ontology_path")?);
    let from_entity_id = required_str(arguments, "from_entity_id")?.to_string();
    let max_depth = parse_optional_max_depth(arguments.get("max_depth"))?;
    let reconciliation = parse_nested_reconciliation_request(arguments)?;
    Ok(TaxAssistRequest {
        ontology_path,
        from_entity_id,
        max_depth,
        reconciliation,
    })
}

fn parse_tax_evidence_chain_request(
    arguments: &Value,
) -> Result<TaxEvidenceChainRequest, ToolError> {
    let ontology_path = PathBuf::from(required_str(arguments, "ontology_path")?);
    let from_entity_id = required_str(arguments, "from_entity_id")?.to_string();
    let tx_id = optional_str(arguments, "tx_id");
    let document_ref = optional_str(arguments, "document_ref");
    Ok(TaxEvidenceChainRequest {
        ontology_path,
        from_entity_id,
        tx_id,
        document_ref,
    })
}

fn parse_tax_ambiguity_review_request(
    arguments: &Value,
) -> Result<TaxAmbiguityReviewRequest, ToolError> {
    let ontology_path = PathBuf::from(required_str(arguments, "ontology_path")?);
    let from_entity_id = required_str(arguments, "from_entity_id")?.to_string();
    let max_depth = parse_optional_max_depth(arguments.get("max_depth"))?;
    let reconciliation = parse_nested_reconciliation_request(arguments)?;
    Ok(TaxAmbiguityReviewRequest {
        ontology_path,
        from_entity_id,
        max_depth,
        reconciliation,
    })
}

fn parse_nested_reconciliation_request(
    arguments: &Value,
) -> Result<ReconciliationStageRequest, ToolError> {
    let reconciliation = arguments.get("reconciliation").ok_or_else(|| {
        ToolError::InvalidInput("missing or invalid `reconciliation` in tool arguments".to_string())
    })?;
    parse_reconciliation_stage_request(reconciliation)
}

fn parse_optional_max_depth(value: Option<&Value>) -> Result<Option<usize>, ToolError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(num)) => {
            let raw = num.as_u64().ok_or_else(|| {
                ToolError::InvalidInput("`max_depth` must be a non-negative integer".to_string())
            })?;
            let depth = usize::try_from(raw).map_err(|_| {
                ToolError::InvalidInput("`max_depth` is too large for this platform".to_string())
            })?;
            Ok(Some(depth))
        }
        _ => Err(ToolError::InvalidInput(
            "missing or invalid `max_depth` in tool arguments".to_string(),
        )),
    }
}

fn parse_optional_bytes(value: Option<&Value>) -> Result<Option<Vec<u8>>, ToolError> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| {
                let num = item.as_u64().ok_or_else(|| {
                    ToolError::InvalidInput(
                        "raw_context_bytes must be an array of bytes".to_string(),
                    )
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
            item.as_str().map(ToString::to_string).ok_or_else(|| {
                ToolError::InvalidInput(format!("`{field_name}` must contain strings"))
            })
        })
        .collect()
}

fn parse_classify_ingested_request(
    arguments: &Value,
) -> Result<ClassifyIngestedRequest, ToolError> {
    let rule_file = PathBuf::from(required_str(arguments, "rule_file")?);
    let review_threshold = match arguments.get("review_threshold") {
        Some(Value::String(s)) => s.parse::<f64>().map_err(|_| {
            ToolError::InvalidInput("review_threshold must be a valid f64".to_string())
        })?,
        Some(Value::Number(n)) => n.as_f64().ok_or_else(|| {
            ToolError::InvalidInput("review_threshold must be a valid number".to_string())
        })?,
        _ => {
            return Err(ToolError::InvalidInput(
                "missing or invalid `review_threshold` in tool arguments".to_string(),
            ))
        }
    };
    Ok(ClassifyIngestedRequest {
        rule_file,
        review_threshold,
    })
}

fn parse_query_flags_request(arguments: &Value) -> Result<QueryFlagsRequest, ToolError> {
    let year = match arguments.get("year") {
        Some(Value::Number(n)) => n
            .as_i64()
            .ok_or_else(|| ToolError::InvalidInput("year must be a valid integer".to_string()))?
            as i32,
        _ => {
            return Err(ToolError::InvalidInput(
                "missing or invalid `year` in tool arguments".to_string(),
            ))
        }
    };
    let status = match arguments.get("status").and_then(Value::as_str) {
        Some("open") => FlagStatusRequest::Open,
        Some("resolved") => FlagStatusRequest::Resolved,
        _ => {
            return Err(ToolError::InvalidInput(
                "missing or invalid `status` in tool arguments (must be 'open' or 'resolved')"
                    .to_string(),
            ))
        }
    };
    Ok(QueryFlagsRequest { year, status })
}

fn parse_query_audit_log_request(_arguments: &Value) -> Result<QueryAuditLogRequest, ToolError> {
    Ok(QueryAuditLogRequest)
}

fn parse_classify_transaction_request(
    arguments: &Value,
) -> Result<ClassifyTransactionRequest, ToolError> {
    let tx_id = required_str(arguments, "tx_id")?.to_string();
    let category = required_str(arguments, "category")?.to_string();
    let confidence = required_str(arguments, "confidence")?.to_string();
    let note = optional_str(arguments, "note");
    let actor = required_str(arguments, "actor")?.to_string();
    Ok(ClassifyTransactionRequest {
        tx_id,
        category,
        confidence,
        note,
        actor,
    })
}

fn parse_reconcile_excel_classification_request(
    arguments: &Value,
) -> Result<ReconcileExcelClassificationRequest, ToolError> {
    let tx_id = required_str(arguments, "tx_id")?.to_string();
    let category = required_str(arguments, "category")?.to_string();
    let confidence = required_str(arguments, "confidence")?.to_string();
    let note = optional_str(arguments, "note");
    let actor = required_str(arguments, "actor")?.to_string();
    Ok(ReconcileExcelClassificationRequest {
        tx_id,
        category,
        confidence,
        note,
        actor,
    })
}

fn parse_get_schedule_summary_request(
    arguments: &Value,
) -> Result<GetScheduleSummaryRequest, ToolError> {
    let year = match arguments.get("year") {
        Some(Value::Number(n)) => n
            .as_i64()
            .ok_or_else(|| ToolError::InvalidInput("year must be a valid integer".to_string()))?
            as i32,
        _ => {
            return Err(ToolError::InvalidInput(
                "missing or invalid `year` in tool arguments".to_string(),
            ))
        }
    };
    let schedule = match arguments.get("schedule").and_then(Value::as_str) {
        Some("ScheduleC") => ScheduleKindRequest::ScheduleC,
        Some("ScheduleD") => ScheduleKindRequest::ScheduleD,
        Some("ScheduleE") => ScheduleKindRequest::ScheduleE,
        Some("Fbar") => ScheduleKindRequest::Fbar,
        _ => {
            return Err(ToolError::InvalidInput(
                "missing or invalid `schedule` in tool arguments (must be 'ScheduleC', 'ScheduleD', 'ScheduleE', or 'Fbar')".to_string(),
            ))
        }
    };
    Ok(GetScheduleSummaryRequest { year, schedule })
}

fn parse_export_cpa_workbook_request(
    arguments: &Value,
) -> Result<ExportCpaWorkbookRequest, ToolError> {
    let workbook_path = PathBuf::from(required_str(arguments, "workbook_path")?);
    Ok(ExportCpaWorkbookRequest { workbook_path })
}

fn parse_ontology_upsert_entities_request(
    arguments: &Value,
) -> Result<OntologyUpsertEntitiesRequest, ToolError> {
    let ontology_path = parse_ontology_path(arguments)?;
    let entities_json = arguments
        .get("entities")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ToolError::InvalidInput("missing or invalid `entities` in tool arguments".to_string())
        })?;
    let entities = entities_json
        .iter()
        .map(|e| {
            let kind = match e.get("kind").and_then(Value::as_str) {
                Some("Document") => crate::OntologyEntityKind::Document,
                Some("Account") => crate::OntologyEntityKind::Account,
                Some("Institution") => crate::OntologyEntityKind::Institution,
                Some("Transaction") => crate::OntologyEntityKind::Transaction,
                Some("TaxCategory") => crate::OntologyEntityKind::TaxCategory,
                Some("EvidenceReference") => crate::OntologyEntityKind::EvidenceReference,
                _ => {
                    return Err(ToolError::InvalidInput(
                        "missing or invalid `kind` in entity (must be Document, Account, Institution, Transaction, TaxCategory, or EvidenceReference)".to_string(),
                    ))
                }
            };
            let mut attrs = std::collections::BTreeMap::new();
            if let Some(id) = e.get("id").and_then(Value::as_str) {
                attrs.insert("id".to_string(), id.to_string());
            }
            if let Some(label) = e.get("label").and_then(Value::as_str) {
                attrs.insert("label".to_string(), label.to_string());
            }
            if let Some(obj) = e.get("properties").and_then(Value::as_object) {
                for (k, v) in obj {
                    attrs.insert(k.clone(), v.to_string());
                }
            }
            Ok::<crate::OntologyEntityInput, ToolError>(crate::OntologyEntityInput { kind, attrs })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OntologyUpsertEntitiesRequest {
        ontology_path,
        entities,
    })
}

fn parse_ontology_upsert_edges_request(
    arguments: &Value,
) -> Result<OntologyUpsertEdgesRequest, ToolError> {
    let ontology_path = parse_ontology_path(arguments)?;
    let edges_json = arguments
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ToolError::InvalidInput("missing or invalid `edges` in tool arguments".to_string())
        })?;
    let edges = edges_json
        .iter()
        .map(|e| {
            let from = required_str(e, "from_id").map(|s| s.to_string())?;
            let to = required_str(e, "to_id").map(|s| s.to_string())?;
            let relation = required_str(e, "relation").map(|s| s.to_string())?;
            let provenance = e
                .get("provenance")
                .and_then(Value::as_object)
                .map(|obj| {
                    obj.iter()
                        .map(|(k, v)| (k.clone(), v.to_string()))
                        .collect::<std::collections::BTreeMap<_, _>>()
                })
                .unwrap_or_default();
            Ok::<crate::OntologyEdgeInput, ToolError>(crate::OntologyEdgeInput {
                from,
                to,
                relation,
                provenance,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(OntologyUpsertEdgesRequest {
        ontology_path,
        edges,
    })
}
