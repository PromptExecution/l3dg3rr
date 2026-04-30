use std::collections::BTreeMap;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ontology::{relation_content_hash, Relation};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalState {
    Proposed,
    Validated,
    Rejected,
    Approved,
    Committed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelMetadata {
    pub provider: String,
    pub model: String,
    pub endpoint_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalValidation {
    pub validator: String,
    pub passed: bool,
    pub checked_at: String,
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProposalApproval {
    pub actor: String,
    pub decided_at: String,
    pub approved: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OntologyEdgeProposal {
    pub proposal_id: String,
    pub proposed_relation: Relation,
    pub confidence: Decimal,
    pub source_artifact_ids: Vec<String>,
    #[serde(default)]
    pub semantic_context_ids: Vec<String>,
    pub model_metadata: ModelMetadata,
    pub validation: Option<ProposalValidation>,
    pub approval: Option<ProposalApproval>,
    pub state: ProposalState,
}

impl OntologyEdgeProposal {
    pub fn validate(mut self, validation: ProposalValidation) -> Self {
        self.state = if validation.passed {
            ProposalState::Validated
        } else {
            ProposalState::Rejected
        };
        self.validation = Some(validation);
        self
    }

    pub fn record_approval(mut self, approval: ProposalApproval) -> Self {
        self.state = if approval.approved {
            ProposalState::Approved
        } else {
            ProposalState::Rejected
        };
        self.approval = Some(approval);
        self
    }

    pub fn commit_relation(
        &self,
        policy: &ProposalPolicy,
    ) -> Result<Relation, ProposalCommitError> {
        let validation = self
            .validation
            .as_ref()
            .ok_or(ProposalCommitError::MissingValidation)?;
        if !validation.passed {
            return Err(ProposalCommitError::ValidationFailed);
        }

        if let Some(reason) = policy.operator_review_reason(self) {
            let approved = self
                .approval
                .as_ref()
                .filter(|approval| approval.approved)
                .ok_or(ProposalCommitError::OperatorReviewRequired(reason))?;
            if approved.actor.trim().is_empty() {
                return Err(ProposalCommitError::InvalidApproval(
                    "approval actor must be non-empty".to_string(),
                ));
            }
        }

        let mut relation = self.proposed_relation.clone();
        relation
            .provenance
            .insert("proposal_id".to_string(), self.proposal_id.clone());
        relation
            .provenance
            .insert("proposal_state".to_string(), "committed".to_string());
        relation.provenance.insert(
            "proposal_confidence".to_string(),
            self.confidence.normalize().to_string(),
        );
        relation.provenance.insert(
            "model_provider".to_string(),
            self.model_metadata.provider.clone(),
        );
        relation
            .provenance
            .insert("model_name".to_string(), self.model_metadata.model.clone());
        relation.provenance.insert(
            "validation_result".to_string(),
            validation.passed.to_string(),
        );
        relation
            .provenance
            .insert("validated_by".to_string(), validation.validator.clone());
        relation.provenance.insert(
            "source_artifact_ids".to_string(),
            self.source_artifact_ids.join(","),
        );
        if !self.semantic_context_ids.is_empty() {
            relation.provenance.insert(
                "semantic_context_ids".to_string(),
                self.semantic_context_ids.join(","),
            );
        }
        if let Some(approval) = self.approval.as_ref().filter(|approval| approval.approved) {
            relation
                .provenance
                .insert("approval_actor".to_string(), approval.actor.clone());
            relation
                .provenance
                .insert("approved_at".to_string(), approval.decided_at.clone());
        }
        relation.id = relation_content_hash(
            &relation.from,
            &relation.to,
            &relation.relation,
            &relation.provenance,
        );
        Ok(relation)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalPolicy {
    pub auto_commit_min_confidence: Decimal,
    pub review_relations: Vec<String>,
}

impl Default for ProposalPolicy {
    fn default() -> Self {
        Self {
            auto_commit_min_confidence: Decimal::new(90, 2),
            review_relations: vec!["linked_to_xero".to_string(), "projects_to".to_string()],
        }
    }
}

impl ProposalPolicy {
    pub fn operator_review_reason(&self, proposal: &OntologyEdgeProposal) -> Option<String> {
        if proposal.confidence < self.auto_commit_min_confidence {
            return Some(format!(
                "confidence {} is below auto-commit threshold {}",
                proposal.confidence.normalize(),
                self.auto_commit_min_confidence.normalize()
            ));
        }
        if self
            .review_relations
            .iter()
            .any(|relation| relation == &proposal.proposed_relation.relation)
        {
            return Some(format!(
                "{} proposals require operator review",
                proposal.proposed_relation.relation
            ));
        }
        None
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProposalCommitError {
    #[error("model proposal must be validated before commit")]
    MissingValidation,
    #[error("model proposal validation failed")]
    ValidationFailed,
    #[error("operator review required: {0}")]
    OperatorReviewRequired(String),
    #[error("invalid approval: {0}")]
    InvalidApproval(String),
}

pub fn rejected_proposal_audit_artifact(
    proposal: &OntologyEdgeProposal,
    rejected_at: impl Into<String>,
) -> BTreeMap<String, String> {
    let mut attrs = BTreeMap::new();
    attrs.insert("proposal_id".to_string(), proposal.proposal_id.clone());
    attrs.insert("state".to_string(), "rejected".to_string());
    attrs.insert("rejected_at".to_string(), rejected_at.into());
    attrs.insert(
        "relation".to_string(),
        proposal.proposed_relation.relation.clone(),
    );
    attrs.insert("from".to_string(), proposal.proposed_relation.from.clone());
    attrs.insert("to".to_string(), proposal.proposed_relation.to.clone());
    attrs.insert("model".to_string(), proposal.model_metadata.model.clone());
    attrs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_proposal(confidence: Decimal, relation_name: &str) -> OntologyEdgeProposal {
        OntologyEdgeProposal {
            proposal_id: "proposal-001".to_string(),
            proposed_relation: Relation {
                id: "draft-edge".to_string(),
                from: "artifact-a".to_string(),
                to: "artifact-b".to_string(),
                relation: relation_name.to_string(),
                provenance: BTreeMap::new(),
            },
            confidence,
            source_artifact_ids: vec!["artifact-a".to_string()],
            semantic_context_ids: Vec::new(),
            model_metadata: ModelMetadata {
                provider: "internal".to_string(),
                model: "phi-4-mini-reasoning".to_string(),
                endpoint_url: "http://127.0.0.1:15115/v1/chat/completions".to_string(),
            },
            validation: None,
            approval: None,
            state: ProposalState::Proposed,
        }
    }

    fn passing_validation() -> ProposalValidation {
        ProposalValidation {
            validator: "ledger-core".to_string(),
            passed: true,
            checked_at: "2026-04-27T00:00:00Z".to_string(),
            notes: "relation endpoints exist".to_string(),
        }
    }

    #[test]
    fn model_proposal_cannot_commit_without_validation() {
        let proposal = base_proposal(Decimal::new(95, 2), "references");

        let error = proposal
            .commit_relation(&ProposalPolicy::default())
            .expect_err("unvalidated proposal must fail closed");

        assert_eq!(error, ProposalCommitError::MissingValidation);
    }

    #[test]
    fn low_confidence_proposal_requires_operator_review() {
        let proposal =
            base_proposal(Decimal::new(70, 2), "references").validate(passing_validation());

        let error = proposal
            .commit_relation(&ProposalPolicy::default())
            .expect_err("low confidence proposal must require review");

        assert!(matches!(
            error,
            ProposalCommitError::OperatorReviewRequired(_)
        ));
    }

    #[test]
    fn approved_proposal_commits_with_model_actor_and_validation_metadata() {
        let proposal = base_proposal(Decimal::new(70, 2), "references")
            .validate(passing_validation())
            .record_approval(ProposalApproval {
                actor: "operator".to_string(),
                decided_at: "2026-04-27T00:01:00Z".to_string(),
                approved: true,
                reason: "source evidence matches".to_string(),
            });

        let relation = proposal
            .commit_relation(&ProposalPolicy::default())
            .expect("approved proposal should commit");

        assert_eq!(relation.provenance["proposal_id"], "proposal-001");
        assert_eq!(relation.provenance["proposal_state"], "committed");
        assert_eq!(relation.provenance["model_name"], "phi-4-mini-reasoning");
        assert_eq!(relation.provenance["approval_actor"], "operator");
        assert_eq!(relation.provenance["validated_by"], "ledger-core");
        assert_ne!(relation.id, "draft-edge");
    }

    #[test]
    fn semantic_context_refs_are_added_to_model_provenance() {
        let mut proposal = base_proposal(Decimal::new(95, 2), "references");
        proposal.semantic_context_ids = vec!["semantic-a".to_string(), "semantic-b".to_string()];
        let proposal = proposal.validate(passing_validation());

        let relation = proposal
            .commit_relation(&ProposalPolicy::default())
            .expect("high confidence validated proposal should commit");

        assert_eq!(
            relation.provenance["semantic_context_ids"],
            "semantic-a,semantic-b"
        );
    }

    #[test]
    fn rejected_proposal_is_audit_artifact_not_committed_relation() {
        let proposal =
            base_proposal(Decimal::new(95, 2), "references").validate(ProposalValidation {
                validator: "ledger-core".to_string(),
                passed: false,
                checked_at: "2026-04-27T00:00:00Z".to_string(),
                notes: "missing endpoint artifact".to_string(),
            });

        let error = proposal
            .commit_relation(&ProposalPolicy::default())
            .expect_err("failed validation cannot commit");
        let audit_attrs = rejected_proposal_audit_artifact(&proposal, "2026-04-27T00:02:00Z");

        assert_eq!(error, ProposalCommitError::ValidationFailed);
        assert_eq!(audit_attrs["state"], "rejected");
        assert_eq!(audit_attrs["proposal_id"], "proposal-001");
    }
}
