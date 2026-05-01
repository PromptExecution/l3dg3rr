//! Missing provenance detection — find gaps in the evidence graph.
//!
//! Identifies transactions that lack required evidence elements
//! for audit compliance.

use crate::edge::EdgeType;
use crate::graph::EvidenceGraph;
use crate::node::NodeType;

/// Element that can be missing from provenance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingElement {
    SourceDocument,
    ExtractedRows,
    Classification,
    OperatorApproval,
    WorkbookExport,
}

impl std::fmt::Display for MissingElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SourceDocument => write!(f, "source_document"),
            Self::ExtractedRows => write!(f, "extracted_rows"),
            Self::Classification => write!(f, "classification"),
            Self::OperatorApproval => write!(f, "operator_approval"),
            Self::WorkbookExport => write!(f, "workbook_export"),
        }
    }
}

/// A gap in transaction provenance.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProvenanceGap {
    pub tx_id: String,
    pub missing: Vec<MissingElement>,
    pub has_source: bool,
    pub has_classification: bool,
    pub has_approval: bool,
    pub has_export: bool,
}

impl ProvenanceGap {
    pub fn is_critical(&self) -> bool {
        self.missing.contains(&MissingElement::SourceDocument)
            || self.missing.contains(&MissingElement::Classification)
    }
}

/// Find all transactions with missing provenance.
pub trait ProvenanceScanner {
    fn find_missing_provenance(&self) -> Vec<ProvenanceGap>;
}

impl ProvenanceScanner for EvidenceGraph {
    fn find_missing_provenance(&self) -> Vec<ProvenanceGap> {
        let mut gaps = Vec::new();

        for tx_node in self.nodes_of_type(NodeType::Transaction) {
            let tx_id = match tx_node.tx_id() {
                Some(id) => id.to_string(),
                None => continue,
            };
            let tx_node_id = tx_node.node_id();

            let mut has_source = false;
            let mut has_classification = false;
            let mut has_approval = false;
            let mut has_export = false;
            let mut missing = Vec::new();

            // Check incoming edges for source rows
            let incoming = self.incoming_edges(&tx_node_id);
            let has_rows = incoming
                .iter()
                .any(|e| e.edge_type == EdgeType::Produces);

            if has_rows {
                // Check if rows have source documents
                for edge in &incoming {
                    if edge.edge_type == EdgeType::Produces {
                        let row_incoming = self.incoming_edges(&edge.from);
                        if row_incoming
                            .iter()
                            .any(|e| e.edge_type == EdgeType::ExtractedFrom)
                        {
                            has_source = true;
                            break;
                        }
                    }
                }
            }

            // Check outgoing edges for classifications, approvals, exports
            for edge in self.outgoing_edges(&tx_node_id) {
                match edge.edge_type {
                    EdgeType::ClassifiedAs => has_classification = true,
                    EdgeType::ApprovedBy => has_approval = true,
                    EdgeType::ExportedTo => has_export = true,
                    _ => {}
                }
            }

            // Build missing list
            if !has_source {
                missing.push(MissingElement::SourceDocument);
            }
            if !has_rows {
                missing.push(MissingElement::ExtractedRows);
            }
            if !has_classification {
                missing.push(MissingElement::Classification);
            }
            if !has_approval {
                missing.push(MissingElement::OperatorApproval);
            }
            if !has_export {
                missing.push(MissingElement::WorkbookExport);
            }

            if !missing.is_empty() {
                gaps.push(ProvenanceGap {
                    tx_id,
                    missing,
                    has_source,
                    has_classification,
                    has_approval,
                    has_export,
                });
            }
        }

        gaps
    }
}

#[cfg(test)]
mod tests {
    use crate::node::Confidence;
    use super::*;
    use crate::builder::EvidenceBuilder;
    use crate::node::{Classification, ExtractedRow, NodeId, SourceDoc, Transaction};
    use chrono::TimeZone;
    use chrono::Utc;
    use rust_decimal::Decimal;

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

    fn test_row(doc_id: NodeId) -> ExtractedRow {
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
    fn complete_chain_has_no_gaps() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);

        let doc = test_doc();
        let doc_id = doc.node_id();
        let rows = vec![test_row(doc_id.clone())];
        let tx = test_tx();
        let cls = test_cls(tx.tx_id.clone());

        builder.build_full_chain(doc, rows, tx, cls).unwrap();

        // build_full_chain doesn't create approvals or exports, so those will be gaps
        let gaps = graph.find_missing_provenance();
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0]
            .missing
            .contains(&MissingElement::OperatorApproval));
        assert!(gaps[0].missing.contains(&MissingElement::WorkbookExport));
        // But source and classification should be present
        assert!(!gaps[0].missing.contains(&MissingElement::SourceDocument));
        assert!(!gaps[0].missing.contains(&MissingElement::Classification));
    }

    #[test]
    fn partial_chain_has_gaps() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);

        let doc = test_doc();
        let doc_id = doc.node_id();
        let rows = vec![test_row(doc_id.clone())];
        let tx = test_tx();

        // Only build document + rows + transaction
        let doc_id = builder.ingest_document(doc).unwrap();
        let row_ids = builder.extract_rows(&doc_id, rows).unwrap();
        builder.create_transaction(tx, &row_ids).unwrap();

        let gaps = graph.find_missing_provenance();
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0]
            .missing
            .contains(&MissingElement::Classification));
        assert!(gaps[0]
            .missing
            .contains(&MissingElement::OperatorApproval));
        assert!(gaps[0].missing.contains(&MissingElement::WorkbookExport));
    }

    #[test]
    fn critical_gap_includes_source_or_classification() {
        let gap = ProvenanceGap {
            tx_id: "tx_1".to_string(),
            missing: vec![
                MissingElement::SourceDocument,
                MissingElement::OperatorApproval,
            ],
            has_source: false,
            has_classification: true,
            has_approval: false,
            has_export: true,
        };
        assert!(gap.is_critical());

        let non_critical = ProvenanceGap {
            tx_id: "tx_2".to_string(),
            missing: vec![MissingElement::OperatorApproval],
            has_source: true,
            has_classification: true,
            has_approval: false,
            has_export: true,
        };
        assert!(!non_critical.is_critical());
    }

    #[test]
    fn missing_element_display_format() {
        assert_eq!(MissingElement::SourceDocument.to_string(), "source_document");
        assert_eq!(
            MissingElement::OperatorApproval.to_string(),
            "operator_approval"
        );
        assert_eq!(MissingElement::WorkbookExport.to_string(), "workbook_export");
    }
}
