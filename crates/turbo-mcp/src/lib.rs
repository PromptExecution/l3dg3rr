use std::path::Path;

use ledger_core::filename::{FilenameError, StatementFilename};
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
}

pub struct TurboLedgerService {
    manifest: Manifest,
}

impl TurboLedgerService {
    pub fn from_manifest_str(src: &str) -> Result<Self, ToolError> {
        let manifest = Manifest::parse(src).map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        Ok(Self { manifest })
    }

    pub fn workbook_path(&self) -> &Path {
        Path::new(&self.manifest.session.workbook_path)
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
}
