//! Evidence chain tracing — full provenance from any node.
//!
//! Traces the complete evidence chain for a transaction,
//! from source document through to workbook export.

use crate::edge::EdgeType;
use crate::graph::EvidenceGraph;
use crate::node::{EvidenceNode, NodeId, NodeType};

/// Complete evidence chain for a transaction.
#[derive(Debug, Clone)]
pub struct EvidenceChain {
    pub tx_id: String,
    pub source_documents: Vec<EvidenceNode>,
    pub extracted_rows: Vec<EvidenceNode>,
    pub classifications: Vec<EvidenceNode>,
    pub proposals: Vec<EvidenceNode>,
    pub approvals: Vec<EvidenceNode>,
    pub workbook_rows: Vec<EvidenceNode>,
}

impl EvidenceChain {
    /// Check if the chain has complete provenance.
    ///
    /// Complete = has source document, extracted rows, classification, and approval.
    pub fn has_complete_provenance(&self) -> bool {
        !self.source_documents.is_empty()
            && !self.extracted_rows.is_empty()
            && !self.classifications.is_empty()
            && !self.approvals.is_empty()
    }

    /// Get the count of source documents.
    pub fn source_count(&self) -> usize {
        self.source_documents.len()
    }

    /// Get the count of extracted rows.
    pub fn row_count(&self) -> usize {
        self.extracted_rows.len()
    }

    /// Get the count of model proposals.
    pub fn proposal_count(&self) -> usize {
        self.proposals.len()
    }

    /// Get the count of operator approvals.
    pub fn approval_count(&self) -> usize {
        self.approvals.len()
    }

    /// Get the count of workbook exports.
    pub fn export_count(&self) -> usize {
        self.workbook_rows.len()
    }

    /// Check if chain is missing source documents.
    pub fn missing_source(&self) -> bool {
        self.source_documents.is_empty()
    }

    /// Check if chain is missing extracted rows.
    pub fn missing_rows(&self) -> bool {
        self.extracted_rows.is_empty()
    }

    /// Check if chain is missing classification.
    pub fn missing_classification(&self) -> bool {
        self.classifications.is_empty()
    }

    /// Check if chain is missing operator approval.
    pub fn missing_approval(&self) -> bool {
        self.approvals.is_empty()
    }

    /// Check if chain is missing workbook export.
    pub fn missing_export(&self) -> bool {
        self.workbook_rows.is_empty()
    }

    /// Get missing elements as a list of descriptions.
    pub fn missing_elements(&self) -> Vec<&str> {
        let mut missing = Vec::new();
        if self.missing_source() {
            missing.push("source_document");
        }
        if self.missing_rows() {
            missing.push("extracted_rows");
        }
        if self.missing_classification() {
            missing.push("classification");
        }
        if self.missing_approval() {
            missing.push("operator_approval");
        }
        if self.missing_export() {
            missing.push("workbook_export");
        }
        missing
    }
}

/// Trace evidence from a transaction ID.
pub trait EvidenceTracer {
    fn trace_transaction(&self, tx_id: &str) -> Option<EvidenceChain>;
}

impl EvidenceTracer for EvidenceGraph {
    fn trace_transaction(&self, tx_id: &str) -> Option<EvidenceChain> {
        let tx_node_id = NodeId::new(NodeType::Transaction, tx_id);
        let _tx_node = self.get_node(&tx_node_id)?;

        let mut chain = EvidenceChain {
            tx_id: tx_id.to_string(),
            source_documents: Vec::new(),
            extracted_rows: Vec::new(),
            classifications: Vec::new(),
            proposals: Vec::new(),
            approvals: Vec::new(),
            workbook_rows: Vec::new(),
        };

        // Trace backwards: find incoming edges (source rows)
        for edge in self.incoming_edges(&tx_node_id) {
            if edge.edge_type == EdgeType::Produces {
                if let Some(row_node) = self.get_node(&edge.from) {
                    chain.extracted_rows.push(row_node.clone());
                    // Trace further back to source documents
                    for row_edge in self.incoming_edges(&edge.from) {
                        if row_edge.edge_type == EdgeType::ExtractedFrom {
                            if let Some(doc_node) = self.get_node(&row_edge.from) {
                                chain.source_documents.push(doc_node.clone());
                            }
                        }
                    }
                }
            }
        }

        // Trace forwards: find outgoing edges (classifications, proposals, approvals, exports)
        for edge in self.outgoing_edges(&tx_node_id) {
            if let Some(target_node) = self.get_node(&edge.to) {
                match edge.edge_type {
                    EdgeType::ClassifiedAs => chain.classifications.push(target_node.clone()),
                    EdgeType::ProposedBy => chain.proposals.push(target_node.clone()),
                    EdgeType::ApprovedBy => chain.approvals.push(target_node.clone()),
                    EdgeType::ExportedTo => chain.workbook_rows.push(target_node.clone()),
                    _ => {}
                }
            }
        }

        Some(chain)
    }
}

