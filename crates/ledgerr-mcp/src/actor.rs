use crossbeam::channel::{Receiver, Sender};

use crate::{
    gate::GateMessage, ApplyTagsRequest, ClassifyIngestedRequest, ClassifyTransactionRequest,
    DocumentInventoryRequest, ExportCpaWorkbookRequest, GetRawContextRequest,
    GetScheduleSummaryRequest, HsmResumeRequest, HsmStatusRequest, HsmTransitionRequest,
    IngestImageRequest, IngestPdfRequest, IngestStatementRowsRequest, ListAccountsRequest,
    ListTaggedRequest, NormalizeFilenameRequest, OntologyExportSnapshotRequest,
    OntologyQueryPathRequest, OntologyUpsertEdgesRequest, OntologyUpsertEntitiesRequest,
    QueryAuditLogRequest, QueryFlagsRequest, ReconciliationStageRequest, ReplayLifecycleRequest,
    RunRhaiRuleRequest, SyncFsMetadataRequest, TaxAmbiguityReviewRequest, TaxAssistRequest,
    TaxEvidenceChainRequest, ToolError, TurboLedgerService, TurboLedgerTools,
};

#[derive(Clone)]
pub struct ServiceHandle {
    tx: Sender<GateMessage>,
}

impl ServiceHandle {
    pub fn new(tx: Sender<GateMessage>) -> Self {
        Self { tx }
    }

    fn send<F, R>(&self, msg: F) -> Result<R, ToolError>
    where
        F: FnOnce(Sender<Result<R, ToolError>>) -> GateMessage,
    {
        let (reply_tx, reply_rx) = crossbeam::channel::bounded::<Result<R, ToolError>>(1);
        self.tx
            .send(msg(reply_tx))
            .map_err(|_| ToolError::Internal("actor channel disconnected".to_string()))?;
        reply_rx
            .recv()
            .map_err(|_| ToolError::Internal("actor reply channel disconnected".to_string()))?
    }

    pub fn list_accounts(&self) -> Result<Vec<crate::AccountSummary>, ToolError> {
        self.send(|reply_tx| GateMessage::ListAccounts { reply_tx })
    }

