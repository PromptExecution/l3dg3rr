//! Evidence builder — incremental graph construction from events.
//!
//! Provides a fluent API for building evidence chains from ingest,
//! classify, approve, and export operations.


use crate::edge::EdgeType;
use crate::graph::EvidenceGraph;
use crate::node::{
    Classification, EvidenceNode, ExtractedRow, ModelProposal, NodeId, NodeType, OperatorApproval,
    SourceDoc, Transaction, ValidationIssue, WorkbookRow,
};

/// Fluent builder for evidence graph construction.
/// All methods are idempotent: calling them multiple times with the same
/// data will not create duplicate nodes or edges. Failed edge insertions
/// (missing target node) are logged with tracing::warn! and do not panic.
pub struct EvidenceBuilder<'a> {
    graph: &'a mut EvidenceGraph,
}

impl<'a> EvidenceBuilder<'a> {
    pub fn new(graph: &'a mut EvidenceGraph) -> Self {
        Self { graph }
    }

    /// Idempotent: ensure a source document node in the evidence graph.
    /// Returns the node ID whether inserted or pre-existing.
    pub fn ensure_document(&mut self, doc: SourceDoc) -> NodeId {
        let node_id = doc.node_id();
        self.graph.ensure_node(EvidenceNode::SourceDoc(doc));
        node_id
    }

    /// Idempotent: ensure extracted row nodes and their ExtractedFrom edges.
    /// Rows referencing a document that does not exist in the graph log a warning.
    pub fn ensure_extracted_rows(
        &mut self,
        doc_id: &NodeId,
        rows: Vec<ExtractedRow>,
    ) -> Vec<NodeId> {
        rows.into_iter()
            .map(|row| {
                let row_id = row.node_id();
                self.graph.ensure_node(EvidenceNode::ExtractedRow(row));
                self.graph
                    .ensure_edge(doc_id.clone(), row_id.clone(), EdgeType::ExtractedFrom);
                row_id
            })
            .collect()
    }

    /// Idempotent: ensure a transaction node with Produces edges to source rows.
    pub fn ensure_transaction(&mut self, tx: Transaction, source_rows: &[NodeId]) -> NodeId {
        let tx_id = tx.node_id();
        self.graph.ensure_node(EvidenceNode::Transaction(tx));
        for row_id in source_rows {
            self.graph
                .ensure_edge(row_id.clone(), tx_id.clone(), EdgeType::Produces);
        }
        tx_id
    }

    /// Idempotent: ensure a classification node with ClassifiedAs edge.
    pub fn ensure_classification(&mut self, classification: Classification) -> NodeId {
        let cls_id = classification.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &classification.tx_id);
        self.graph
            .ensure_node(EvidenceNode::Classification(classification));
        self.graph
            .ensure_edge(tx_id, cls_id.clone(), EdgeType::ClassifiedAs);
        cls_id
    }

    /// Idempotent: ensure a model proposal node.
    pub fn ensure_proposal(&mut self, proposal: ModelProposal) -> NodeId {
        let prop_id = proposal.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &proposal.tx_id);
        self.graph.ensure_node(EvidenceNode::ModelProposal(proposal));
        self.graph
            .ensure_edge(tx_id, prop_id.clone(), EdgeType::ProposedBy);
        prop_id
    }

    /// Idempotent: ensure an operator approval node.
    pub fn ensure_approval(&mut self, approval: OperatorApproval) -> NodeId {
        let approval_id = approval.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &approval.tx_id);
        self.graph
            .ensure_node(EvidenceNode::OperatorApproval(approval));
        self.graph
            .ensure_edge(tx_id, approval_id.clone(), EdgeType::ApprovedBy);
        approval_id
    }

    /// Idempotent: ensure a workbook row node with ExportedTo edge.
    pub fn ensure_workbook_row(&mut self, wb_row: WorkbookRow) -> NodeId {
        let wb_id = wb_row.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &wb_row.tx_id);
        self.graph.ensure_node(EvidenceNode::WorkbookRow(wb_row));
        self.graph.ensure_edge(tx_id, wb_id.clone(), EdgeType::ExportedTo);
        wb_id
    }

    /// Idempotent: ensure a validation issue node with ValidatedAs edge.
    pub fn ensure_validation_issue(&mut self, issue: ValidationIssue) -> NodeId {
        let vi_id = issue.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &issue.tx_id);
        self.graph
            .ensure_node(EvidenceNode::ValidationIssue(issue));
        self.graph
            .ensure_edge(tx_id, vi_id.clone(), EdgeType::ValidatedAs);
        vi_id
    }

    /// Build a complete evidence chain for a transaction in one call.
    /// Idempotent — safe to call multiple times with the same data.
    pub fn build_full_chain(
        &mut self,
        doc: SourceDoc,
        rows: Vec<ExtractedRow>,
        tx: Transaction,
        classification: Classification,
    ) {
        let doc_id = self.ensure_document(doc);
        let row_ids = self.ensure_extracted_rows(&doc_id, rows);
        let tx_id = self.ensure_transaction(tx, &row_ids);
        let mut cls = classification;
        cls.tx_id = tx_id.hash().to_string();
        self.ensure_classification(cls);
    }
}

