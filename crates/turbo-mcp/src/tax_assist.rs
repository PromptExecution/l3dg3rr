use std::path::PathBuf;

use crate::{
    EventHistoryResponse, OntologyQueryPathResponse, ReconciliationStageRequest,
    ReconciliationStageResponse, ReplayLifecycleResponse,
};

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

pub fn build_tax_assist_response(
    source_entity_id: &str,
    stage: ReconciliationStageResponse,
    path: Option<OntologyQueryPathResponse>,
) -> TaxAssistResponse {
    if stage.status != "passed" {
        return TaxAssistResponse {
            status: "blocked".to_string(),
            stage_marker: stage.stage_marker,
            blocked_reasons: stage.blocked_reasons,
            summary: TaxAssistSummary {
                source_entity_id: source_entity_id.to_string(),
                schedule_row_count: 0,
                fbar_row_count: 0,
                ambiguity_count: 0,
            },
            schedule_rows: Vec::new(),
            fbar_rows: Vec::new(),
            ambiguity: Vec::new(),
        };
    }

    let path = path.unwrap_or(OntologyQueryPathResponse {
        nodes: Vec::new(),
        edges: Vec::new(),
    });
    let nodes = path
        .nodes
        .into_iter()
        .map(|node| {
            let id = node.id.clone();
            (id, node)
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut edges = path.edges;
    edges.sort_by(|a, b| {
        (&a.relation, &a.from, &a.to, &a.id).cmp(&(&b.relation, &b.from, &b.to, &b.id))
    });

    let mut schedule_rows = Vec::new();
    let mut fbar_rows = Vec::new();
    let mut ambiguity = Vec::new();

    for edge in edges {
        let amount = nodes
            .get(&edge.from)
            .and_then(|node| node.attrs.get("amount"))
            .cloned()
            .unwrap_or_else(|| "0".to_string());
        let provenance_refs = provenance_refs(&edge.provenance);
        if edge.relation.starts_with("schedule_") {
            schedule_rows.push(TaxEvidenceRow {
                section: "schedule".to_string(),
                entity_id: edge.to,
                relation: edge.relation,
                amount,
                provenance_refs,
            });
            continue;
        }
        if edge.relation.contains("fbar") {
            fbar_rows.push(TaxEvidenceRow {
                section: "fbar".to_string(),
                entity_id: edge.to,
                relation: edge.relation,
                amount,
                provenance_refs,
            });
            continue;
        }
        if edge.relation == "ambiguity" {
            ambiguity.push(TaxAmbiguityRecord {
                tx_id: Some(edge.from),
                review_state: "needs_review".to_string(),
                reason: "ambiguous_tax_treatment".to_string(),
                provenance_refs,
            });
        }
    }

    TaxAssistResponse {
        status: "ready".to_string(),
        stage_marker: stage.stage_marker,
        blocked_reasons: Vec::new(),
        summary: TaxAssistSummary {
            source_entity_id: source_entity_id.to_string(),
            schedule_row_count: schedule_rows.len(),
            fbar_row_count: fbar_rows.len(),
            ambiguity_count: ambiguity.len(),
        },
        schedule_rows,
        fbar_rows,
        ambiguity,
    }
}

pub fn build_tax_ambiguity_review_response(
    stage: ReconciliationStageResponse,
    ambiguity: Vec<TaxAmbiguityRecord>,
) -> TaxAmbiguityReviewResponse {
    if stage.status == "passed" {
        TaxAmbiguityReviewResponse {
            status: "review_ready".to_string(),
            stage_marker: stage.stage_marker,
            blocked_reasons: Vec::new(),
            ambiguity,
        }
    } else {
        TaxAmbiguityReviewResponse {
            status: "blocked".to_string(),
            stage_marker: stage.stage_marker,
            blocked_reasons: stage.blocked_reasons,
            ambiguity: Vec::new(),
        }
    }
}

pub fn build_tax_evidence_chain_response(
    source: TaxEvidenceSource,
    event_history: EventHistoryResponse,
    replay: ReplayLifecycleResponse,
    ambiguity: Vec<TaxAmbiguityRecord>,
) -> TaxEvidenceChainResponse {
    let mut events = event_history
        .events
        .into_iter()
        .map(|event| TaxEvidenceEvent {
            event_id: event.event_id,
            sequence: event.sequence,
            event_type: event.event_type,
            tx_id: event.tx_id,
            document_ref: event.document_ref,
        })
        .collect::<Vec<_>>();
    events.sort_by(|a, b| {
        a.sequence
            .cmp(&b.sequence)
            .then_with(|| a.event_id.cmp(&b.event_id))
    });

    TaxEvidenceChainResponse {
        source,
        events,
        current_state: TaxEvidenceCurrentState {
            reconstructed_state: replay.reconstructed_state,
            event_count: replay.event_count,
            diagnostics: replay.diagnostics,
        },
        ambiguity,
    }
}

fn provenance_refs(provenance: &std::collections::BTreeMap<String, String>) -> Vec<String> {
    let mut refs = provenance
        .iter()
        .filter_map(|(key, value)| {
            if key.contains("source") || key.contains("ref") {
                Some(value.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    refs.sort();
    refs.dedup();
    refs
}
