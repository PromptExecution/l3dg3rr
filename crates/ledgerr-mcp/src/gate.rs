use std::path::PathBuf;

use crossbeam::channel::Sender;

use crate::{
    ClassifyIngestedRequest, ClassifyIngestedResponse, ClassifyTransactionRequest,
    ClassifyTransactionResponse, DocumentInventoryRequest, DocumentInventoryResponse,
    EventHistoryFilter, EventHistoryResponse, ExportCpaWorkbookRequest,
    ExportCpaWorkbookResponse, GetRawContextRequest, GetRawContextResponse,
    GetScheduleSummaryRequest, GetScheduleSummaryResponse, HsmResumeRequest, HsmResumeResponse,
    HsmStatusRequest, HsmStatusResponse, HsmTransitionRequest, HsmTransitionResponse,
    IngestImageRequest, IngestImageResponse, IngestPdfRequest, IngestPdfResponse,
    IngestStatementRowsRequest, IngestStatementRowsResponse, NormalizeFilenameRequest,
    NormalizeFilenameResponse, OntologyExportSnapshotRequest, OntologyExportSnapshotResponse,
    OntologyQueryPathRequest, OntologyQueryPathResponse, OntologyUpsertEdgesRequest,
    OntologyUpsertEdgesResponse, OntologyUpsertEntitiesRequest, OntologyUpsertEntitiesResponse,
    QueryAuditLogRequest, QueryAuditLogResponse, QueryFlagsRequest, QueryFlagsResponse,
    ReconciliationStageRequest, ReconciliationStageResponse, ReplayLifecycleRequest,
    ReplayLifecycleResponse, RunRhaiRuleRequest, RunRhaiRuleResponse, SyncFsMetadataRequest,
    SyncFsMetadataResponse, TaxAmbiguityReviewRequest, TaxAmbiguityReviewResponse,
    TaxAssistRequest, TaxAssistResponse, TaxEvidenceChainRequest, TaxEvidenceChainResponse,
    ToolError,
};

use ledger_core::filename::StatementFilename;