    pub fn list_accounts_tool(
        &self,
        request: ListAccountsRequest,
    ) -> Result<crate::ListAccountsResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ListAccountsTool { request, reply_tx })
    }

    pub fn document_inventory(
        &self,
        request: DocumentInventoryRequest,
    ) -> Result<crate::DocumentInventoryResponse, ToolError> {
        self.send(|reply_tx| GateMessage::DocumentInventory { request, reply_tx })
    }

    pub fn validate_source_filename(
        &self,
        file_name: String,
    ) -> Result<ledger_core::filename::StatementFilename, ToolError> {
        self.send(|reply_tx| GateMessage::ValidateFilename { file_name, reply_tx })
    }

    pub fn ingest_statement_rows(
        &self,
        request: IngestStatementRowsRequest,
    ) -> Result<crate::IngestStatementRowsResponse, ToolError> {
        self.send(|reply_tx| GateMessage::IngestStatementRows { request, reply_tx })
    }

    pub fn ingest_pdf(
        &self,
        request: IngestPdfRequest,
    ) -> Result<crate::IngestPdfResponse, ToolError> {
        self.send(|reply_tx| GateMessage::IngestPdf { request, reply_tx })
    }

    pub fn get_raw_context(
        &self,
        request: GetRawContextRequest,
    ) -> Result<crate::GetRawContextResponse, ToolError> {
        self.send(|reply_tx| GateMessage::GetRawContext { request, reply_tx })
    }

    pub fn run_rhai_rule(
        &self,
        request: RunRhaiRuleRequest,
    ) -> Result<crate::RunRhaiRuleResponse, ToolError> {
        self.send(|reply_tx| GateMessage::RunRhaiRule { request, reply_tx })
    }

    pub fn classify_ingested(
        &self,
        request: ClassifyIngestedRequest,
    ) -> Result<crate::ClassifyIngestedResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ClassifyIngested { request, reply_tx })
    }

    pub fn query_flags(
        &self,
        request: QueryFlagsRequest,
    ) -> Result<crate::QueryFlagsResponse, ToolError> {
        self.send(|reply_tx| GateMessage::QueryFlags { request, reply_tx })
    }

    pub fn classify_transaction(
        &self,
        request: ClassifyTransactionRequest,
    ) -> Result<crate::ClassifyTransactionResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ClassifyTransaction { request, reply_tx })
    }

    pub fn reconcile_excel_classification(
        &self,
        request: crate::ReconcileExcelClassificationRequest,
    ) -> Result<crate::ClassifyTransactionResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ReconcileExcelClassification { request, reply_tx })
    }

    pub fn query_audit_log(
        &self,
        request: QueryAuditLogRequest,
    ) -> Result<crate::QueryAuditLogResponse, ToolError> {
        self.send(|reply_tx| GateMessage::QueryAuditLog { request, reply_tx })
    }

    pub fn export_cpa_workbook(
        &self,
        request: ExportCpaWorkbookRequest,
    ) -> Result<crate::ExportCpaWorkbookResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ExportCpaWorkbook { request, reply_tx })
    }

    pub fn get_schedule_summary(
        &self,
        request: GetScheduleSummaryRequest,
    ) -> Result<crate::GetScheduleSummaryResponse, ToolError> {
        self.send(|reply_tx| GateMessage::GetScheduleSummary { request, reply_tx })
    }

    pub fn hsm_transition(
        &self,
        request: HsmTransitionRequest,
    ) -> Result<crate::HsmTransitionResponse, ToolError> {
        self.send(|reply_tx| GateMessage::HsmTransition { request, reply_tx })
    }

    pub fn hsm_status(
        &self,
        request: HsmStatusRequest,
    ) -> Result<crate::HsmStatusResponse, ToolError> {
        self.send(|reply_tx| GateMessage::HsmStatus { request, reply_tx })
    }

    pub fn hsm_resume(
        &self,
        request: HsmResumeRequest,
    ) -> Result<crate::HsmResumeResponse, ToolError> {
        self.send(|reply_tx| GateMessage::HsmResume { request, reply_tx })
    }

    pub fn event_history(
        &self,
        filter: crate::EventHistoryFilter,
    ) -> Result<crate::EventHistoryResponse, ToolError> {
        self.send(|reply_tx| GateMessage::EventHistory { filter, reply_tx })
    }

    pub fn replay_lifecycle(
        &self,
        request: ReplayLifecycleRequest,
    ) -> Result<crate::ReplayLifecycleResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ReplayLifecycle { request, reply_tx })
    }

    pub fn tax_assist(
        &self,
        request: TaxAssistRequest,
    ) -> Result<crate::TaxAssistResponse, ToolError> {
        self.send(|reply_tx| GateMessage::TaxAssist { request, reply_tx })
    }

    pub fn tax_evidence_chain(
        &self,
        request: TaxEvidenceChainRequest,
    ) -> Result<crate::TaxEvidenceChainResponse, ToolError> {
        self.send(|reply_tx| GateMessage::TaxEvidenceChain { request, reply_tx })
    }

    pub fn tax_ambiguity_review(
        &self,
        request: TaxAmbiguityReviewRequest,
    ) -> Result<crate::TaxAmbiguityReviewResponse, ToolError> {
        self.send(|reply_tx| GateMessage::TaxAmbiguityReview { request, reply_tx })
    }

    pub fn validate_reconciliation_stage(
        &self,
        request: ReconciliationStageRequest,
    ) -> Result<crate::ReconciliationStageResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ValidateReconciliationStage { request, reply_tx })
    }

    pub fn reconcile_reconciliation_stage(
        &self,
        request: ReconciliationStageRequest,
    ) -> Result<crate::ReconciliationStageResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ReconcileReconciliationStage { request, reply_tx })
    }

    pub fn commit_reconciliation_stage(
        &self,
        request: ReconciliationStageRequest,
    ) -> Result<crate::ReconciliationStageResponse, ToolError> {
        self.send(|reply_tx| GateMessage::CommitReconciliationStage { request, reply_tx })
    }

    pub fn adjust_transaction(
        &self,
        request: ClassifyTransactionRequest,
    ) -> Result<crate::ClassifyTransactionResponse, ToolError> {
        self.send(|reply_tx| GateMessage::AdjustTransaction { request, reply_tx })
    }

    pub fn ontology_upsert_entities(
        &self,
        request: OntologyUpsertEntitiesRequest,
    ) -> Result<crate::OntologyUpsertEntitiesResponse, ToolError> {
        self.send(|reply_tx| GateMessage::OntologyUpsertEntities { request, reply_tx })
    }

    pub fn ontology_upsert_edges(
        &self,
        request: OntologyUpsertEdgesRequest,
    ) -> Result<crate::OntologyUpsertEdgesResponse, ToolError> {
        self.send(|reply_tx| GateMessage::OntologyUpsertEdges { request, reply_tx })
    }

    pub fn ontology_query_path(
        &self,
        request: OntologyQueryPathRequest,
    ) -> Result<crate::OntologyQueryPathResponse, ToolError> {
        self.send(|reply_tx| GateMessage::OntologyQueryPath { request, reply_tx })
    }

    pub fn ontology_export_snapshot(
        &self,
        request: OntologyExportSnapshotRequest,
    ) -> Result<crate::OntologyExportSnapshotResponse, ToolError> {
        self.send(|reply_tx| GateMessage::OntologyExportSnapshot { request, reply_tx })
    }

    pub fn ingest_image(
        &self,
        request: IngestImageRequest,
    ) -> Result<crate::IngestImageResponse, ToolError> {
        self.send(|reply_tx| GateMessage::IngestImage { request, reply_tx })
    }

    pub fn apply_tags(
        &self,
        request: ApplyTagsRequest,
    ) -> Result<crate::ApplyTagsResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ApplyTags { request, reply_tx })
    }

    pub fn remove_tags(
        &self,
        request: ApplyTagsRequest,
    ) -> Result<crate::ApplyTagsResponse, ToolError> {
        self.send(|reply_tx| GateMessage::RemoveTags { request, reply_tx })
    }

    pub fn list_tagged(
        &self,
        request: ListTaggedRequest,
    ) -> Result<crate::ListTaggedResponse, ToolError> {
        self.send(|reply_tx| GateMessage::ListTagged { request, reply_tx })
    }

    pub fn sync_fs_metadata(
        &self,
        request: SyncFsMetadataRequest,
    ) -> Result<crate::SyncFsMetadataResponse, ToolError> {
        self.send(|reply_tx| GateMessage::SyncFsMetadata { request, reply_tx })
    }

    pub fn normalize_filename(
        &self,
        request: NormalizeFilenameRequest,
    ) -> Result<crate::NormalizeFilenameResponse, ToolError> {
        self.send(|reply_tx| GateMessage::NormalizeFilename { request, reply_tx })
    }

    #[cfg(feature = "xero")]
    pub fn xero_get_auth_url(&self) -> Result<String, ToolError> {
        self.send(|reply_tx| GateMessage::XeroGetAuthUrl { reply_tx })
    }

    #[cfg(feature = "xero")]
    pub fn xero_exchange_code(
        &self,
        code: String,
        state: String,
    ) -> Result<serde_json::Value, ToolError> {
        self.send(|reply_tx| GateMessage::XeroExchangeCode {
            code,
            state,
            reply_tx,
        })
    }

    #[cfg(feature = "xero")]
    pub fn xero_fetch_contacts(
        &self,
        search: Option<String>,
    ) -> Result<serde_json::Value, ToolError> {
        self.send(|reply_tx| GateMessage::XeroFetchContacts { search, reply_tx })
    }

    #[cfg(feature = "xero")]
    pub fn xero_fetch_accounts(&self) -> Result<serde_json::Value, ToolError> {
        self.send(|reply_tx| GateMessage::XeroFetchAccounts { reply_tx })
    }

    #[cfg(feature = "xero")]
    pub fn xero_fetch_bank_accounts(&self) -> Result<serde_json::Value, ToolError> {
        self.send(|reply_tx| GateMessage::XeroFetchBankAccounts { reply_tx })
    }

    #[cfg(feature = "xero")]
    pub fn xero_fetch_invoices(
        &self,
        status: Option<String>,
    ) -> Result<serde_json::Value, ToolError> {
        self.send(|reply_tx| GateMessage::XeroFetchInvoices { status, reply_tx })
    }

    #[cfg(feature = "xero")]
    pub fn xero_link_entity(
        &self,
        local_id: String,
        xero_entity_type: String,
        xero_id: String,
        display_name: String,
        ontology_path: Option<std::path::PathBuf>,
    ) -> Result<serde_json::Value, ToolError> {
        self.send(|reply_tx| GateMessage::XeroLinkEntity {
            local_id,
            xero_entity_type,
            xero_id,
            display_name,
            ontology_path,
            reply_tx,
        })
    }

    #[cfg(feature = "xero")]
    pub fn xero_sync_catalog(
        &self,
        ontology_path: std::path::PathBuf,
    ) -> Result<serde_json::Value, ToolError> {
        self.send(|reply_tx| GateMessage::XeroSyncCatalog {
            ontology_path,
            reply_tx,
        })
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(GateMessage::Shutdown);
    }
}

