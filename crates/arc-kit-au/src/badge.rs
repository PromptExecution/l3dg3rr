//! Badge system — visual indicators for review surface.
//!
//! Provides provenance status badges for the operator UI.
//! Badges indicate completeness of evidence chains.

use serde::{Deserialize, Serialize};

use crate::trace::EvidenceChain;

/// Provenance badge for review surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceBadge {
    /// Complete evidence chain with all required elements.
    Complete,
    /// Partial evidence chain with some missing elements.
    Partial { missing: Vec<String> },
    /// Missing critical evidence (source or classification).
    Missing { missing: Vec<String> },
    /// No evidence chain found.
    NotFound,
}

impl ProvenanceBadge {
    /// Display label for UI rendering.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Complete => "Complete",
            Self::Partial { .. } => "Partial",
            Self::Missing { .. } => "Missing",
            Self::NotFound => "Not Found",
        }
    }

    /// CSS class suggestion for UI styling.
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Complete => "badge-complete",
            Self::Partial { .. } => "badge-partial",
            Self::Missing { .. } => "badge-missing",
            Self::NotFound => "badge-not-found",
        }
    }

    /// Whether this badge indicates a review-required state.
    pub fn needs_review(&self) -> bool {
        !matches!(self, Self::Complete)
    }
}

impl From<&EvidenceChain> for ProvenanceBadge {
    fn from(chain: &EvidenceChain) -> Self {
        if chain.source_documents.is_empty()
            && chain.extracted_rows.is_empty()
            && chain.classifications.is_empty()
        {
            return Self::NotFound;
        }

        let missing = chain.missing_elements();
        if missing.is_empty() {
            return Self::Complete;
        }

        let critical_missing: Vec<_> = missing
            .iter()
            .filter(|m| {
                **m == "source_document"
                    || **m == "classification"
                    || **m == "extracted_rows"
            })
            .map(|s| s.to_string())
            .collect();

        if !critical_missing.is_empty() {
            Self::Missing {
                missing: critical_missing,
            }
        } else {
            Self::Partial {
                missing: missing.into_iter().map(|s| s.to_string()).collect(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::node::Confidence;
    use super::*;
    use crate::node::{EvidenceNode, NodeId, NodeType, SourceDoc};
    use chrono::TimeZone;
    use chrono::Utc;

    fn test_doc() -> SourceDoc {
        SourceDoc {
            filename: "test.pdf".to_string(),
            vendor: "V".to_string(),
            account_id: "A".to_string(),
            statement_date: "2024-01-31".to_string(),
            document_type: "statement".to_string(),
            content_hash: "hash".to_string(),
            ingested_at: Utc.with_ymd_and_hms(2024, 2, 1, 10, 0, 0).unwrap(),
            raw_context_path: None,
        }
    }

    #[test]
    fn badge_labels_are_stable() {
        assert_eq!(ProvenanceBadge::Complete.label(), "Complete");
        assert_eq!(ProvenanceBadge::NotFound.label(), "Not Found");
    }

    #[test]
    fn css_classes_are_distinct() {
        assert_eq!(ProvenanceBadge::Complete.css_class(), "badge-complete");
        assert_eq!(
            ProvenanceBadge::Partial { missing: vec![] }.css_class(),
            "badge-partial"
        );
        assert_eq!(
            ProvenanceBadge::Missing { missing: vec![] }.css_class(),
            "badge-missing"
        );
        assert_eq!(ProvenanceBadge::NotFound.css_class(), "badge-not-found");
    }

    #[test]
    fn needs_review_returns_false_for_complete() {
        assert!(!ProvenanceBadge::Complete.needs_review());
        assert!(ProvenanceBadge::NotFound.needs_review());
        assert!(ProvenanceBadge::Partial { missing: vec![] }.needs_review());
        assert!(ProvenanceBadge::Missing { missing: vec![] }.needs_review());
    }

    #[test]
    fn from_complete_chain_returns_complete_badge() {
        let chain = EvidenceChain {
            tx_id: "tx_1".to_string(),
            source_documents: vec![EvidenceNode::SourceDoc(test_doc())],
            extracted_rows: vec![EvidenceNode::ExtractedRow(crate::node::ExtractedRow {
                account_id: "A".to_string(),
                date: "2024-01-15".to_string(),
                amount: rust_decimal::Decimal::new(-1234, 2),
                description: "Test".to_string(),
                source_document: NodeId::new(NodeType::SourceDoc, "abc"),
                extraction_confidence: Confidence::from(0.95),
            })],
            classifications: vec![EvidenceNode::Classification(
                crate::node::Classification {
                    tx_id: "tx_1".to_string(),
                    category: "Meals".to_string(),
                    sub_category: None,
                    confidence: Confidence::from(0.92),
                    rule_used: None,
                    actor: "operator".to_string(),
                    classified_at: Utc.with_ymd_and_hms(2024, 2, 1, 11, 0, 0).unwrap(),
                    note: None,
                },
            )],
            proposals: vec![],
            approvals: vec![EvidenceNode::OperatorApproval(
                crate::node::OperatorApproval {
                    tx_id: "tx_1".to_string(),
                    operator_id: "user1".to_string(),
                    approved: true,
                    rationale: None,
                    approved_at: Utc.with_ymd_and_hms(2024, 2, 1, 12, 0, 0).unwrap(),
                },
            )],
            workbook_rows: vec![EvidenceNode::WorkbookRow(crate::node::WorkbookRow {
                tx_id: "tx_1".to_string(),
                sheet_name: "Transactions".to_string(),
                row_index: 1,
                category: "Meals".to_string(),
                amount: "-12.34".to_string(),
                exported_at: Utc.with_ymd_and_hms(2024, 2, 1, 13, 0, 0).unwrap(),
            })],
        };

        let badge = ProvenanceBadge::from(&chain);
        assert_eq!(badge, ProvenanceBadge::Complete);
    }

    #[test]
    fn from_empty_chain_returns_not_found() {
        let chain = EvidenceChain {
            tx_id: "tx_2".to_string(),
            source_documents: vec![],
            extracted_rows: vec![],
            classifications: vec![],
            proposals: vec![],
            approvals: vec![],
            workbook_rows: vec![],
        };

        let badge = ProvenanceBadge::from(&chain);
        assert_eq!(badge, ProvenanceBadge::NotFound);
    }

    #[test]
    fn from_chain_missing_source_returns_missing_badge() {
        let chain = EvidenceChain {
            tx_id: "tx_3".to_string(),
            source_documents: vec![],
            extracted_rows: vec![],
            classifications: vec![EvidenceNode::Classification(
                crate::node::Classification {
                    tx_id: "tx_3".to_string(),
                    category: "Meals".to_string(),
                    sub_category: None,
                    confidence: Confidence::from(0.92),
                    rule_used: None,
                    actor: "operator".to_string(),
                    classified_at: Utc.with_ymd_and_hms(2024, 2, 1, 11, 0, 0).unwrap(),
                    note: None,
                },
            )],
            proposals: vec![],
            approvals: vec![EvidenceNode::OperatorApproval(
                crate::node::OperatorApproval {
                    tx_id: "tx_3".to_string(),
                    operator_id: "user1".to_string(),
                    approved: true,
                    rationale: None,
                    approved_at: Utc.with_ymd_and_hms(2024, 2, 1, 12, 0, 0).unwrap(),
                },
            )],
            workbook_rows: vec![],
        };

        let badge = ProvenanceBadge::from(&chain);
        assert!(matches!(badge, ProvenanceBadge::Missing { .. }));
    }
}
