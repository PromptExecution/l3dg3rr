use std::path::PathBuf;
use std::sync::Mutex;

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
    pub rows: Vec<TransactionInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestStatementRowsResponse {
    pub inserted_count: usize,
    pub tx_ids: Vec<String>,
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
}

pub struct TurboLedgerService {
    manifest: Manifest,
    ingest_state: Mutex<IngestedLedger>,
}

impl TurboLedgerService {
    pub fn from_manifest_str(src: &str) -> Result<Self, ToolError> {
        let manifest = Manifest::parse(src).map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        Ok(Self {
            manifest,
            ingest_state: Mutex::new(IngestedLedger::default()),
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
        let mut state = self
            .ingest_state
            .lock()
            .map_err(|_| ToolError::Internal("ingest lock poisoned".to_string()))?;
        let inserted = state
            .ingest_to_journal(&request.rows, &request.journal_path)
            .map_err(|e| ToolError::Internal(e.to_string()))?;
        let tx_ids = inserted.iter().map(|row| row.tx_id.clone()).collect::<Vec<_>>();
        Ok(IngestStatementRowsResponse {
            inserted_count: tx_ids.len(),
            tx_ids,
        })
    }
}