pub struct ServiceActor {
    service: TurboLedgerService,
    rx: Receiver<GateMessage>,
}

impl ServiceActor {
    pub fn new(service: TurboLedgerService, rx: Receiver<GateMessage>) -> Self {
        Self { service, rx }
    }

    pub fn run(&mut self) {
        while let Ok(msg) = self.rx.recv() {
            // Check for shutdown before entering the panic boundary.
            if matches!(msg, GateMessage::Shutdown) {
                break;
            }
            // Catch panics so a single faulty request doesn't kill the entire actor.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.dispatch(msg)
            }));
            if let Err(panic) = result {
                let info = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                tracing::error!(panic = %info, "actor panic caught, continuing");
            }
        }
    }

    fn dispatch(&mut self, msg: GateMessage) {
        match msg {
                GateMessage::Shutdown => { /* handled in run() */ }
                GateMessage::ListAccounts { reply_tx } => {
                    let _ = reply_tx.send(self.service.list_accounts());
                }
                GateMessage::ListAccountsTool { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.list_accounts_tool(request));
                }
                GateMessage::DocumentInventory { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.document_inventory(request));
                }
                GateMessage::ValidateFilename {
                    file_name,
                    reply_tx,
                } => {
                    let _ = reply_tx.send(self.service.validate_source_filename(&file_name));
                }
                GateMessage::IngestStatementRows { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.ingest_statement_rows(request));
                }
                GateMessage::IngestPdf { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.ingest_pdf(request));
                }
                GateMessage::GetRawContext { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.get_raw_context(request));
                }
                GateMessage::RunRhaiRule { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.run_rhai_rule(request));
                }
                GateMessage::ClassifyIngested { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.classify_ingested(request));
                }
                GateMessage::QueryFlags { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.query_flags(request));
                }
                GateMessage::ClassifyTransaction { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.classify_transaction(request));
                }
                GateMessage::ReconcileExcelClassification { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.reconcile_excel_classification(request));
                }
                GateMessage::QueryAuditLog { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.query_audit_log(request));
                }
                GateMessage::ExportCpaWorkbook { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.export_cpa_workbook(request));
                }
                GateMessage::GetScheduleSummary { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.get_schedule_summary(request));
                }
                GateMessage::HsmTransition { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.hsm_transition_tool(request));
                }
                GateMessage::HsmStatus { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.hsm_status_tool(request));
                }
                GateMessage::HsmResume { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.hsm_resume_tool(request));
                }
                GateMessage::EventHistory { filter, reply_tx } => {
                    let _ = reply_tx.send(self.service.event_history(filter));
                }
                GateMessage::ReplayLifecycle { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.replay_lifecycle(request));
                }
                GateMessage::TaxAssist { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.tax_assist_tool(request));
                }
                GateMessage::TaxEvidenceChain { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.tax_evidence_chain_tool(request));
                }
                GateMessage::TaxAmbiguityReview { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.tax_ambiguity_review_tool(request));
                }
                GateMessage::ValidateReconciliationStage { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.validate_reconciliation_stage_tool(request));
                }
                GateMessage::ReconcileReconciliationStage { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.reconcile_reconciliation_stage_tool(request));
                }
                GateMessage::CommitReconciliationStage { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.commit_reconciliation_stage_tool(request));
                }
                GateMessage::AdjustTransaction { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.adjust_transaction(request));
                }
                GateMessage::OntologyUpsertEntities { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.ontology_upsert_entities(request));
                }
                GateMessage::OntologyUpsertEdges { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.ontology_upsert_edges(request));
                }
                GateMessage::OntologyQueryPath { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.ontology_query_path(request));
                }
                GateMessage::OntologyExportSnapshot { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.ontology_export_snapshot(request));
                }
                GateMessage::IngestImage { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.ingest_image_tool(request));
                }
                GateMessage::ApplyTags { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.apply_tags_tool(request));
                }
                GateMessage::RemoveTags { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.remove_tags_tool(request));
                }
                GateMessage::ListTagged { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.list_tagged_tool(request));
                }
                GateMessage::SyncFsMetadata { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.sync_fs_metadata_tool(request));
                }
                GateMessage::NormalizeFilename { request, reply_tx } => {
                    let _ = reply_tx.send(self.service.normalize_filename_tool(request));
                }
                #[cfg(feature = "xero")]
                GateMessage::XeroGetAuthUrl { reply_tx } => {
                    let _ = reply_tx.send(self.service.xero_get_auth_url());
                }
                #[cfg(feature = "xero")]
                GateMessage::XeroExchangeCode {
                    code,
                    state,
                    reply_tx,
                } => {
                    let _ = reply_tx.send(self.service.xero_exchange_code(code, state));
                }
                #[cfg(feature = "xero")]
                GateMessage::XeroFetchContacts { search, reply_tx } => {
                    let _ =
                        reply_tx.send(self.service.xero_fetch_contacts(search.as_deref()));
                }
                #[cfg(feature = "xero")]
                GateMessage::XeroFetchAccounts { reply_tx } => {
                    let _ = reply_tx.send(self.service.xero_fetch_accounts());
                }
                #[cfg(feature = "xero")]
                GateMessage::XeroFetchBankAccounts { reply_tx } => {
                    let _ = reply_tx.send(self.service.xero_fetch_bank_accounts());
                }
                #[cfg(feature = "xero")]
                GateMessage::XeroFetchInvoices { status, reply_tx } => {
                    let _ =
                        reply_tx.send(self.service.xero_fetch_invoices(status.as_deref()));
                }
                #[cfg(feature = "xero")]
                GateMessage::XeroLinkEntity {
                    local_id,
                    xero_entity_type,
                    xero_id,
                    display_name,
                    ontology_path,
                    reply_tx,
                } => {
                    let _ = reply_tx.send(self.service.xero_link_entity(
                        local_id,
                        xero_entity_type,
                        xero_id,
                        display_name,
                        ontology_path,
                    ));
                }
                #[cfg(feature = "xero")]
                GateMessage::XeroSyncCatalog {
                    ontology_path,
                    reply_tx,
                } => {
                    let _ = reply_tx.send(self.service.xero_sync_catalog(ontology_path));
                }
            }
        }
    }

