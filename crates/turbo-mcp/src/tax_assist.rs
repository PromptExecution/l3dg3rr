use std::path::PathBuf;

use crate::ReconciliationStageRequest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxAssistRequest {
    pub ontology_path: PathBuf,
    pub from_entity_id: String,
    pub max_depth: Option<usize>,
    pub reconciliation: ReconciliationStageRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxAssistSummary {
    pub source_entity_id: String,
    pub schedule_row_count: usize,
    pub fbar_row_count: usize,
    pub ambiguity_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxEvidenceRow {
    pub section: String,
    pub entity_id: String,
    pub relation: String,
    pub amount: String,
    pub provenance_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxAmbiguityRecord {
    pub tx_id: Option<String>,
    pub review_state: String,
    pub reason: String,
    pub provenance_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxAssistResponse {
    pub status: String,
    pub stage_marker: String,
    pub blocked_reasons: Vec<String>,
    pub summary: TaxAssistSummary,
    pub schedule_rows: Vec<TaxEvidenceRow>,
    pub fbar_rows: Vec<TaxEvidenceRow>,
    pub ambiguity: Vec<TaxAmbiguityRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxEvidenceChainRequest {
    pub ontology_path: PathBuf,
    pub from_entity_id: String,
    pub tx_id: Option<String>,
    pub document_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxEvidenceSource {
    pub from_entity_id: String,
    pub node_ids: Vec<String>,
    pub edge_ids: Vec<String>,
    pub provenance_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxEvidenceEvent {
    pub event_id: String,
    pub sequence: u64,
    pub event_type: String,
    pub tx_id: Option<String>,
    pub document_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxEvidenceCurrentState {
    pub reconstructed_state: String,
    pub event_count: usize,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxEvidenceChainResponse {
    pub source: TaxEvidenceSource,
    pub events: Vec<TaxEvidenceEvent>,
    pub current_state: TaxEvidenceCurrentState,
    pub ambiguity: Vec<TaxAmbiguityRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxAmbiguityReviewRequest {
    pub ontology_path: PathBuf,
    pub from_entity_id: String,
    pub max_depth: Option<usize>,
    pub reconciliation: ReconciliationStageRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaxAmbiguityReviewResponse {
    pub status: String,
    pub stage_marker: String,
    pub blocked_reasons: Vec<String>,
    pub ambiguity: Vec<TaxAmbiguityRecord>,
}