#[cfg(test)]
mod tests {
    use crate::node::Confidence;
    use super::*;
    use crate::builder::EvidenceBuilder;
    use crate::node::{Classification, ExtractedRow, OperatorApproval, SourceDoc, Transaction};
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
    fn trace_complete_chain() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);

        let doc = test_doc();
        let doc_id = doc.node_id();
        let rows = vec![test_row(doc_id.clone())];
        let tx = test_tx();
        let cls = test_cls(tx.tx_id.clone());

        builder.build_full_chain(doc, rows, tx, cls);

        let chain = graph.trace_transaction("tx_123");
        assert!(chain.is_some());
        let chain = chain.unwrap();

        assert_eq!(chain.source_count(), 1);
        assert_eq!(chain.row_count(), 1);
        assert!(!chain.missing_source());
        assert!(!chain.missing_rows());
        assert!(!chain.missing_classification());
        assert!(chain.missing_approval()); // Not added in build_full_chain
    }

    #[test]
    fn trace_missing_elements() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);

        let doc = test_doc();
        let doc_id = doc.node_id();
        let rows = vec![test_row(doc_id.clone())];
        let tx = test_tx();

        // Only build partial chain
        let doc_id = builder.ensure_document(doc);
        let row_ids = builder.ensure_extracted_rows(&doc_id, rows);
        let tx_id = builder.ensure_transaction(tx, &row_ids);

        let chain = graph.trace_transaction(tx_id.hash()).unwrap();
        let missing = chain.missing_elements();

        // Should be missing classification, approval, and export
        assert!(missing.contains(&"classification"));
        assert!(missing.contains(&"operator_approval"));
        assert!(missing.contains(&"workbook_export"));
        // But should have source and rows
        assert!(!missing.contains(&"source_document"));
        assert!(!missing.contains(&"extracted_rows"));
    }

    #[test]
    fn trace_returns_none_for_missing_transaction() {
        let graph = EvidenceGraph::new();
        let chain = graph.trace_transaction("nonexistent_tx");
        assert!(chain.is_none());
    }

    #[test]
    fn has_complete_provenance_requires_all_elements() {
        let complete = EvidenceChain {
            tx_id: "tx_1".to_string(),
            source_documents: vec![EvidenceNode::SourceDoc(test_doc())],
            extracted_rows: vec![EvidenceNode::ExtractedRow(test_row(NodeId::new(
                NodeType::SourceDoc,
                "abc",
            )))],
            classifications: vec![EvidenceNode::Classification(test_cls("tx_1".to_string()))],
            proposals: vec![],
            approvals: vec![EvidenceNode::OperatorApproval(OperatorApproval {
                tx_id: "tx_1".to_string(),
                operator_id: "user1".to_string(),
                approved: true,
                rationale: None,
                approved_at: Utc.with_ymd_and_hms(2024, 2, 1, 11, 0, 0).unwrap(),
            })],
            workbook_rows: vec![],
        };
        assert!(complete.has_complete_provenance());

        let incomplete = EvidenceChain {
            tx_id: "tx_2".to_string(),
            source_documents: vec![],
            extracted_rows: vec![],
            classifications: vec![],
            proposals: vec![],
            approvals: vec![],
            workbook_rows: vec![],
        };
        assert!(!incomplete.has_complete_provenance());
    }
}