pub fn spawn_actor(service: TurboLedgerService) -> ServiceHandle {
    let (tx, rx) = crossbeam::channel::unbounded::<GateMessage>();
    let mut actor = ServiceActor::new(service, rx);
    std::thread::spawn(move || {
        actor.run();
    });
    ServiceHandle::new(tx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccountSummary, ListAccountsRequest};
    use ledger_core::manifest::Manifest;

    fn test_manifest() -> String {
        format!(
            "[session]\nworkbook_path=\"{}.xlsx\"\nactive_year=2023\n\n\
             [accounts]\nWF-BH-CHK = {{ institution = \"Wells Fargo\", type = \"checking\", currency = \"USD\" }}\n",
            std::env::temp_dir().join("actor-test").display()
        )
    }

    #[test]
    fn service_handle_list_accounts() {
        let service = TurboLedgerService::from_manifest_str(&test_manifest())
            .expect("manifest must parse");
        let handle = spawn_actor(service);
        let accounts = handle.list_accounts().expect("list_accounts must succeed");
        assert!(accounts.iter().any(|a: &AccountSummary| a.account_id == "WF-BH-CHK"));
    }

    #[test]
    fn service_handle_list_accounts_tool() {
        let service = TurboLedgerService::from_manifest_str(&test_manifest())
            .expect("manifest must parse");
        let handle = spawn_actor(service);
        let response = handle.list_accounts_tool(ListAccountsRequest)
            .expect("list_accounts_tool must succeed");
        assert!(response.accounts.iter().any(|a| a.account_id == "WF-BH-CHK"));
    }

    #[test]
    fn service_handle_validate_filename() {
        let service = TurboLedgerService::from_manifest_str(&test_manifest())
            .expect("manifest must parse");
        let handle = spawn_actor(service);
        let result = handle.validate_source_filename("WF--BH-CHK--2023-01--statement.pdf".to_string());
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.vendor, "WF");
        assert_eq!(parsed.account, "BH-CHK");
        assert_eq!(parsed.year, 2023);
    }

    #[test]
    fn actor_survives_bad_statement_filename() {
        let service = TurboLedgerService::from_manifest_str(&test_manifest())
            .expect("manifest must parse");
        let handle = spawn_actor(service);
        let result = handle.validate_source_filename("bad-filename.pdf".to_string());
        assert!(result.is_err());
        // Actor thread should still be alive for subsequent calls.
        let accounts = handle.list_accounts().expect("actor should still be responsive");
        assert!(!accounts.is_empty());
    }

    #[test]
    fn actor_handles_concurrent_calls() {
        let service = TurboLedgerService::from_manifest_str(&test_manifest())
            .expect("manifest must parse");
        let handle = spawn_actor(service);
        let handle2 = handle.clone();
        let jh1 = std::thread::spawn(move || {
            handle.list_accounts().expect("thread 1")
        });
        let jh2 = std::thread::spawn(move || {
            handle2.list_accounts().expect("thread 2")
        });
        let r1 = jh1.join().expect("thread 1 join");
        let r2 = jh2.join().expect("thread 2 join");
        assert!(r1.iter().any(|a| a.account_id == "WF-BH-CHK"));
        assert!(r2.iter().any(|a| a.account_id == "WF-BH-CHK"));
    }
}
