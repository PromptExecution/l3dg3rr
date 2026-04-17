use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;

use ledger_core::classify::{ClassificationEngine, FlagStatus, SampleTransaction};
use ledger_core::filename::{FilenameError, StatementFilename};
use ledger_core::ingest::{deterministic_tx_id, IngestedLedger, TransactionInput};
use ledger_core::manifest::Manifest;
use ledger_core::workbook::REQUIRED_SHEETS;
use rust_decimal::Decimal;
use rust_xlsxwriter::Workbook;
use serde::{Deserialize, Serialize};

pub mod contract;
pub mod events;
pub mod hsm;
pub mod mcp_adapter;
pub mod ontology;
pub mod plugin_info;
pub mod reconciliation;
pub mod tax_assist;
pub use events::{
    AppendEventResult, EventHistoryFilter, EventHistoryResponse, InMemoryLifecycleEventStore,
    LifecycleEvent, LifecycleEventStore, ReplayProjection,
};
pub use hsm::{
    HsmMachine, HsmResumeRequest, HsmResumeResponse, HsmStatusRequest, HsmStatusResponse,
    HsmTransitionRequest, HsmTransitionResponse,
};
pub use ontology::{
    OntologyEdge, OntologyEdgeInput, OntologyEntity, OntologyEntityInput, OntologyEntityKind,
    OntologyQueryPathRequest, OntologyQueryPathResponse, OntologyStore, OntologyUpsertEdgesRequest,
    OntologyUpsertEdgesResponse, OntologyUpsertEntitiesRequest, OntologyUpsertEntitiesResponse,
};
pub use reconciliation::{
    commit_stage, reconcile_stage, validate_stage, ReconciliationDiagnostic,
    ReconciliationStageRequest, ReconciliationStageResponse,
};
pub use tax_assist::{
    TaxAmbiguityRecord, TaxAmbiguityReviewRequest, TaxAmbiguityReviewResponse, TaxAssistRequest,
    TaxAssistResponse, TaxAssistSummary, TaxEvidenceChainRequest, TaxEvidenceChainResponse,
    TaxEvidenceCurrentState, TaxEvidenceEvent, TaxEvidenceRow, TaxEvidenceSource,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountSummary {
    pub account_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListAccountsRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestStatementRowsRequest {
    pub journal_path: PathBuf,
    pub workbook_path: PathBuf,
    pub rows: Vec<TransactionInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestStatementRowsResponse {
    pub inserted_count: usize,
    pub tx_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestPdfRequest {
    pub pdf_path: String,
    pub journal_path: PathBuf,
    pub workbook_path: PathBuf,
    pub raw_context_bytes: Option<Vec<u8>>,
    pub extracted_rows: Vec<TransactionInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestPdfResponse {
    pub inserted_count: usize,
    pub tx_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetRawContextRequest {
    pub rkyv_ref: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetRawContextResponse {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleTxRequest {
    pub tx_id: String,
    pub account_id: String,
    pub date: String,
    pub amount: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRhaiRuleRequest {
    pub rule_file: PathBuf,
    pub sample_tx: SampleTxRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunRhaiRuleResponse {
    pub category: String,
    pub confidence: f64,
    pub review: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassifyIngestedRequest {
    pub rule_file: PathBuf,
    pub review_threshold: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassifiedTxResponse {
    pub tx_id: String,
    pub category: String,
    pub confidence: f64,
    pub needs_review: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassifyIngestedResponse {
    pub classifications: Vec<ClassifiedTxResponse>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagStatusRequest {
    Open,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryFlagsRequest {
    pub year: i32,
    pub status: FlagStatusRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlagRecordResponse {
    pub tx_id: String,
    pub year: i32,
    pub status: FlagStatusRequest,
    pub reason: String,
    pub category: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryFlagsResponse {
    pub flags: Vec<FlagRecordResponse>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifyTransactionRequest {
    pub tx_id: String,
    pub category: String,
    pub confidence: String,
    pub note: Option<String>,
    pub actor: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileExcelClassificationRequest {
    pub tx_id: String,
    pub category: String,
    pub confidence: String,
    pub actor: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryAuditLogRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntryResponse {
    pub timestamp: String,
    pub actor: String,
    pub tx_id: String,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifyTransactionResponse {
    pub tx_id: String,
    pub category: String,
    pub confidence: String,
    pub audit_entries: Vec<AuditEntryResponse>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayLifecycleRequest {
    pub tx_id: Option<String>,
    pub document_ref: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplayLifecycleResponse {
    pub reconstructed_state: String,
    pub event_count: usize,
    pub diagnostics: Vec<String>,
    pub filter: EventHistoryFilter,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryAuditLogResponse {
    pub entries: Vec<AuditEntryResponse>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportCpaWorkbookRequest {
    pub workbook_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportCpaWorkbookResponse {
    pub sheets_written: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyExportSnapshotRequest {
    pub ontology_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct OntologyExportSnapshotResponse {
    pub entities: Vec<OntologyEntity>,
    pub edges: Vec<OntologyEdge>,
    pub entity_count: usize,
    pub edge_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleKindRequest {
    ScheduleC,
    ScheduleD,
    ScheduleE,
    Fbar,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetScheduleSummaryRequest {
    pub year: i32,
    pub schedule: ScheduleKindRequest,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduleLineResponse {
    pub key: String,
    pub total: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GetScheduleSummaryResponse {
    pub year: i32,
    pub schedule: ScheduleKindRequest,
    pub total: f64,
    pub lines: Vec<ScheduleLineResponse>,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<FilenameError> for ToolError {
    fn from(value: FilenameError) -> Self {
        Self::InvalidInput(value.to_string())
    }
}

pub trait TurboLedgerTools {
    fn list_accounts(&self) -> Result<Vec<AccountSummary>, ToolError>;
    fn validate_source_filename(&self, file_name: &str) -> Result<StatementFilename, ToolError>;
    fn ingest_statement_rows(
        &self,
        request: IngestStatementRowsRequest,
    ) -> Result<IngestStatementRowsResponse, ToolError>;
    fn ingest_pdf(&self, request: IngestPdfRequest) -> Result<IngestPdfResponse, ToolError>;
    fn get_raw_context(
        &self,
        request: GetRawContextRequest,
    ) -> Result<GetRawContextResponse, ToolError>;
    fn run_rhai_rule(&self, request: RunRhaiRuleRequest) -> Result<RunRhaiRuleResponse, ToolError>;
    fn classify_ingested(
        &self,
        request: ClassifyIngestedRequest,
    ) -> Result<ClassifyIngestedResponse, ToolError>;
    fn query_flags(&self, request: QueryFlagsRequest) -> Result<QueryFlagsResponse, ToolError>;
    fn classify_transaction(
        &self,
        request: ClassifyTransactionRequest,
    ) -> Result<ClassifyTransactionResponse, ToolError>;
    fn reconcile_excel_classification(
        &self,
        request: ReconcileExcelClassificationRequest,
    ) -> Result<ClassifyTransactionResponse, ToolError>;
    fn query_audit_log(
        &self,
        request: QueryAuditLogRequest,
    ) -> Result<QueryAuditLogResponse, ToolError>;
    fn export_cpa_workbook(
        &self,
        request: ExportCpaWorkbookRequest,
    ) -> Result<ExportCpaWorkbookResponse, ToolError>;
    fn get_schedule_summary(
        &self,
        request: GetScheduleSummaryRequest,
    ) -> Result<GetScheduleSummaryResponse, ToolError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredClassification {
    category: String,
    confidence: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuditEntry {
    timestamp: String,
    actor: String,
    tx_id: String,
    field: String,
    old_value: Option<String>,
    new_value: String,
    note: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct ClassificationState {
    tx_rows: BTreeMap<String, TransactionInput>,
    classifications: BTreeMap<String, StoredClassification>,
    audit_log: Vec<AuditEntry>,
    engine: ClassificationEngine,
}

const PERSISTED_STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedServiceState {
    version: u32,
    ingest_state: IngestedLedger,
    classification_state: ClassificationState,
    lifecycle_events: InMemoryLifecycleEventStore,
    hsm_state: HsmMachine,
}

impl Default for PersistedServiceState {
    fn default() -> Self {
        Self {
            version: PERSISTED_STATE_VERSION,
            ingest_state: IngestedLedger::default(),
            classification_state: ClassificationState::default(),
            lifecycle_events: InMemoryLifecycleEventStore::default(),
            hsm_state: HsmMachine::default(),
        }
    }
}

pub struct TurboLedgerService {
    manifest: Manifest,
    ingest_state: Mutex<IngestedLedger>,
    classification_state: Mutex<ClassificationState>,
    lifecycle_events: Mutex<InMemoryLifecycleEventStore>,
    hsm_state: Mutex<HsmMachine>,
}

impl TurboLedgerService {
    pub fn from_manifest_str(src: &str) -> Result<Self, ToolError> {
        let manifest = Manifest::parse(src).map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        let persisted =
            load_persisted_state(std::path::Path::new(&manifest.session.workbook_path))?;
        Ok(Self {
            manifest,
            ingest_state: Mutex::new(persisted.ingest_state),
            classification_state: Mutex::new(persisted.classification_state),
            lifecycle_events: Mutex::new(persisted.lifecycle_events),
            hsm_state: Mutex::new(persisted.hsm_state),
        })
    }

    pub fn workbook_path(&self) -> &std::path::Path {
        std::path::Path::new(&self.manifest.session.workbook_path)
    }

    fn state_sidecar_path(&self) -> PathBuf {
        persisted_state_path(self.workbook_path())
    }

    fn snapshot_persisted_state(&self) -> Result<PersistedServiceState, ToolError> {
        // Persist all restart-visible state as one snapshot so idempotency, audit,
        // lifecycle replay, and HSM resume move together. Partial silent resets would
        // be worse than a hard failure in this accountant-facing workflow.
        let ingest_state = self
            .ingest_state
            .lock()
            .map_err(|_| ToolError::Internal("ingest lock poisoned".to_string()))?
            .clone();
        let classification_state = self
            .classification_state
            .lock()
            .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?
            .clone();
        let lifecycle_events = self
            .lifecycle_events
            .lock()
            .map_err(|_| ToolError::Internal("events lock poisoned".to_string()))?
            .clone();
        let hsm_state = self
            .hsm_state
            .lock()
            .map_err(|_| ToolError::Internal("hsm lock poisoned".to_string()))?
            .clone();
        Ok(PersistedServiceState {
            version: PERSISTED_STATE_VERSION,
            ingest_state,
            classification_state,
            lifecycle_events,
            hsm_state,
        })
    }

    fn persist_state(&self) -> Result<(), ToolError> {
        let snapshot = self.snapshot_persisted_state()?;
        persist_state_to_path(&self.state_sidecar_path(), &snapshot)
    }

    pub fn list_accounts_tool(
        &self,
        _request: ListAccountsRequest,
    ) -> Result<ListAccountsResponse, ToolError> {
        Ok(ListAccountsResponse {
            accounts: self.list_accounts()?,
        })
    }

    pub fn ontology_upsert_entities(
        &self,
        request: OntologyUpsertEntitiesRequest,
    ) -> Result<OntologyUpsertEntitiesResponse, ToolError> {
        let mut store = OntologyStore::load(&request.ontology_path)?;
        let response = store.upsert_entities(request.entities)?;
        store.persist(&request.ontology_path)?;
        Ok(response)
    }

    pub fn ontology_upsert_entities_tool(
        &self,
        request: OntologyUpsertEntitiesRequest,
    ) -> Result<OntologyUpsertEntitiesResponse, ToolError> {
        self.ontology_upsert_entities(request)
    }

    pub fn ontology_upsert_edges(
        &self,
        request: OntologyUpsertEdgesRequest,
    ) -> Result<OntologyUpsertEdgesResponse, ToolError> {
        let mut store = OntologyStore::load(&request.ontology_path)?;
        let response = store.upsert_edges(request.edges)?;
        store.persist(&request.ontology_path)?;
        Ok(response)
    }

    pub fn ontology_upsert_edges_tool(
        &self,
        request: OntologyUpsertEdgesRequest,
    ) -> Result<OntologyUpsertEdgesResponse, ToolError> {
        self.ontology_upsert_edges(request)
    }

    pub fn ontology_query_path(
        &self,
        request: OntologyQueryPathRequest,
    ) -> Result<OntologyQueryPathResponse, ToolError> {
        let store = OntologyStore::load(&request.ontology_path)?;
        store.query_path(&request.from_entity_id, request.max_depth)
    }

    pub fn ontology_query_path_tool(
        &self,
        request: OntologyQueryPathRequest,
    ) -> Result<OntologyQueryPathResponse, ToolError> {
        self.ontology_query_path(request)
    }

    pub fn ontology_export_snapshot(
        &self,
        request: OntologyExportSnapshotRequest,
    ) -> Result<OntologyExportSnapshotResponse, ToolError> {
        let store = OntologyStore::load(&request.ontology_path)?;
        Ok(OntologyExportSnapshotResponse {
            entity_count: store.entities.len(),
            edge_count: store.edges.len(),
            entities: store.entities,
            edges: store.edges,
        })
    }

    pub fn validate_reconciliation_stage_tool(
        &self,
        request: ReconciliationStageRequest,
    ) -> Result<ReconciliationStageResponse, ToolError> {
        validate_stage(&request)
    }

    pub fn reconcile_reconciliation_stage_tool(
        &self,
        request: ReconciliationStageRequest,
    ) -> Result<ReconciliationStageResponse, ToolError> {
        reconcile_stage(&request)
    }

    pub fn commit_reconciliation_stage_tool(
        &self,
        request: ReconciliationStageRequest,
    ) -> Result<ReconciliationStageResponse, ToolError> {
        commit_stage(&request)
    }

    pub fn hsm_transition_tool(
        &self,
        request: HsmTransitionRequest,
    ) -> Result<HsmTransitionResponse, ToolError> {
        let requested = hsm::parse_node(&request.target_state, &request.target_substate)
            .ok_or_else(|| {
                ToolError::InvalidInput(
                    "target_state/target_substate must match lifecycle vocabulary".to_string(),
                )
            })?;

        let mut hsm = self
            .hsm_state
            .lock()
            .map_err(|_| ToolError::Internal("hsm lock poisoned".to_string()))?;
        let current = hsm.current;
        if hsm::allowed_next_node(current) == Some(requested) {
            hsm.current = requested;
            hsm.last_valid_checkpoint = hsm::checkpoint_marker(requested);
            let response = hsm::transition_advanced_response(requested);
            drop(hsm);
            self.persist_state()?;
            return Ok(response);
        }

        Ok(hsm::transition_blocked_response(current, requested))
    }

    pub fn hsm_status_tool(
        &self,
        _request: HsmStatusRequest,
    ) -> Result<HsmStatusResponse, ToolError> {
        let hsm = self
            .hsm_state
            .lock()
            .map_err(|_| ToolError::Internal("hsm lock poisoned".to_string()))?;
        Ok(hsm::status_response(hsm.current, Vec::new()))
    }

    pub fn hsm_resume_tool(
        &self,
        request: HsmResumeRequest,
    ) -> Result<HsmResumeResponse, ToolError> {
        let mut hsm = self
            .hsm_state
            .lock()
            .map_err(|_| ToolError::Internal("hsm lock poisoned".to_string()))?;
        let Some(resume_node) = hsm::parse_checkpoint_marker(&request.state_marker) else {
            return Ok(hsm::resume_response(
                hsm.current,
                false,
                vec!["checkpoint_invalid".to_string()],
            ));
        };

        if request.state_marker != hsm.last_valid_checkpoint {
            return Ok(hsm::resume_response(
                hsm.current,
                false,
                vec!["checkpoint_unknown".to_string()],
            ));
        }

        hsm.current = resume_node;
        let response = hsm::resume_response(hsm.current, true, Vec::new());
        drop(hsm);
        self.persist_state()?;
        Ok(response)
    }

    pub fn adjust_transaction(
        &self,
        request: ClassifyTransactionRequest,
    ) -> Result<ClassifyTransactionResponse, ToolError> {
        self.apply_classification_action(request, "adjustment")
    }

    pub fn event_history(
        &self,
        filter: EventHistoryFilter,
    ) -> Result<EventHistoryResponse, ToolError> {
        self.lifecycle_events
            .lock()
            .map_err(|_| ToolError::Internal("events lock poisoned".to_string()))?
            .list_events(filter)
    }

    pub fn replay_lifecycle(
        &self,
        request: ReplayLifecycleRequest,
    ) -> Result<ReplayLifecycleResponse, ToolError> {
        let filter = EventHistoryFilter {
            tx_id: request.tx_id,
            document_ref: request.document_ref,
            time_start: None,
            time_end: None,
        };
        let history = self.event_history(filter.clone())?;
        let projection = events::reconstruct_lifecycle(&history.events);
        Ok(ReplayLifecycleResponse {
            reconstructed_state: projection.reconstructed_state,
            event_count: projection.event_count,
            diagnostics: projection.diagnostics,
            filter,
        })
    }

    pub fn tax_assist_tool(
        &self,
        request: TaxAssistRequest,
    ) -> Result<TaxAssistResponse, ToolError> {
        let stage = self.reconcile_reconciliation_stage_tool(request.reconciliation)?;
        let path = if stage.status == "passed" {
            let ontology_path = request.ontology_path.clone();
            let mut path = self.ontology_query_path_tool(OntologyQueryPathRequest {
                ontology_path: ontology_path.clone(),
                from_entity_id: request.from_entity_id.clone(),
                max_depth: request.max_depth,
            })?;
            let store = OntologyStore::load(&ontology_path)?;
            let entity_lookup = store
                .entities
                .iter()
                .map(|node| (node.id.clone(), node.clone()))
                .collect::<BTreeMap<_, _>>();
            let mut existing_edge_ids = path
                .edges
                .iter()
                .map(|edge| edge.id.clone())
                .collect::<BTreeSet<_>>();
            let mut existing_node_ids = path
                .nodes
                .iter()
                .map(|node| node.id.clone())
                .collect::<BTreeSet<_>>();
            for edge in store
                .edges
                .into_iter()
                .filter(|edge| edge.from == request.from_entity_id)
            {
                if existing_edge_ids.insert(edge.id.clone()) {
                    if !existing_node_ids.contains(&edge.to) {
                        if let Some(node) = entity_lookup.get(&edge.to) {
                            path.nodes.push(node.clone());
                            existing_node_ids.insert(node.id.clone());
                        }
                    }
                    path.edges.push(edge);
                }
            }
            Some(path)
        } else {
            None
        };
        Ok(tax_assist::build_tax_assist_response(
            &request.from_entity_id,
            stage,
            path,
        ))
    }

    pub fn tax_evidence_chain_tool(
        &self,
        request: TaxEvidenceChainRequest,
    ) -> Result<TaxEvidenceChainResponse, ToolError> {
        let normalized_tx_id = request
            .tx_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let normalized_document_ref = request
            .document_ref
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());

        let path = self.ontology_query_path_tool(OntologyQueryPathRequest {
            ontology_path: request.ontology_path,
            from_entity_id: request.from_entity_id.clone(),
            max_depth: None,
        })?;
        let history_filter = EventHistoryFilter {
            tx_id: normalized_tx_id.clone(),
            document_ref: normalized_document_ref.clone(),
            time_start: None,
            time_end: None,
        };
        let events = self.event_history(history_filter.clone())?;
        let replay = self.replay_lifecycle(ReplayLifecycleRequest {
            tx_id: history_filter.tx_id,
            document_ref: history_filter.document_ref,
        })?;

        let mut ambiguity = path
            .edges
            .iter()
            .filter(|edge| edge.relation == "ambiguity")
            .map(|edge| TaxAmbiguityRecord {
                tx_id: normalized_tx_id.clone().or_else(|| Some(edge.from.clone())),
                review_state: "needs_review".to_string(),
                reason: "ambiguous_tax_treatment".to_string(),
                provenance_refs: edge
                    .provenance
                    .iter()
                    .filter_map(|(key, value)| {
                        if key.contains("source") || key.contains("ref") {
                            Some(value.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();
        ambiguity.sort_by(|a, b| {
            a.tx_id
                .cmp(&b.tx_id)
                .then_with(|| a.review_state.cmp(&b.review_state))
                .then_with(|| a.reason.cmp(&b.reason))
        });
        let mut provenance_refs = path
            .edges
            .iter()
            .flat_map(|edge| edge.provenance.iter())
            .filter_map(|(key, value)| {
                if key.contains("source") || key.contains("ref") {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        provenance_refs.sort();
        provenance_refs.dedup();
        let mut node_ids = path
            .nodes
            .into_iter()
            .map(|node| node.id)
            .collect::<Vec<_>>();
        node_ids.sort();
        let mut edge_ids = path
            .edges
            .into_iter()
            .map(|edge| edge.id)
            .collect::<Vec<_>>();
        edge_ids.sort();
        let source = TaxEvidenceSource {
            from_entity_id: request.from_entity_id,
            node_ids,
            edge_ids,
            provenance_refs,
        };
        Ok(tax_assist::build_tax_evidence_chain_response(
            source, events, replay, ambiguity,
        ))
    }

    pub fn tax_ambiguity_review_tool(
        &self,
        request: TaxAmbiguityReviewRequest,
    ) -> Result<TaxAmbiguityReviewResponse, ToolError> {
        let stage = self.reconcile_reconciliation_stage_tool(request.reconciliation)?;
        let path = if stage.status == "passed" {
            self.ontology_query_path_tool(OntologyQueryPathRequest {
                ontology_path: request.ontology_path,
                from_entity_id: request.from_entity_id,
                max_depth: request.max_depth,
            })?
        } else {
            OntologyQueryPathResponse {
                nodes: Vec::new(),
                edges: Vec::new(),
            }
        };
        let assist = tax_assist::build_tax_assist_response("", stage.clone(), Some(path));
        Ok(tax_assist::build_tax_ambiguity_review_response(
            stage,
            assist.ambiguity,
        ))
    }

    fn append_lifecycle_event(
        &self,
        event_type: &str,
        tx_id: Option<String>,
        document_ref: Option<String>,
        payload: BTreeMap<String, String>,
    ) -> Result<AppendEventResult, ToolError> {
        self.lifecycle_events
            .lock()
            .map_err(|_| ToolError::Internal("events lock poisoned".to_string()))?
            .append_event(event_type, tx_id, document_ref, payload)
    }

    fn apply_classification_action(
        &self,
        request: ClassifyTransactionRequest,
        event_type: &str,
    ) -> Result<ClassifyTransactionResponse, ToolError> {
        let (response, tx_row) = {
            let mut classification = self
                .classification_state
                .lock()
                .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?;

            let tx_row = classification
                .tx_rows
                .get(&request.tx_id)
                .cloned()
                .ok_or_else(|| ToolError::InvalidInput("unknown tx_id".to_string()))?;

            validate_invariants(&tx_row, &request.tx_id, &request.category)?;
            let confidence = parse_confidence(&request.confidence)?;

            let old = classification.classifications.get(&request.tx_id).cloned();
            let mut new_entries = Vec::new();
            let timestamp = now_timestamp();

            if old.as_ref().map(|c| c.category.as_str()) != Some(request.category.as_str()) {
                let entry = AuditEntry {
                    timestamp: timestamp.clone(),
                    actor: request.actor.clone(),
                    tx_id: request.tx_id.clone(),
                    field: "category".to_string(),
                    old_value: old.as_ref().map(|c| c.category.clone()),
                    new_value: request.category.clone(),
                    note: request.note.clone(),
                };
                classification.audit_log.push(entry.clone());
                new_entries.push(to_audit_response(entry));
            }

            if old.as_ref().map(|c| c.confidence) != Some(confidence) {
                let entry = AuditEntry {
                    timestamp,
                    actor: request.actor.clone(),
                    tx_id: request.tx_id.clone(),
                    field: "confidence".to_string(),
                    old_value: old.as_ref().map(|c| c.confidence.to_string()),
                    new_value: confidence.to_string(),
                    note: request.note.clone(),
                };
                classification.audit_log.push(entry.clone());
                new_entries.push(to_audit_response(entry));
            }

            classification.classifications.insert(
                request.tx_id.clone(),
                StoredClassification {
                    category: request.category.clone(),
                    confidence,
                },
            );

            if confidence < Decimal::from_str("0.80").expect("valid decimal literal")
                || request.category.eq_ignore_ascii_case("uncategorized")
            {
                classification.engine.record_review_flag(
                    request.tx_id.clone(),
                    &tx_row.date,
                    "manual classification requires review".to_string(),
                    request.category.clone(),
                    confidence.to_string().parse::<f64>().unwrap_or(0.0),
                );
            }

            (
                ClassifyTransactionResponse {
                    tx_id: request.tx_id.clone(),
                    category: request.category.clone(),
                    confidence: confidence.to_string(),
                    audit_entries: new_entries,
                },
                tx_row,
            )
        };

        let mut payload = BTreeMap::new();
        payload.insert("actor".to_string(), request.actor.clone());
        payload.insert("category".to_string(), request.category.clone());
        payload.insert("confidence".to_string(), request.confidence.clone());
        payload.insert("date".to_string(), tx_row.date.clone());
        payload.insert("note".to_string(), request.note.clone().unwrap_or_default());
        self.append_lifecycle_event(
            event_type,
            Some(request.tx_id.clone()),
            Some(tx_row.source_ref.clone()),
            payload,
        )?;
        self.persist_state()?;

        Ok(response)
    }
}

impl TurboLedgerTools for TurboLedgerService {
    fn list_accounts(&self) -> Result<Vec<AccountSummary>, ToolError> {
        let out = self
            .manifest
            .list_account_ids()
            .into_iter()
            .map(|account_id| AccountSummary { account_id })
            .collect();
        Ok(out)
    }

    fn validate_source_filename(&self, file_name: &str) -> Result<StatementFilename, ToolError> {
        Ok(StatementFilename::parse(file_name)?)
    }

    fn ingest_statement_rows(
        &self,
        request: IngestStatementRowsRequest,
    ) -> Result<IngestStatementRowsResponse, ToolError> {
        let inserted = {
            let mut state = self
                .ingest_state
                .lock()
                .map_err(|_| ToolError::Internal("ingest lock poisoned".to_string()))?;
            state
                .ingest_to_journal_and_workbook(
                    &request.rows,
                    &request.journal_path,
                    &request.workbook_path,
                )
                .map_err(|e| ToolError::Internal(e.to_string()))?
        };

        let mut by_id = BTreeMap::<String, TransactionInput>::new();
        for row in &request.rows {
            by_id.insert(deterministic_tx_id(row), row.clone());
        }
        let mut classification = self
            .classification_state
            .lock()
            .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?;
        for tx in &inserted {
            if let Some(row) = by_id.get(&tx.tx_id) {
                classification.tx_rows.insert(tx.tx_id.clone(), row.clone());
            }
        }
        drop(classification);

        for row in &request.rows {
            let tx_id = deterministic_tx_id(row);
            let mut payload = BTreeMap::new();
            payload.insert("account_id".to_string(), row.account_id.clone());
            payload.insert("amount".to_string(), row.amount.clone());
            payload.insert("date".to_string(), row.date.clone());
            payload.insert("description".to_string(), row.description.clone());
            payload.insert(
                "inserted".to_string(),
                inserted.iter().any(|tx| tx.tx_id == tx_id).to_string(),
            );
            self.append_lifecycle_event(
                "ingest",
                Some(tx_id),
                Some(row.source_ref.clone()),
                payload,
            )?;
        }
        self.persist_state()?;

        let tx_ids = inserted
            .iter()
            .map(|row| row.tx_id.clone())
            .collect::<Vec<_>>();
        Ok(IngestStatementRowsResponse {
            inserted_count: tx_ids.len(),
            tx_ids,
        })
    }

    fn ingest_pdf(&self, request: IngestPdfRequest) -> Result<IngestPdfResponse, ToolError> {
        let file_name = std::path::Path::new(&request.pdf_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                ToolError::InvalidInput("pdf_path must have a valid filename".to_string())
            })?;
        let _parsed = self.validate_source_filename(file_name)?;

        // Derive the allowed base directory from the workbook path to prevent path traversal.
        let allowed_base = request
            .workbook_path
            .parent()
            .ok_or_else(|| {
                ToolError::InvalidInput("workbook_path must have a parent directory".to_string())
            })?
            .to_path_buf();

        for row in &request.extracted_rows {
            let source_path = std::path::Path::new(&row.source_ref);
            let resolved = if source_path.is_absolute() {
                // Absolute paths are allowed only if they reside within the allowed base directory.
                // Reject any `..` components that could escape the base via lexical traversal.
                if source_path
                    .components()
                    .any(|c| c == std::path::Component::ParentDir)
                {
                    return Err(ToolError::InvalidInput(format!(
                        "source_ref '{}' contains path traversal components",
                        row.source_ref
                    )));
                }
                if !source_path.starts_with(&allowed_base) {
                    return Err(ToolError::InvalidInput(format!(
                        "source_ref '{}' resolves outside the allowed directory",
                        row.source_ref
                    )));
                }
                source_path.to_path_buf()
            } else {
                // Relative paths must not contain `..` components.
                if source_path
                    .components()
                    .any(|c| c == std::path::Component::ParentDir)
                {
                    return Err(ToolError::InvalidInput(format!(
                        "source_ref '{}' contains path traversal components",
                        row.source_ref
                    )));
                }
                allowed_base.join(source_path)
            };
            if resolved.exists() {
                continue;
            }
            if let Some(parent) = resolved.parent() {
                std::fs::create_dir_all(parent).map_err(|e| ToolError::Internal(e.to_string()))?;
            }
            let bytes = request.raw_context_bytes.as_deref().ok_or_else(|| {
                ToolError::InvalidInput(
                    "raw_context_bytes required when source_ref file does not exist".to_string(),
                )
            })?;
            std::fs::write(&resolved, bytes).map_err(|e| ToolError::Internal(e.to_string()))?;
        }

        let response = self.ingest_statement_rows(IngestStatementRowsRequest {
            journal_path: request.journal_path,
            workbook_path: request.workbook_path,
            rows: request.extracted_rows,
        })?;
        Ok(IngestPdfResponse {
            inserted_count: response.inserted_count,
            tx_ids: response.tx_ids,
        })
    }

    fn get_raw_context(
        &self,
        request: GetRawContextRequest,
    ) -> Result<GetRawContextResponse, ToolError> {
        let allowed_base = self
            .workbook_path()
            .parent()
            .ok_or_else(|| {
                ToolError::InvalidInput("workbook_path must have a parent directory".to_string())
            })?
            .to_path_buf();

        let rkyv_path = &request.rkyv_ref;
        if rkyv_path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(ToolError::InvalidInput(format!(
                "rkyv_ref '{}' contains path traversal components",
                rkyv_path.display()
            )));
        }
        let resolved = if rkyv_path.is_absolute() {
            if !rkyv_path.starts_with(&allowed_base) {
                return Err(ToolError::InvalidInput(format!(
                    "rkyv_ref '{}' resolves outside the allowed directory",
                    rkyv_path.display()
                )));
            }
            rkyv_path.to_path_buf()
        } else {
            allowed_base.join(rkyv_path)
        };

        let bytes = std::fs::read(&resolved).map_err(|e| ToolError::Internal(e.to_string()))?;
        Ok(GetRawContextResponse { bytes })
    }

    fn run_rhai_rule(&self, request: RunRhaiRuleRequest) -> Result<RunRhaiRuleResponse, ToolError> {
        let sample = SampleTransaction {
            tx_id: request.sample_tx.tx_id,
            account_id: request.sample_tx.account_id,
            date: request.sample_tx.date,
            amount: request.sample_tx.amount,
            description: request.sample_tx.description,
        };
        let classification = self
            .classification_state
            .lock()
            .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?
            .engine
            .run_rule_from_file(&request.rule_file, &sample)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        Ok(RunRhaiRuleResponse {
            category: classification.category,
            confidence: classification.confidence,
            review: classification.needs_review,
            reason: classification.reason,
        })
    }

    fn classify_ingested(
        &self,
        request: ClassifyIngestedRequest,
    ) -> Result<ClassifyIngestedResponse, ToolError> {
        let mut classification = self
            .classification_state
            .lock()
            .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?;

        let rows = classification.tx_rows.values().cloned().collect::<Vec<_>>();
        let batch = classification
            .engine
            .classify_rows_from_file(&request.rule_file, &rows, request.review_threshold)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;

        let timestamp = now_timestamp();
        let mut results = Vec::with_capacity(batch.classifications.len());
        for c in batch.classifications {
            let confidence = Decimal::try_from(c.confidence).unwrap_or(Decimal::ZERO);
            let old = classification.classifications.get(&c.tx_id).cloned();
            // Emit audit entries for every change, including first classifications (old_value: None).
            if old.as_ref().map(|e| e.category.as_str()) != Some(c.category.as_str()) {
                classification.audit_log.push(AuditEntry {
                    timestamp: timestamp.clone(),
                    actor: "rhai-rule".to_string(),
                    tx_id: c.tx_id.clone(),
                    field: "category".to_string(),
                    old_value: old.as_ref().map(|e| e.category.clone()),
                    new_value: c.category.clone(),
                    note: Some(c.reason.clone()),
                });
            }
            if old.as_ref().map(|e| e.confidence) != Some(confidence) {
                classification.audit_log.push(AuditEntry {
                    timestamp: timestamp.clone(),
                    actor: "rhai-rule".to_string(),
                    tx_id: c.tx_id.clone(),
                    field: "confidence".to_string(),
                    old_value: old.as_ref().map(|e| e.confidence.to_string()),
                    new_value: confidence.to_string(),
                    note: Some(c.reason.clone()),
                });
            }
            classification.classifications.insert(
                c.tx_id.clone(),
                StoredClassification {
                    category: c.category.clone(),
                    confidence,
                },
            );
            results.push(ClassifiedTxResponse {
                tx_id: c.tx_id,
                category: c.category,
                confidence: c.confidence,
                needs_review: c.needs_review,
                reason: c.reason,
            });
        }
        drop(classification);
        self.persist_state()?;

        Ok(ClassifyIngestedResponse {
            classifications: results,
        })
    }

    fn query_flags(&self, request: QueryFlagsRequest) -> Result<QueryFlagsResponse, ToolError> {
        let status = match request.status {
            FlagStatusRequest::Open => FlagStatus::Open,
            FlagStatusRequest::Resolved => FlagStatus::Resolved,
        };
        let flags = self
            .classification_state
            .lock()
            .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?
            .engine
            .query_flags(request.year, status);

        Ok(QueryFlagsResponse {
            flags: flags
                .into_iter()
                .map(|f| FlagRecordResponse {
                    tx_id: f.tx_id,
                    year: f.year,
                    status: match f.status {
                        FlagStatus::Open => FlagStatusRequest::Open,
                        FlagStatus::Resolved => FlagStatusRequest::Resolved,
                    },
                    reason: f.reason,
                    category: f.category,
                    confidence: f.confidence,
                })
                .collect(),
        })
    }

    fn classify_transaction(
        &self,
        request: ClassifyTransactionRequest,
    ) -> Result<ClassifyTransactionResponse, ToolError> {
        self.apply_classification_action(request, "classification")
    }

    fn reconcile_excel_classification(
        &self,
        request: ReconcileExcelClassificationRequest,
    ) -> Result<ClassifyTransactionResponse, ToolError> {
        self.apply_classification_action(
            ClassifyTransactionRequest {
                tx_id: request.tx_id,
                category: request.category,
                confidence: request.confidence,
                note: request.note,
                actor: request.actor,
            },
            "reconciliation",
        )
    }

    fn query_audit_log(
        &self,
        _request: QueryAuditLogRequest,
    ) -> Result<QueryAuditLogResponse, ToolError> {
        let entries = self
            .classification_state
            .lock()
            .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?
            .audit_log
            .clone();
        Ok(QueryAuditLogResponse {
            entries: entries.into_iter().map(to_audit_response).collect(),
        })
    }

    fn export_cpa_workbook(
        &self,
        request: ExportCpaWorkbookRequest,
    ) -> Result<ExportCpaWorkbookResponse, ToolError> {
        let classification = self
            .classification_state
            .lock()
            .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?;

        let active_year = self.manifest.session.active_year;

        // Workbook export is an artifact projection, not a mutable source of truth.
        // Rebuild the full accountant-facing workbook from canonical service state on
        // every export so the handoff file stays consistent with the declared contract.
        let mut workbook = Workbook::new();
        for sheet_name in REQUIRED_SHEETS {
            workbook
                .add_worksheet()
                .set_name(*sheet_name)
                .map_err(map_xlsx)?;
        }
        let mut sheets_written: usize = REQUIRED_SHEETS.len();

        let mut categories = BTreeSet::new();
        categories.insert("Uncategorized".to_string());
        for entry in classification.classifications.values() {
            categories.insert(entry.category.clone());
        }

        {
            let meta_sheet = workbook
                .worksheet_from_name("META.config")
                .map_err(map_xlsx)?;
            meta_sheet
                .write_string(0, 0, "workbook_path")
                .map_err(map_xlsx)?;
            meta_sheet
                .write_string(0, 1, "active_year")
                .map_err(map_xlsx)?;
            meta_sheet
                .write_string(1, 0, &self.manifest.session.workbook_path)
                .map_err(map_xlsx)?;
            meta_sheet
                .write_number(1, 1, f64::from(active_year))
                .map_err(map_xlsx)?;
        }

        {
            let account_sheet = workbook
                .worksheet_from_name("ACCT.registry")
                .map_err(map_xlsx)?;
            account_sheet
                .write_string(0, 0, "account_id")
                .map_err(map_xlsx)?;
            account_sheet
                .write_string(0, 1, "institution")
                .map_err(map_xlsx)?;
            account_sheet
                .write_string(0, 2, "account_type")
                .map_err(map_xlsx)?;
            account_sheet
                .write_string(0, 3, "currency")
                .map_err(map_xlsx)?;
            for (idx, (account_id, account)) in self.manifest.accounts.iter().enumerate() {
                let row = (idx + 1) as u32;
                account_sheet
                    .write_string(row, 0, account_id)
                    .map_err(map_xlsx)?;
                account_sheet
                    .write_string(row, 1, &account.institution)
                    .map_err(map_xlsx)?;
                account_sheet
                    .write_string(row, 2, &account.account_type)
                    .map_err(map_xlsx)?;
                account_sheet
                    .write_string(row, 3, &account.currency)
                    .map_err(map_xlsx)?;
            }
        }

        {
            let cat_sheet = workbook
                .worksheet_from_name("CAT.taxonomy")
                .map_err(map_xlsx)?;
            cat_sheet.write_string(0, 0, "category").map_err(map_xlsx)?;
            for (idx, category) in categories.iter().enumerate() {
                cat_sheet
                    .write_string((idx + 1) as u32, 0, category)
                    .map_err(map_xlsx)?;
            }
        }

        let mut by_account = BTreeMap::<String, Vec<(String, TransactionInput)>>::new();
        for (tx_id, row) in &classification.tx_rows {
            by_account
                .entry(row.account_id.clone())
                .or_default()
                .push((tx_id.clone(), row.clone()));
        }

        for (account, rows) in by_account {
            let sheet_name = format!("TX.{account}");
            let ws = workbook
                .add_worksheet()
                .set_name(sheet_name)
                .map_err(map_xlsx)?;
            sheets_written += 1;
            ws.write_string(0, 0, "tx_id").map_err(map_xlsx)?;
            ws.write_string(0, 1, "date").map_err(map_xlsx)?;
            ws.write_string(0, 2, "amount").map_err(map_xlsx)?;
            ws.write_string(0, 3, "description").map_err(map_xlsx)?;
            ws.write_string(0, 4, "category").map_err(map_xlsx)?;
            ws.write_string(0, 5, "confidence").map_err(map_xlsx)?;
            ws.write_string(0, 6, "source_ref").map_err(map_xlsx)?;

            for (idx, (tx_id, row)) in rows.into_iter().enumerate() {
                let line = (idx + 1) as u32;
                let classified = classification.classifications.get(&tx_id);
                ws.write_string(line, 0, tx_id).map_err(map_xlsx)?;
                ws.write_string(line, 1, &row.date).map_err(map_xlsx)?;
                ws.write_string(line, 2, &row.amount).map_err(map_xlsx)?;
                ws.write_string(line, 3, &row.description)
                    .map_err(map_xlsx)?;
                ws.write_string(
                    line,
                    4,
                    classified
                        .map(|c| c.category.as_str())
                        .unwrap_or("Uncategorized"),
                )
                .map_err(map_xlsx)?;
                ws.write_string(
                    line,
                    5,
                    classified
                        .map(|c| c.confidence.to_string())
                        .unwrap_or_else(|| "0.0".to_string()),
                )
                .map_err(map_xlsx)?;
                ws.write_string(line, 6, &row.source_ref)
                    .map_err(map_xlsx)?;
            }
        }

        let open_flags = classification
            .engine
            .query_flags(active_year as i32, FlagStatus::Open);
        let resolved_flags = classification
            .engine
            .query_flags(active_year as i32, FlagStatus::Resolved);
        {
            let ws = workbook
                .worksheet_from_name("FLAGS.open")
                .map_err(map_xlsx)?;
            ws.write_string(0, 0, "tx_id").map_err(map_xlsx)?;
            ws.write_string(0, 1, "reason").map_err(map_xlsx)?;
            for (idx, flag) in open_flags.iter().enumerate() {
                ws.write_string((idx + 1) as u32, 0, &flag.tx_id)
                    .map_err(map_xlsx)?;
                ws.write_string((idx + 1) as u32, 1, &flag.reason)
                    .map_err(map_xlsx)?;
            }
        }
        {
            let ws = workbook
                .worksheet_from_name("FLAGS.resolved")
                .map_err(map_xlsx)?;
            ws.write_string(0, 0, "tx_id").map_err(map_xlsx)?;
            ws.write_string(0, 1, "reason").map_err(map_xlsx)?;
            for (idx, flag) in resolved_flags.iter().enumerate() {
                ws.write_string((idx + 1) as u32, 0, &flag.tx_id)
                    .map_err(map_xlsx)?;
                ws.write_string((idx + 1) as u32, 1, &flag.reason)
                    .map_err(map_xlsx)?;
            }
        }

        write_schedule_sheet(
            &mut workbook,
            "SCHED.C",
            &build_schedule_summary_from_classification(
                &classification,
                active_year as i32,
                ScheduleKindRequest::ScheduleC,
            ),
        )?;
        write_schedule_sheet(
            &mut workbook,
            "SCHED.D",
            &build_schedule_summary_from_classification(
                &classification,
                active_year as i32,
                ScheduleKindRequest::ScheduleD,
            ),
        )?;
        write_schedule_sheet(
            &mut workbook,
            "SCHED.E",
            &build_schedule_summary_from_classification(
                &classification,
                active_year as i32,
                ScheduleKindRequest::ScheduleE,
            ),
        )?;
        write_schedule_sheet(
            &mut workbook,
            "FBAR.accounts",
            &build_schedule_summary_from_classification(
                &classification,
                active_year as i32,
                ScheduleKindRequest::Fbar,
            ),
        )?;

        {
            let audit_sheet = workbook
                .worksheet_from_name("AUDIT.log")
                .map_err(map_xlsx)?;
            audit_sheet
                .write_string(0, 0, "timestamp")
                .map_err(map_xlsx)?;
            audit_sheet.write_string(0, 1, "actor").map_err(map_xlsx)?;
            audit_sheet.write_string(0, 2, "tx_id").map_err(map_xlsx)?;
            audit_sheet.write_string(0, 3, "field").map_err(map_xlsx)?;
            audit_sheet
                .write_string(0, 4, "old_value")
                .map_err(map_xlsx)?;
            audit_sheet
                .write_string(0, 5, "new_value")
                .map_err(map_xlsx)?;
            audit_sheet.write_string(0, 6, "note").map_err(map_xlsx)?;

            for (idx, entry) in classification.audit_log.iter().enumerate() {
                let row = (idx + 1) as u32;
                audit_sheet
                    .write_string(row, 0, &entry.timestamp)
                    .map_err(map_xlsx)?;
                audit_sheet
                    .write_string(row, 1, &entry.actor)
                    .map_err(map_xlsx)?;
                audit_sheet
                    .write_string(row, 2, &entry.tx_id)
                    .map_err(map_xlsx)?;
                audit_sheet
                    .write_string(row, 3, &entry.field)
                    .map_err(map_xlsx)?;
                audit_sheet
                    .write_string(row, 4, entry.old_value.as_deref().unwrap_or(""))
                    .map_err(map_xlsx)?;
                audit_sheet
                    .write_string(row, 5, &entry.new_value)
                    .map_err(map_xlsx)?;
                audit_sheet
                    .write_string(row, 6, entry.note.as_deref().unwrap_or(""))
                    .map_err(map_xlsx)?;
            }
        }

        workbook.save(&request.workbook_path).map_err(map_xlsx)?;
        Ok(ExportCpaWorkbookResponse { sheets_written })
    }

    fn get_schedule_summary(
        &self,
        request: GetScheduleSummaryRequest,
    ) -> Result<GetScheduleSummaryResponse, ToolError> {
        let classification = self
            .classification_state
            .lock()
            .map_err(|_| ToolError::Internal("classification lock poisoned".to_string()))?;
        Ok(build_schedule_summary_from_classification(
            &classification,
            request.year,
            request.schedule,
        ))
    }
}

fn parse_confidence(input: &str) -> Result<Decimal, ToolError> {
    let confidence = Decimal::from_str(input)
        .map_err(|_| ToolError::InvalidInput("confidence must be a valid decimal".to_string()))?;
    if confidence < Decimal::ZERO || confidence > Decimal::ONE {
        return Err(ToolError::InvalidInput(
            "confidence must be between 0 and 1".to_string(),
        ));
    }
    Ok(confidence)
}

fn validate_invariants(
    row: &TransactionInput,
    tx_id: &str,
    category: &str,
) -> Result<(), ToolError> {
    if category.trim().is_empty() {
        return Err(ToolError::InvalidInput(
            "category must not be empty".to_string(),
        ));
    }
    Decimal::from_str(row.amount.trim())
        .map_err(|_| ToolError::InvalidInput("invalid amount decimal".to_string()))?;

    if deterministic_tx_id(row) != tx_id {
        return Err(ToolError::InvalidInput(
            "tx_id invariant violation: deterministic hash mismatch".to_string(),
        ));
    }

    let parts: Vec<&str> = row.date.split('-').collect();
    if parts.len() != 3
        || parts[0].parse::<u32>().is_err()
        || parts[1].parse::<u32>().is_err()
        || parts[2].parse::<u32>().is_err()
    {
        return Err(ToolError::InvalidInput(
            "schema invariant violation: date must be YYYY-MM-DD".to_string(),
        ));
    }
    Ok(())
}

fn now_timestamp() -> String {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    }
}

fn persisted_state_path(workbook_path: &std::path::Path) -> PathBuf {
    PathBuf::from(format!("{}.ledgerr-state.json", workbook_path.display()))
}

fn load_persisted_state(
    workbook_path: &std::path::Path,
) -> Result<PersistedServiceState, ToolError> {
    let sidecar_path = persisted_state_path(workbook_path);
    if !sidecar_path.exists() {
        return Ok(PersistedServiceState::default());
    }

    let bytes = std::fs::read(&sidecar_path).map_err(|e| {
        ToolError::Internal(format!(
            "failed to read persisted state '{}': {e}",
            sidecar_path.display()
        ))
    })?;
    let state: PersistedServiceState = serde_json::from_slice(&bytes).map_err(|e| {
        ToolError::Internal(format!(
            "failed to parse persisted state '{}': {e}",
            sidecar_path.display()
        ))
    })?;
    if state.version != PERSISTED_STATE_VERSION {
        return Err(ToolError::Internal(format!(
            "unsupported persisted state version {} in '{}'",
            state.version,
            sidecar_path.display()
        )));
    }
    Ok(state)
}

fn persist_state_to_path(
    sidecar_path: &std::path::Path,
    state: &PersistedServiceState,
) -> Result<(), ToolError> {
    if let Some(parent) = sidecar_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|e| {
            ToolError::Internal(format!(
                "failed to create persisted state directory '{}': {e}",
                parent.display()
            ))
        })?;
    }
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|e| ToolError::Internal(format!("failed to serialize persisted state: {e}")))?;
    // Write to a sibling temp file first, then rename atomically so a mid-write
    // crash never leaves a truncated/corrupt sidecar that causes startup to fail closed.
    let tmp_path: std::path::PathBuf = {
        let mut tmp_os_string = sidecar_path.as_os_str().to_os_string();
        tmp_os_string.push(".tmp");
        tmp_os_string.into()
    };
    std::fs::write(&tmp_path, &bytes).map_err(|e| {
        ToolError::Internal(format!(
            "failed to write persisted state temp file '{}': {e}",
            tmp_path.display()
        ))
    })?;
    std::fs::rename(&tmp_path, sidecar_path).map_err(|e| {
        ToolError::Internal(format!(
            "failed to rename persisted state '{}' to '{}': {e}",
            tmp_path.display(),
            sidecar_path.display()
        ))
    })
}

fn to_audit_response(entry: AuditEntry) -> AuditEntryResponse {
    AuditEntryResponse {
        timestamp: entry.timestamp,
        actor: entry.actor,
        tx_id: entry.tx_id,
        field: entry.field,
        old_value: entry.old_value,
        new_value: entry.new_value,
        note: entry.note,
    }
}

fn map_xlsx(err: rust_xlsxwriter::XlsxError) -> ToolError {
    ToolError::Internal(err.to_string())
}

fn derive_year(date: &str) -> i32 {
    date.split('-')
        .next()
        .and_then(|y| y.parse::<i32>().ok())
        .unwrap_or(0)
}

fn schedule_for_category(category: &str) -> Option<ScheduleKindRequest> {
    let category = category.to_ascii_lowercase();
    if category.contains("crypto") || category.contains("capital") || category.contains("baddebt") {
        return Some(ScheduleKindRequest::ScheduleD);
    }
    if category.contains("rent") || category.contains("property") {
        return Some(ScheduleKindRequest::ScheduleE);
    }
    if category != "uncategorized" {
        return Some(ScheduleKindRequest::ScheduleC);
    }
    None
}

fn write_schedule_sheet(
    workbook: &mut Workbook,
    sheet_name: &str,
    summary: &GetScheduleSummaryResponse,
) -> Result<(), ToolError> {
    let sheet = workbook.worksheet_from_name(sheet_name).map_err(map_xlsx)?;
    sheet.write_string(0, 0, "key").map_err(map_xlsx)?;
    sheet.write_string(0, 1, "total").map_err(map_xlsx)?;
    for (idx, line) in summary.lines.iter().enumerate() {
        let row = (idx + 1) as u32;
        sheet.write_string(row, 0, &line.key).map_err(map_xlsx)?;
        sheet.write_number(row, 1, line.total).map_err(map_xlsx)?;
    }
    Ok(())
}

fn build_schedule_summary_from_classification(
    classification: &ClassificationState,
    year: i32,
    schedule: ScheduleKindRequest,
) -> GetScheduleSummaryResponse {
    let mut grouped = BTreeMap::<String, Decimal>::new();
    for (tx_id, row) in &classification.tx_rows {
        if derive_year(&row.date) != year {
            continue;
        }
        let amount = match Decimal::from_str(row.amount.trim()) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let key = match schedule {
            ScheduleKindRequest::Fbar => row.account_id.clone(),
            _ => {
                let category = classification
                    .classifications
                    .get(tx_id)
                    .map(|c| c.category.clone())
                    .unwrap_or_else(|| "Uncategorized".to_string());
                if schedule_for_category(&category) != Some(schedule) {
                    continue;
                }
                category
            }
        };

        if schedule == ScheduleKindRequest::Fbar {
            let abs_amount = amount.abs();
            let current = grouped.entry(key).or_insert(Decimal::ZERO);
            if abs_amount > *current {
                *current = abs_amount;
            }
        } else {
            *grouped.entry(key).or_insert(Decimal::ZERO) += amount;
        }
    }

    let lines = grouped
        .into_iter()
        .map(|(key, total)| ScheduleLineResponse {
            key,
            total: decimal_to_f64(total),
        })
        .collect::<Vec<_>>();
    let total = lines.iter().map(|line| line.total).sum::<f64>();
    GetScheduleSummaryResponse {
        year,
        schedule,
        total,
        lines,
    }
}

fn decimal_to_f64(value: Decimal) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}