#[allow(clippy::large_enum_variant)]
pub enum GateMessage {
    ListAccounts {
        reply_tx: Sender<Result<Vec<crate::AccountSummary>, ToolError>>,
    },
    ListAccountsTool {
        request: crate::ListAccountsRequest,
        reply_tx: Sender<Result<crate::ListAccountsResponse, ToolError>>,
    },
    DocumentInventory {
        request: DocumentInventoryRequest,
        reply_tx: Sender<Result<DocumentInventoryResponse, ToolError>>,
    },
    ValidateFilename {
        file_name: String,
        reply_tx: Sender<Result<StatementFilename, ToolError>>,
    },
    IngestStatementRows {
        request: IngestStatementRowsRequest,
        reply_tx: Sender<Result<IngestStatementRowsResponse, ToolError>>,
    },
    IngestPdf {
        request: IngestPdfRequest,
        reply_tx: Sender<Result<IngestPdfResponse, ToolError>>,
    },
    GetRawContext {
        request: GetRawContextRequest,
        reply_tx: Sender<Result<GetRawContextResponse, ToolError>>,
    },
    RunRhaiRule {
        request: RunRhaiRuleRequest,
        reply_tx: Sender<Result<RunRhaiRuleResponse, ToolError>>,
    },
    ClassifyIngested {
        request: ClassifyIngestedRequest,
        reply_tx: Sender<Result<ClassifyIngestedResponse, ToolError>>,
    },
    QueryFlags {
        request: QueryFlagsRequest,
        reply_tx: Sender<Result<QueryFlagsResponse, ToolError>>,
    },
    ClassifyTransaction {
        request: ClassifyTransactionRequest,
        reply_tx: Sender<Result<ClassifyTransactionResponse, ToolError>>,
    },
    ReconcileExcelClassification {
        request: crate::ReconcileExcelClassificationRequest,
        reply_tx: Sender<Result<ClassifyTransactionResponse, ToolError>>,
    },
    QueryAuditLog {
        request: QueryAuditLogRequest,
        reply_tx: Sender<Result<QueryAuditLogResponse, ToolError>>,
    },
    ExportCpaWorkbook {
        request: ExportCpaWorkbookRequest,
        reply_tx: Sender<Result<ExportCpaWorkbookResponse, ToolError>>,
    },
    GetScheduleSummary {
        request: GetScheduleSummaryRequest,
        reply_tx: Sender<Result<GetScheduleSummaryResponse, ToolError>>,
    },
    HsmTransition {
        request: HsmTransitionRequest,
        reply_tx: Sender<Result<HsmTransitionResponse, ToolError>>,
    },
    HsmStatus {
        request: HsmStatusRequest,
        reply_tx: Sender<Result<HsmStatusResponse, ToolError>>,
    },
    HsmResume {
        request: HsmResumeRequest,
        reply_tx: Sender<Result<HsmResumeResponse, ToolError>>,
    },
    EventHistory {
        filter: EventHistoryFilter,
        reply_tx: Sender<Result<EventHistoryResponse, ToolError>>,
    },
    ReplayLifecycle {
        request: ReplayLifecycleRequest,
        reply_tx: Sender<Result<ReplayLifecycleResponse, ToolError>>,
    },
    TaxAssist {
        request: TaxAssistRequest,
        reply_tx: Sender<Result<TaxAssistResponse, ToolError>>,
    },
    TaxEvidenceChain {
        request: TaxEvidenceChainRequest,
        reply_tx: Sender<Result<TaxEvidenceChainResponse, ToolError>>,
    },
    TaxAmbiguityReview {
        request: TaxAmbiguityReviewRequest,
        reply_tx: Sender<Result<TaxAmbiguityReviewResponse, ToolError>>,
    },
    ValidateReconciliationStage {
        request: ReconciliationStageRequest,
        reply_tx: Sender<Result<ReconciliationStageResponse, ToolError>>,
    },
    ReconcileReconciliationStage {
        request: ReconciliationStageRequest,
        reply_tx: Sender<Result<ReconciliationStageResponse, ToolError>>,
    },
    CommitReconciliationStage {
        request: ReconciliationStageRequest,
        reply_tx: Sender<Result<ReconciliationStageResponse, ToolError>>,
    },
    AdjustTransaction {
        request: ClassifyTransactionRequest,
        reply_tx: Sender<Result<ClassifyTransactionResponse, ToolError>>,
    },
    OntologyUpsertEntities {
        request: OntologyUpsertEntitiesRequest,
        reply_tx: Sender<Result<OntologyUpsertEntitiesResponse, ToolError>>,
    },
    OntologyUpsertEdges {
        request: OntologyUpsertEdgesRequest,
        reply_tx: Sender<Result<OntologyUpsertEdgesResponse, ToolError>>,
    },
    OntologyQueryPath {
        request: OntologyQueryPathRequest,
        reply_tx: Sender<Result<OntologyQueryPathResponse, ToolError>>,
    },
    OntologyExportSnapshot {
        request: OntologyExportSnapshotRequest,
        reply_tx: Sender<Result<OntologyExportSnapshotResponse, ToolError>>,
    },
    IngestImage {
        request: IngestImageRequest,
        reply_tx: Sender<Result<IngestImageResponse, ToolError>>,
    },
    ApplyTags {
        request: crate::ApplyTagsRequest,
        reply_tx: Sender<Result<crate::ApplyTagsResponse, ToolError>>,
    },
    RemoveTags {
        request: crate::ApplyTagsRequest,
        reply_tx: Sender<Result<crate::ApplyTagsResponse, ToolError>>,
    },
    ListTagged {
        request: crate::ListTaggedRequest,
        reply_tx: Sender<Result<crate::ListTaggedResponse, ToolError>>,
    },
    SyncFsMetadata {
        request: SyncFsMetadataRequest,
        reply_tx: Sender<Result<SyncFsMetadataResponse, ToolError>>,
    },
    NormalizeFilename {
        request: NormalizeFilenameRequest,
        reply_tx: Sender<Result<NormalizeFilenameResponse, ToolError>>,
    },
    #[cfg(feature = "xero")]
    #[cfg(feature = "xero")]
    XeroGetAuthUrl {
        reply_tx: Sender<Result<String, ToolError>>,
    },
    #[cfg(feature = "xero")]
    #[cfg(feature = "xero")]
    XeroExchangeCode {
        code: String,
        state: String,
        reply_tx: Sender<Result<serde_json::Value, ToolError>>,
    },
    #[cfg(feature = "xero")]
    #[cfg(feature = "xero")]
    XeroFetchContacts {
        search: Option<String>,
        reply_tx: Sender<Result<serde_json::Value, ToolError>>,
    },
    #[cfg(feature = "xero")]
    #[cfg(feature = "xero")]
    XeroFetchAccounts {
        reply_tx: Sender<Result<serde_json::Value, ToolError>>,
    },
    #[cfg(feature = "xero")]
    #[cfg(feature = "xero")]
    XeroFetchBankAccounts {
        reply_tx: Sender<Result<serde_json::Value, ToolError>>,
    },
    #[cfg(feature = "xero")]
    #[cfg(feature = "xero")]
    XeroFetchInvoices {
        status: Option<String>,
        reply_tx: Sender<Result<serde_json::Value, ToolError>>,
    },
    #[cfg(feature = "xero")]
    #[cfg(feature = "xero")]
    XeroLinkEntity {
        local_id: String,
        xero_entity_type: String,
        xero_id: String,
        display_name: String,
        ontology_path: Option<PathBuf>,
        reply_tx: Sender<Result<serde_json::Value, ToolError>>,
    },
    #[cfg(feature = "xero")]
    XeroSyncCatalog {
        ontology_path: PathBuf,
        reply_tx: Sender<Result<serde_json::Value, ToolError>>,
    },
    Shutdown,
}
