#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationStageRequest {
    pub source_total: String,
    pub extracted_total: String,
    pub posting_amounts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationDiagnostic {
    pub key: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconciliationStageResponse {
    pub stage: String,
    pub status: String,
    pub blocked_reasons: Vec<String>,
    pub stage_marker: String,
    pub diagnostics: Vec<ReconciliationDiagnostic>,
}
