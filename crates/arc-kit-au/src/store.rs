//! Evidence store — persistence for the evidence graph.
//!
//! Sidecar JSON format next to the manifest workbook.
//! Supports atomic save/load with version checking.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::edge::EvidenceEdge;
use crate::graph::EvidenceGraph;
use crate::node::EvidenceNode;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported store version: {0}")]
    UnsupportedVersion(u32),
    #[error("store file not found: {0}")]
    NotFound(PathBuf),
}

/// Persisted store format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStoreData {
    pub version: u32,
    pub nodes: Vec<EvidenceNode>,
    pub edges: Vec<EvidenceEdge>,
    pub hsm_checkpoint: Option<String>,
}

/// Persistent storage for the evidence graph.
#[derive(Debug, Clone)]
pub struct EvidenceStore {
    path: PathBuf,
}

impl EvidenceStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Load the evidence graph from disk.
    pub fn load(&self) -> Result<EvidenceGraph, StoreError> {
        if !self.path.exists() {
            return Ok(EvidenceGraph::new());
        }

        let raw = std::fs::read_to_string(&self.path)?;
        let data: EvidenceStoreData = serde_json::from_str(&raw)?;

        if data.version != 1 {
            return Err(StoreError::UnsupportedVersion(data.version));
        }

        let mut graph = EvidenceGraph::new();
        for node in data.nodes {
            let _ = graph.add_node(node);
        }
        for edge in data.edges {
            let _ = graph.add_edge(edge.from, edge.to, edge.edge_type);
        }
        Ok(graph)
    }

    /// Save the evidence graph to disk atomically.
    pub fn save(&self, graph: &EvidenceGraph) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let data = EvidenceStoreData {
            version: 1,
            nodes: graph.all_nodes().to_vec(),
            edges: graph.all_edges().to_vec(),
            hsm_checkpoint: None,
        };

        let temp_path = self.path.with_extension("json.tmp");
        let json = serde_json::to_vec_pretty(&data)?;
        std::fs::write(&temp_path, json)?;

        #[cfg(windows)]
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }

        std::fs::rename(temp_path, &self.path)?;
        Ok(())
    }

    /// Delete the evidence store.
    pub fn delete(&self) -> Result<(), StoreError> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    /// Check if the store exists.
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
}

#[cfg(test)]
mod tests {
    use crate::node::Confidence;
    use super::*;
    use crate::builder::EvidenceBuilder;
    use crate::node::{Classification, ExtractedRow, SourceDoc, Transaction};
    use chrono::TimeZone;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use tempfile::tempdir;

    fn test_doc() -> SourceDoc {
        SourceDoc {
            filename: "WF--BH-CHK--2024-01--statement.pdf".to_string(),
            vendor: "WF".to_string(),
            account_id: "BH-CHK".to_string(),
            statement_date: "2024-01-31".to_string(),
            document_type: "statement".to_string(),
            content_hash: "abc123".to_string(),
            ingested_at: Utc.with_ymd_and_hms(2024, 2, 1, 10, 0, 0).unwrap(),
            raw_context_path: None,
        }
    }

    fn test_row(doc_id: crate::node::NodeId) -> ExtractedRow {
        ExtractedRow {
            account_id: "BH-CHK".to_string(),
            date: "2024-01-15".to_string(),
            amount: Decimal::new(-1234, 2),
            description: "Cafe lunch".to_string(),
            source_document: doc_id,
            extraction_confidence: Confidence::from(0.95),
        }
    }

    fn test_tx() -> Transaction {
        Transaction {
            tx_id: "tx_123".to_string(),
            account_id: "BH-CHK".to_string(),
            date: "2024-01-15".to_string(),
            amount: "-12.34".to_string(),
            description: "Cafe lunch".to_string(),
            source_rows: vec![],
        }
    }

    fn test_cls(tx_id: String) -> Classification {
        Classification {
            tx_id,
            category: "Meals".to_string(),
            sub_category: None,
            confidence: Confidence::from(0.92),
            rule_used: Some("default_rule".to_string()),
            actor: "operator".to_string(),
            classified_at: Utc.with_ymd_and_hms(2024, 2, 1, 11, 0, 0).unwrap(),
            note: None,
        }
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("evidence.json");
        let store = EvidenceStore::new(path.clone());

        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);

        let doc = test_doc();
        let doc_id = doc.node_id();
        let rows = vec![test_row(doc_id.clone())];
        let tx = test_tx();
        let cls = test_cls(tx.tx_id.clone());

        builder.build_full_chain(doc, rows, tx, cls).unwrap();

        store.save(&graph).unwrap();
        assert!(store.exists());

        let loaded = store.load().unwrap();
        assert_eq!(loaded.node_count(), graph.node_count());
        assert_eq!(loaded.edge_count(), graph.edge_count());
    }

    #[test]
    fn load_returns_empty_graph_if_file_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nonexistent.json");
        let store = EvidenceStore::new(path);

        let graph = store.load().unwrap();
        assert!(graph.is_empty());
    }

    #[test]
    fn delete_removes_store_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("evidence.json");
        let store = EvidenceStore::new(path.clone());

        let graph = EvidenceGraph::new();
        store.save(&graph).unwrap();
        assert!(store.exists());

        store.delete().unwrap();
        assert!(!store.exists());
    }

    #[test]
    fn unsupported_version_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("evidence.json");

        let data = EvidenceStoreData {
            version: 99,
            nodes: vec![],
            edges: vec![],
            hsm_checkpoint: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        std::fs::write(&path, json).unwrap();

        let store = EvidenceStore::new(path);
        let result = store.load();
        assert!(matches!(result, Err(StoreError::UnsupportedVersion(99))));
    }
}
