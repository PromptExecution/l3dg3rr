use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::ToolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OntologyEntityKind {
    Document,
    Account,
    Institution,
    Transaction,
    TaxCategory,
    EvidenceReference,
}

impl OntologyEntityKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Account => "account",
            Self::Institution => "institution",
            Self::Transaction => "transaction",
            Self::TaxCategory => "tax_category",
            Self::EvidenceReference => "evidence_reference",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyEntityInput {
    pub kind: OntologyEntityKind,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyEdgeInput {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyEntity {
    pub id: String,
    pub kind: OntologyEntityKind,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub relation: String,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct OntologyStore {
    pub entities: Vec<OntologyEntity>,
    pub edges: Vec<OntologyEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyUpsertEntitiesRequest {
    pub ontology_path: std::path::PathBuf,
    pub entities: Vec<OntologyEntityInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyUpsertEntitiesResponse {
    pub inserted_count: usize,
    pub entity_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyUpsertEdgesRequest {
    pub ontology_path: std::path::PathBuf,
    pub edges: Vec<OntologyEdgeInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyUpsertEdgesResponse {
    pub inserted_count: usize,
    pub edge_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyQueryPathRequest {
    pub ontology_path: std::path::PathBuf,
    pub from_entity_id: String,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyQueryPathResponse {
    pub nodes: Vec<OntologyEntity>,
    pub edges: Vec<OntologyEdge>,
}

impl OntologyStore {
    pub fn load(path: &Path) -> Result<Self, ToolError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(path).map_err(|e| ToolError::Internal(e.to_string()))?;
        let mut store: Self =
            serde_json::from_str(&raw).map_err(|e| ToolError::Internal(e.to_string()))?;
        store.sort_deterministic();
        Ok(store)
    }

    pub fn persist(&mut self, path: &Path) -> Result<(), ToolError> {
        self.sort_deterministic();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ToolError::Internal(e.to_string()))?;
        }
        let payload =
            serde_json::to_string_pretty(self).map_err(|e| ToolError::Internal(e.to_string()))?;
        std::fs::write(path, payload).map_err(|e| ToolError::Internal(e.to_string()))
    }

    pub fn upsert_entities(
        &mut self,
        entities: Vec<OntologyEntityInput>,
    ) -> Result<OntologyUpsertEntitiesResponse, ToolError> {
        let mut inserted_count = 0usize;
        let mut entity_ids = Vec::with_capacity(entities.len());

        for input in entities {
            let id = entity_content_hash(input.kind, &input.attrs);
            entity_ids.push(id.clone());
            if self.entities.iter().any(|existing| existing.id == id) {
                continue;
            }

            self.entities.push(OntologyEntity {
                id,
                kind: input.kind,
                attrs: input.attrs,
            });
            inserted_count += 1;
        }

        self.sort_deterministic();

        Ok(OntologyUpsertEntitiesResponse {
            inserted_count,
            entity_ids,
        })
    }

    pub fn upsert_edges(
        &mut self,
        edges: Vec<OntologyEdgeInput>,
    ) -> Result<OntologyUpsertEdgesResponse, ToolError> {
        let entity_ids = self
            .entities
            .iter()
            .map(|entity| entity.id.clone())
            .collect::<BTreeSet<_>>();

        let mut inserted_count = 0usize;
        let mut edge_ids = Vec::with_capacity(edges.len());

        for input in edges {
            if !entity_ids.contains(&input.from) || !entity_ids.contains(&input.to) {
                return Err(ToolError::InvalidInput(
                    "missing_ref: edge endpoints must reference existing entities".to_string(),
                ));
            }

            let id = edge_content_hash(&input.from, &input.to, &input.relation, &input.provenance);
            edge_ids.push(id.clone());

            if self.edges.iter().any(|existing| existing.id == id) {
                continue;
            }

            self.edges.push(OntologyEdge {
                id,
                from: input.from,
                to: input.to,
                relation: input.relation,
                provenance: input.provenance,
            });
            inserted_count += 1;
        }

        self.sort_deterministic();

        Ok(OntologyUpsertEdgesResponse {
            inserted_count,
            edge_ids,
        })
    }

    pub fn query_path(
        &self,
        from_entity_id: &str,
        max_depth: Option<usize>,
    ) -> Result<OntologyQueryPathResponse, ToolError> {
        let entity_lookup = self
            .entities
            .iter()
            .cloned()
            .map(|entity| (entity.id.clone(), entity))
            .collect::<BTreeMap<_, _>>();

        let start = entity_lookup.get(from_entity_id).cloned().ok_or_else(|| {
            ToolError::InvalidInput("missing_ref: from_entity_id must reference an existing entity".to_string())
        })?;

        let depth_limit = max_depth.unwrap_or(usize::MAX);
        let mut queue = VecDeque::new();
        queue.push_back((from_entity_id.to_string(), 0usize));

        let mut visited = BTreeSet::new();
        visited.insert(from_entity_id.to_string());

        let mut nodes = vec![start];
        let mut edges = Vec::new();

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= depth_limit {
                continue;
            }

            let mut outgoing = self
                .edges
                .iter()
                .filter(|edge| edge.from == current_id)
                .cloned()
                .collect::<Vec<_>>();
            outgoing.sort_by(|a, b| {
                (&a.relation, &a.to, &a.id).cmp(&(&b.relation, &b.to, &b.id))
            });

            for edge in outgoing {
                if visited.contains(&edge.to) {
                    continue;
                }

                if let Some(node) = entity_lookup.get(&edge.to) {
                    visited.insert(edge.to.clone());
                    queue.push_back((edge.to.clone(), depth + 1));
                    nodes.push(node.clone());
                    edges.push(edge);
                }
            }
        }

        Ok(OntologyQueryPathResponse { nodes, edges })
    }

    fn sort_deterministic(&mut self) {
        self.entities
            .sort_by(|a, b| (a.kind, &a.id).cmp(&(b.kind, &b.id)));
        self.edges.sort_by(|a, b| {
            (&a.relation, &a.from, &a.to, &a.id).cmp(&(&b.relation, &b.from, &b.to, &b.id))
        });
    }
}

pub fn entity_content_hash(kind: OntologyEntityKind, attrs: &BTreeMap<String, String>) -> String {
    let mut canonical = format!("entity|{}", kind.as_str());
    for (key, value) in attrs {
        canonical.push('|');
        canonical.push_str(key);
        canonical.push('=');
        canonical.push_str(value);
    }
    content_hash(&canonical)
}

pub fn edge_content_hash(
    from: &str,
    to: &str,
    relation: &str,
    provenance: &BTreeMap<String, String>,
) -> String {
    let mut canonical = format!("edge|{}|{}|{}", from, to, relation);
    for (key, value) in provenance {
        canonical.push('|');
        canonical.push_str(key);
        canonical.push('=');
        canonical.push_str(value);
    }
    content_hash(&canonical)
}

pub fn content_hash(canonical: &str) -> String {
    blake3::hash(canonical.as_bytes()).to_hex().to_string()
}
