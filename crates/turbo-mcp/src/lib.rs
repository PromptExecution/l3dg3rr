use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use ledger_core::classify::{ClassificationEngine, FlagStatus, SampleTransaction};
use ledger_core::filename::{FilenameError, StatementFilename};
use ledger_core::ingest::{IngestedLedger, TransactionInput};
use ledger_core::manifest::Manifest;

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
    fn get_raw_context(&self, request: GetRawContextRequest)
        -> Result<GetRawContextResponse, ToolError>;
    fn run_rhai_rule(&self, request: RunRhaiRuleRequest) -> Result<RunRhaiRuleResponse, ToolError>;
    fn classify_ingested(
        &self,
        request: ClassifyIngestedRequest,
    ) -> Result<ClassifyIngestedResponse, ToolError>;
    fn query_flags(&self, request: QueryFlagsRequest) -> Result<QueryFlagsResponse, ToolError>;
}

#[derive(Debug, Default)]
struct ClassificationState {
    tx_rows: BTreeMap<String, TransactionInput>,
    engine: ClassificationEngine,
}

pub struct TurboLedgerService {
    manifest: Manifest,
    ingest_state: Mutex<IngestedLedger>,
    classification_state: Mutex<ClassificationState>,
}

impl TurboLedgerService {
    pub fn from_manifest_str(src: &str) -> Result<Self, ToolError> {
        let manifest = Manifest::parse(src).map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        Ok(Self {
            manifest,
            ingest_state: Mutex::new(IngestedLedger::default()),
            classification_state: Mutex::new(ClassificationState::default()),
        })
    }

    pub fn workbook_path(&self) -> &std::path::Path {
        std::path::Path::new(&self.manifest.session.workbook_path)
    }

    pub fn list_accounts_tool(
        &self,
        _request: ListAccountsRequest,
    ) -> Result<ListAccountsResponse, ToolError> {
        Ok(ListAccountsResponse {
            accounts: self.list_accounts()?,
        })
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
            by_id.insert(ledger_core::ingest::deterministic_tx_id(row), row.clone());
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

        let tx_ids = inserted.iter().map(|row| row.tx_id.clone()).collect::<Vec<_>>();
        Ok(IngestStatementRowsResponse {
            inserted_count: tx_ids.len(),
            tx_ids,
        })
    }

    fn ingest_pdf(&self, request: IngestPdfRequest) -> Result<IngestPdfResponse, ToolError> {
        let file_name = std::path::Path::new(&request.pdf_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| ToolError::InvalidInput("pdf_path must have a valid filename".to_string()))?;
        let _parsed = self.validate_source_filename(file_name)?;

        for row in &request.extracted_rows {
            let source_path = std::path::Path::new(&row.source_ref);
            if source_path.exists() {
                continue;
            }
            if let Some(parent) = source_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| ToolError::Internal(e.to_string()))?;
            }
            let bytes = request
                .raw_context_bytes
                .as_deref()
                .ok_or_else(|| ToolError::InvalidInput("raw_context_bytes required when source_ref file does not exist".to_string()))?;
            std::fs::write(source_path, bytes).map_err(|e| ToolError::Internal(e.to_string()))?;
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
        let bytes = std::fs::read(&request.rkyv_ref).map_err(|e| ToolError::Internal(e.to_string()))?;
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

        Ok(ClassifyIngestedResponse {
            classifications: batch
                .classifications
                .into_iter()
                .map(|c| ClassifiedTxResponse {
                    tx_id: c.tx_id,
                    category: c.category,
                    confidence: c.confidence,
                    needs_review: c.needs_review,
                    reason: c.reason,
                })
                .collect(),
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
}
