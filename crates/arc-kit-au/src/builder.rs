//! Evidence builder — incremental graph construction from events.
//!
//! Provides a fluent API for building evidence chains from ingest,
//! classify, approve, and export operations.


use crate::edge::EdgeType;
use crate::graph::{EvidenceGraph, GraphError};
use crate::node::{
    Classification, EvidenceNode, ExtractedRow, ModelProposal, NodeId, NodeType, OperatorApproval,
    SourceDoc, Transaction, WorkbookRow,
};

/// Fluent builder for evidence graph construction.
pub struct EvidenceBuilder<'a> {
    graph: &'a mut EvidenceGraph,
}

impl<'a> EvidenceBuilder<'a> {
    pub fn new(graph: &'a mut EvidenceGraph) -> Self {
        Self { graph }
    }

    /// Ingest a source document into the evidence graph.
    pub fn ingest_document(&mut self, doc: SourceDoc) -> Result<NodeId, GraphError> {
        let node_id = doc.node_id();
        self.graph.add_node(EvidenceNode::SourceDoc(doc))?;
        Ok(node_id)
    }

    /// Extract rows from a source document.
    pub fn extract_rows(
        &mut self,
        doc_id: &NodeId,
        rows: Vec<ExtractedRow>,
    ) -> Result<Vec<NodeId>, GraphError> {
        let mut row_ids = Vec::new();
        for row in rows {
            let row_id = row.node_id();
            self.graph.add_node(EvidenceNode::ExtractedRow(row))?;
            self.graph.add_edge(
                doc_id.clone(),
                row_id.clone(),
                EdgeType::ExtractedFrom,
            )?;
            row_ids.push(row_id);
        }
        Ok(row_ids)
    }

    /// Create a transaction from extracted rows.
    pub fn create_transaction(
        &mut self,
        tx: Transaction,
        source_rows: &[NodeId],
    ) -> Result<NodeId, GraphError> {
        let tx_id = tx.node_id();
        self.graph.add_node(EvidenceNode::Transaction(tx))?;
        for row_id in source_rows {
            self.graph
                .add_edge(row_id.clone(), tx_id.clone(), EdgeType::Produces)?;
        }
        Ok(tx_id)
    }