#[cfg(test)]
mod tests {
    use crate::node::Confidence;
    use super::*;
    use chrono::{TimeZone, Utc};
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
    fn builder_ingests_document() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ensure_document(test_doc());
        assert_eq!(graph.node_count(), 1);
        assert_eq!(doc_id.node_type(), NodeType::SourceDoc);
    }

    #[test]
    fn builder_extracts_rows_with_edges() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ensure_document(test_doc());
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.ensure_extracted_rows(builder.extract_rows(&doc_id, rows).unwrap()doc_id, rows);

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(row_ids.len(), 1);
    }

    #[test]
    fn builder_creates_transaction_from_rows() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ensure_document(test_doc());
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.ensure_extracted_rows(builder.extract_rows(&doc_id, rows).unwrap()doc_id, rows);
        let tx_id = builder.ensure_transaction(test_tx(), &row_ids);

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
        assert_eq!(tx_id.node_type(), NodeType::Transaction);
    }

    #[test]
    fn builder_classifies_transaction() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ensure_document(test_doc());
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.ensure_extracted_rows(builder.extract_rows(&doc_id, rows).unwrap()doc_id, rows);
        let tx_id = builder.ensure_transaction(test_tx(), &row_ids);

        let cls = test_cls(tx_id.hash().to_string());
        let cls_id = builder.ensure_classification(cls);

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);
        assert_eq!(cls_id.node_type(), NodeType::Classification);
    }

    #[test]
    fn builder_records_proposal_and_approval() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ensure_document(test_doc());
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.ensure_extracted_rows(builder.extract_rows(&doc_id, rows).unwrap()doc_id, rows);
        let tx_id = builder.ensure_transaction(test_tx(), &row_ids);

        let proposal = ModelProposal {
            tx_id: tx_id.hash().to_string(),
            model_name: "phi-4-mini".to_string(),
            proposed_category: "Meals".to_string(),
            confidence: Confidence::from(0.87),
            reasoning: Some("vendor name suggests food purchase".to_string()),
            proposed_at: Utc.with_ymd_and_hms(2024, 2, 1, 10, 30, 0).unwrap(),
            validated: true,
        };
        let prop_id = builder.ensure_proposal(proposal);

        let approval = OperatorApproval {
            tx_id: tx_id.hash().to_string(),
            operator_id: "user1".to_string(),
            approved: true,
            rationale: Some("correct category".to_string()),
            approved_at: Utc.with_ymd_and_hms(2024, 2, 1, 11, 0, 0).unwrap(),
        };
        let approval_id = builder.ensure_approval(approval);

        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4);
        assert_eq!(prop_id.node_type(), NodeType::ModelProposal);
        assert_eq!(approval_id.node_type(), NodeType::OperatorApproval);
    }

    #[test]
    fn builder_exports_workbook_row() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ensure_document(test_doc());
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.ensure_extracted_rows(builder.extract_rows(&doc_id, rows).unwrap()doc_id, rows);
        let tx_id = builder.ensure_transaction(test_tx(), &row_ids);

        let wb_row = WorkbookRow {
            tx_id: tx_id.hash().to_string(),
            sheet_name: "Transactions".to_string(),
            row_index: 42,
            category: "Meals".to_string(),
            amount: "-12.34".to_string(),
            exported_at: Utc.with_ymd_and_hms(2024, 2, 1, 12, 0, 0).unwrap(),
        };
        let wb_id = builder.ensure_workbook_row(wb_row);

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);
        assert_eq!(wb_id.node_type(), NodeType::WorkbookRow);
    }

    #[test]
    fn build_full_chain_creates_complete_evidence() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);

        let doc = test_doc();
        let doc_id = doc.node_id();
        let rows = vec![test_row(doc_id.clone())];
        let tx = test_tx();
        let cls = test_cls(tx.tx_id.clone());

        builder.build_full_chain(doc, rows, tx, cls).unwrap();

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);
    }
}