    /// Apply a classification to a transaction.
    pub fn classify_transaction(
        &mut self,
        classification: Classification,
    ) -> Result<NodeId, GraphError> {
        let cls_id = classification.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &classification.tx_id);
        self.graph
            .add_node(EvidenceNode::Classification(classification))?;
        self.graph.add_edge(tx_id, cls_id.clone(), EdgeType::ClassifiedAs)?;
        Ok(cls_id)
    }

    /// Record a model proposal for a transaction.
    pub fn record_proposal(&mut self, proposal: ModelProposal) -> Result<NodeId, GraphError> {
        let prop_id = proposal.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &proposal.tx_id);
        self.graph
            .add_node(EvidenceNode::ModelProposal(proposal))?;
        self.graph.add_edge(tx_id, prop_id.clone(), EdgeType::ProposedBy)?;
        Ok(prop_id)
    }

    /// Record an operator approval/rejection.
    pub fn record_approval(&mut self, approval: OperatorApproval) -> Result<NodeId, GraphError> {
        let approval_id = approval.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &approval.tx_id);
        self.graph
            .add_node(EvidenceNode::OperatorApproval(approval))?;
        self.graph
            .add_edge(tx_id, approval_id.clone(), EdgeType::ApprovedBy)?;
        Ok(approval_id)
    }

    /// Record a workbook export for a transaction.
    pub fn export_workbook_row(&mut self, wb_row: WorkbookRow) -> Result<NodeId, GraphError> {
        let wb_id = wb_row.node_id();
        let tx_id = NodeId::new(NodeType::Transaction, &wb_row.tx_id);
        self.graph.add_node(EvidenceNode::WorkbookRow(wb_row))?;
        self.graph.add_edge(tx_id, wb_id.clone(), EdgeType::ExportedTo)?;
        Ok(wb_id)
    }

    /// Build a complete evidence chain for a transaction in one call.
    pub fn build_full_chain(
        &mut self,
        doc: SourceDoc,
        rows: Vec<ExtractedRow>,
        tx: Transaction,
        classification: Classification,
    ) -> Result<(), GraphError> {
        let doc_id = self.ingest_document(doc)?;
        let row_ids = self.extract_rows(&doc_id, rows)?;
        let tx_id = self.create_transaction(tx, &row_ids)?;

        let mut cls = classification;
        cls.tx_id = tx_id.hash().to_string();
        self.classify_transaction(cls)?;

        Ok(())
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
        let doc_id = builder.ingest_document(test_doc()).unwrap();
        assert_eq!(graph.node_count(), 1);
        assert_eq!(doc_id.node_type(), NodeType::SourceDoc);
    }

    #[test]
    fn builder_extracts_rows_with_edges() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ingest_document(test_doc()).unwrap();
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.extract_rows(&doc_id, rows).unwrap();

        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(row_ids.len(), 1);
    }

    #[test]
    fn builder_creates_transaction_from_rows() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ingest_document(test_doc()).unwrap();
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.extract_rows(&doc_id, rows).unwrap();
        let tx_id = builder.create_transaction(test_tx(), &row_ids).unwrap();

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
        assert_eq!(tx_id.node_type(), NodeType::Transaction);
    }

    #[test]
    fn builder_classifies_transaction() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ingest_document(test_doc()).unwrap();
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.extract_rows(&doc_id, rows).unwrap();
        let tx_id = builder.create_transaction(test_tx(), &row_ids).unwrap();

        let cls = test_cls(tx_id.hash().to_string());
        let cls_id = builder.classify_transaction(cls).unwrap();

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);
        assert_eq!(cls_id.node_type(), NodeType::Classification);
    }

    #[test]
    fn builder_records_proposal_and_approval() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ingest_document(test_doc()).unwrap();
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.extract_rows(&doc_id, rows).unwrap();
        let tx_id = builder.create_transaction(test_tx(), &row_ids).unwrap();

        let proposal = ModelProposal {
            tx_id: tx_id.hash().to_string(),
            model_name: "phi-4-mini".to_string(),
            proposed_category: "Meals".to_string(),
            confidence: Confidence::from(0.87),
            reasoning: Some("vendor name suggests food purchase".to_string()),
            proposed_at: Utc.with_ymd_and_hms(2024, 2, 1, 10, 30, 0).unwrap(),
            validated: true,
        };
        let prop_id = builder.record_proposal(proposal).unwrap();

        let approval = OperatorApproval {
            tx_id: tx_id.hash().to_string(),
            operator_id: "user1".to_string(),
            approved: true,
            rationale: Some("correct category".to_string()),
            approved_at: Utc.with_ymd_and_hms(2024, 2, 1, 11, 0, 0).unwrap(),
        };
        let approval_id = builder.record_approval(approval).unwrap();

        assert_eq!(graph.node_count(), 5);
        assert_eq!(graph.edge_count(), 4);
        assert_eq!(prop_id.node_type(), NodeType::ModelProposal);
        assert_eq!(approval_id.node_type(), NodeType::OperatorApproval);
    }

    #[test]
    fn builder_exports_workbook_row() {
        let mut graph = EvidenceGraph::new();
        let mut builder = EvidenceBuilder::new(&mut graph);
        let doc_id = builder.ingest_document(test_doc()).unwrap();
        let rows = vec![test_row(doc_id.clone())];
        let row_ids = builder.extract_rows(&doc_id, rows).unwrap();
        let tx_id = builder.create_transaction(test_tx(), &row_ids).unwrap();

        let wb_row = WorkbookRow {
            tx_id: tx_id.hash().to_string(),
            sheet_name: "Transactions".to_string(),
            row_index: 42,
            category: "Meals".to_string(),
            amount: "-12.34".to_string(),
            exported_at: Utc.with_ymd_and_hms(2024, 2, 1, 12, 0, 0).unwrap(),
        };
        let wb_id = builder.export_workbook_row(wb_row).unwrap();

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
