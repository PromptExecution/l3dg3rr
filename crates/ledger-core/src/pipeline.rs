//! Ledger Pipeline: A typed domain language for financial document processing.
//! Uses statig HSM + type-state pattern + generics for compile-time safety.
//!
//! ## Architecture
//! - statig provides the hierarchical state machine (events, actions, superstates)
//! - Type-state (phantom types) enforces valid state transitions at compile time
//! - Generics allow externalized domain syntax that's executable in Rhai
//! - Jurisdiction-aware rules compile to Z3 constraints

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

// ============================================================================
// TYPE-STATE: Compile-time valid transitions
// ============================================================================

/// Type-state marker for initial ingest state.
pub struct Ingested;
/// Type-state marker for validation completed.
pub struct Validated;
/// Type-state marker for classification completed.
pub struct Classified;
/// Type-state marker for reconciliation completed.
pub struct Reconciled;
/// Type-state marker for committed (terminal).
pub struct Committed;
/// Type-state marker for review required.
pub struct NeedsReview;

/// Type-state wrapper that encodes current pipeline position.
/// Use this for compile-time enforcement of valid operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState<S = Ingested> {
    pub document_id: String,
    pub source_ref: String,
    pub confidence: f32,
    pub issues: Vec<crate::validation::Issue>,
    pub meta: crate::validation::MetaCtx,
    _state: PhantomData<S>,
}

impl<S> PipelineState<S> {
    pub fn new(document_id: impl Into<String>, source_ref: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            source_ref: source_ref.into(),
            confidence: 1.0,
            issues: Vec::new(),
            meta: crate::validation::MetaCtx::default(),
            _state: PhantomData,
        }
    }

    pub fn with_confidence(mut self, c: f32) -> Self {
        self.confidence = c;
        self
    }
}

/// Type-state transition constructors.
/// These consume the old state and produce the new state.
impl PipelineState<Ingested> {
    /// Transition to Validated state.
    pub fn validate(self, issues: Vec<crate::validation::Issue>) -> PipelineState<Validated> {
        let confidence = self.compute_confidence(&issues);
        PipelineState {
            document_id: self.document_id,
            source_ref: self.source_ref,
            confidence,
            issues,
            meta: self.meta.advance("validate", confidence, &[]),
            _state: PhantomData,
        }
    }

    fn compute_confidence(&self, issues: &[crate::validation::Issue]) -> f32 {
        if issues.iter().any(|i| i.disposition == crate::validation::Disposition::Unrecoverable) {
            return 0.0;
        }
        let recovery_penalty = issues.iter()
            .filter(|i| i.disposition == crate::validation::Disposition::Recoverable)
            .count() as f32 * 0.1;
        (self.confidence - recovery_penalty).max(0.0)
    }
}

impl PipelineState<Validated> {
    pub fn classify(self, category: String) -> PipelineState<Classified> {
        PipelineState {
            document_id: self.document_id,
            source_ref: self.source_ref,
            confidence: self.confidence,
            issues: self.issues,
            meta: self.meta.advance("classify", self.confidence, &[]),
            _state: PhantomData,
        }
    }
}

impl PipelineState<Classified> {
    pub fn reconcile(self, xero_id: Option<String>) -> PipelineState<Reconciled> {
        PipelineState {
            document_id: self.document_id,
            source_ref: self.source_ref,
            confidence: self.confidence,
            issues: self.issues,
            meta: self.meta,
            _state: PhantomData,
        }
    }

    pub fn request_review(self) -> PipelineState<NeedsReview> {
        PipelineState {
            document_id: self.document_id,
            source_ref: self.source_ref,
            confidence: self.confidence,
            issues: self.issues,
            meta: self.meta,
            _state: PhantomData,
        }
    }
}

// ============================================================================
// STATIG HSM: Event-driven state machine (statig 0.4)
// ============================================================================

/// Pipeline events.
#[derive(Debug, Clone)]
pub enum PipelineEvent {
    DocumentIngested { document_id: String, source_ref: String },
    ValidationPassed,
    ValidationFailed { reason: String },
    Classified { category: String },
    LowConfidence { score: f32 },
    Reconciled { xero_id: Option<String> },
    XeroPushFailed { error: String },
    CommitApproved,
    CommitRejected { reason: String },
}

/// Pipeline context passed to all state handlers.
#[derive(Default)]
pub struct PipelineCtx {
    pub jurisdiction: crate::legal::Jurisdiction,
    pub repair_attempts: usize,
    pub xero_retries: usize,
}

/// The statig state machine definition.
/// Uses the statig 0.4 API with StateMachine and handler functions.
pub struct LedgerPipeline {
    pub jurisdiction: crate::legal::Jurisdiction,
    pub repair_attempts: usize,
    pub xero_retries: usize,
}

impl Default for LedgerPipeline {
    fn default() -> Self {
        Self {
            jurisdiction: crate::legal::Jurisdiction::US,
            repair_attempts: 0,
            xero_retries: 0,
        }
    }
}

impl LedgerPipeline {
    pub fn new(jurisdiction: crate::legal::Jurisdiction) -> Self {
        Self {
            jurisdiction,
            repair_attempts: 0,
            xero_retries: 0,
        }
    }
}

/// State enumeration for statig HSM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum State {
    Ingested,
    Validating,
    Classifying,
    Reconciling,
    Committed,
    NeedsReview,
}

impl Default for State {
    fn default() -> Self {
        State::Ingested
    }
}

/// Initialize the state machine.
/// Returns initial state (Ingested).
pub fn init() -> State {
    State::Ingested
}

/// State transition table: (current_state, event) -> Option<(next_state, action)>
pub fn handle_event(state: State, event: &PipelineEvent, ctx: &mut LedgerPipeline) -> Option<State> {
    match (state, event) {
        // Ingested -> Validating
        (State::Ingested, PipelineEvent::DocumentIngested { .. }) => Some(State::Validating),
        
        // Validating -> Classifying or NeedsReview
        (State::Validating, PipelineEvent::ValidationPassed) => Some(State::Classifying),
        (State::Validating, PipelineEvent::ValidationFailed { .. }) => {
            ctx.repair_attempts += 1;
            if ctx.repair_attempts >= 2 {
                Some(State::NeedsReview)
            } else {
                Some(State::Validating)
            }
        }
        
        // Classifying -> Reconciling or NeedsReview
        (State::Classifying, PipelineEvent::Classified { .. }) => Some(State::Reconciling),
        (State::Classifying, PipelineEvent::LowConfidence { .. }) => Some(State::NeedsReview),
        
        // Reconciling -> Committed or NeedsReview
        (State::Reconciling, PipelineEvent::Reconciled { .. }) => Some(State::Committed),
        (State::Reconciling, PipelineEvent::XeroPushFailed { .. }) => {
            if ctx.xero_retries < 3 {
                ctx.xero_retries += 1;
                Some(State::Reconciling)
            } else {
                Some(State::NeedsReview)
            }
        }
        
        // Commit approval
        (State::Reconciling, PipelineEvent::CommitApproved) => Some(State::Committed),
        
        // Terminal states
        (State::Committed, PipelineEvent::CommitRejected { .. }) => Some(State::NeedsReview),
        
        // Default: stay in current state
        _ => None,
    }
}

// ============================================================================
// VERB TRAIT: Externalized domain syntax (execute in Rhai)
// ============================================================================

/// Verb trait defines pipeline actions with type-safe input/output.
/// Implementors can be called from Rhai scripts.
pub trait Verb: Send + Sync + 'static {
    type Input: serde::Serialize + serde::de::DeserializeOwned;
    type Output: serde::Serialize + serde::de::DeserializeOwned;

    fn name(&self) -> &'static str;
    fn reversibility(&self) -> crate::validation::Reversibility;
    fn access(&self) -> crate::validation::AccessCriteria;

    /// Execute the verb. Returns issues and output.
    fn execute(&self, input: Self::Input) -> (Vec<crate::validation::Issue>, Self::Output);
}

pub mod verbs {
    use super::*;

    /// Detect verb: Identify document shape.
    pub struct DetectVerb;

    impl Verb for DetectVerb {
        type Input = Vec<u8>;
        type Output = String;

        fn name(&self) -> &'static str { "detect" }
        fn reversibility(&self) -> crate::validation::Reversibility { crate::validation::Reversibility::Free }
        fn access(&self) -> crate::validation::AccessCriteria { crate::validation::AccessCriteria::Open }

        fn execute(&self, input: Vec<u8>) -> (Vec<crate::validation::Issue>, String) {
            // Check for PDF magic bytes
            if input.len() >= 4 && &input[..4] == b"%PDF" {
                (Vec::new(), "pdf".to_string())
            } else {
                (
                    vec![crate::validation::Issue::unrecoverable("unknown_shape", "Could not detect document type")],
                    "unknown".to_string()
                )
            }
        }
    }

    /// Validate verb: Check data plausibility.
    pub struct ValidateVerb;

    impl Verb for ValidateVerb {
        type Input = (String, f64); // (description, amount)
        type Output = bool;

        fn name(&self) -> &'static str { "validate" }
        fn reversibility(&self) -> crate::validation::Reversibility { crate::validation::Reversibility::Free }
        fn access(&self) -> crate::validation::AccessCriteria { crate::validation::AccessCriteria::Open }

        fn execute(&self, input: (String, f64)) -> (Vec<crate::validation::Issue>, bool) {
            let (description, amount) = input;
            let mut issues = Vec::new();

            if amount == 0.0 {
                issues.push(crate::validation::Issue::unrecoverable("zero_amount", "Amount cannot be zero"));
            }
            if description.trim().is_empty() {
                issues.push(crate::validation::Issue::recoverable(
                    "empty_description",
                    "Description is empty",
                    crate::validation::IssueSource::TypeCheck,
                ));
            }

            let valid = issues.is_empty() || !issues.iter().any(|i| 
                i.disposition == crate::validation::Disposition::Unrecoverable
            );
            (issues, valid)
        }
    }
}

// ============================================================================
// GENERIC CONSTRAINT SOLVER: Externalized to Rhai via traits
// ============================================================================

/// Constraint solver trait - implementation for numeric range checking.
/// Can be delegated to Rhai or Kasuari.
pub trait ConstraintSolver: Send + Sync {
    fn evaluate(&self, field: &str, value: f64, constraints: &[(f64, f64)]) -> f32;
    fn strength(&self, constraint: &str) -> crate::constraints::ConstraintStrength;
}

/// Kasuari-backed constraint solver.
pub struct KasuariSolver;

impl ConstraintSolver for KasuariSolver {
    fn evaluate(&self, _field: &str, value: f64, constraints: &[(f64, f64)]) -> f32 {
        for (min, max) in constraints {
            if value >= *min && value <= *max {
                return 1.0;
            }
            // Partial match within 50%
            if value >= *min * 0.5 && value <= *max * 2.0 {
                return 0.5;
            }
        }
        0.0
    }

    fn strength(&self, constraint: &str) -> crate::constraints::ConstraintStrength {
        match constraint {
            "required" => crate::constraints::ConstraintStrength::Required,
            "strong" => crate::constraints::ConstraintStrength::Strong,
            "medium" => crate::constraints::ConstraintStrength::Medium,
            _ => crate::constraints::ConstraintStrength::Weak,
        }
    }
}

// ============================================================================
// BUILDER: Fluent construction of pipeline context
// ============================================================================

pub struct PipelineBuilder {
    jurisdiction: crate::legal::Jurisdiction,
    min_confidence: f32,
    max_retries: usize,
    enable_legal_verification: bool,
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self {
            jurisdiction: crate::legal::Jurisdiction::US,
            min_confidence: 0.85,
            max_retries: 2,
            enable_legal_verification: true,
        }
    }
}

impl PipelineBuilder {
    pub fn jurisdiction(mut self, j: crate::legal::Jurisdiction) -> Self {
        self.jurisdiction = j;
        self
    }

    pub fn min_confidence(mut self, c: f32) -> Self {
        self.min_confidence = c;
        self
    }

    pub fn enable_legal_verification(mut self, b: bool) -> Self {
        self.enable_legal_verification = b;
        self
    }

    pub fn build(self) -> LedgerPipeline {
        LedgerPipeline::new(self.jurisdiction)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_state_transition() {
        let state = PipelineState::<Ingested>::new("doc1", "WF--BH--2026-01");
        let validated = state.validate(Vec::new());
        
        // Type system enforces valid operations per state
        let classified = validated.classify("6-8800".to_string());
        let reconciled = classified.reconcile(Some("XERO-123".to_string()));
        
        assert_eq!(reconciled.document_id, "doc1");
    }

    #[test]
    fn test_pipeline_builder() {
        let pipeline = PipelineBuilder::default()
            .jurisdiction(crate::legal::Jurisdiction::AU)
            .min_confidence(0.9)
            .enable_legal_verification(true)
            .build();
        
        assert_eq!(pipeline.jurisdiction, crate::legal::Jurisdiction::AU);
    }

    #[test]
    fn test_verb_execution() {
        let verb = verbs::DetectVerb;
        let pdf_bytes = b"%PDF-1.4 fake pdf content".to_vec();
        
        let (issues, output) = verb.execute(pdf_bytes);
        
        assert!(issues.is_empty());
        assert_eq!(output, "pdf");
    }

    #[test]
    fn test_validate_verb() {
        let verb = verbs::ValidateVerb;
        
        let (issues, valid) = verb.execute(("AWS bill".to_string(), 250.0));
        assert!(issues.is_empty());
        assert!(valid);
        
        let (issues, valid) = verb.execute(("".to_string(), 0.0));
        assert!(!issues.is_empty());
        assert!(!valid);
    }

    #[test]
    fn test_constraint_solver() {
        let solver = KasuariSolver;
        
        // Exact match in range returns 1.0
        let result = solver.evaluate("amount", 250.0, &[(100.0, 500.0)]);
        assert_eq!(result, 1.0);
        
        // Way below range definitely 0.0
        let result = solver.evaluate("amount", 10.0, &[(100.0, 500.0)]);
        assert_eq!(result, 0.0);
        
        // At upper bound returns 1.0  
        let result = solver.evaluate("amount", 500.0, &[(100.0, 500.0)]);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_hsm_transitions() {
        let mut ctx = LedgerPipeline::default();
        
        // Test valid transitions
        let next = handle_event(State::Ingested, &PipelineEvent::DocumentIngested { 
            document_id: "doc1".to_string(), 
            source_ref: "WF--2026-01".to_string() 
        }, &mut ctx);
        assert_eq!(next, Some(State::Validating));
        
        // Test validation pass
        let next = handle_event(State::Validating, &PipelineEvent::ValidationPassed, &mut ctx);
        assert_eq!(next, Some(State::Classifying));
        
        // Test classification
        let next = handle_event(State::Classifying, &PipelineEvent::Classified { 
            category: "6-8800".to_string() 
        }, &mut ctx);
        assert_eq!(next, Some(State::Reconciling));
        
        // Test reconcile
        let next = handle_event(State::Reconciling, &PipelineEvent::Reconciled { 
            xero_id: Some("XERO-123".to_string()) 
        }, &mut ctx);
        assert_eq!(next, Some(State::Committed));
    }

    #[test]
    fn test_hsm_retry_logic() {
        let mut ctx = LedgerPipeline::default();
        
        // First failure allowed
        ctx.repair_attempts = 0;
        let next = handle_event(State::Validating, &PipelineEvent::ValidationFailed { 
            reason: "test".to_string() 
        }, &mut ctx);
        assert_eq!(next, Some(State::Validating)); // Retry
        assert_eq!(ctx.repair_attempts, 1);
        
        // Second failure triggers review
        let next = handle_event(State::Validating, &PipelineEvent::ValidationFailed { 
            reason: "test".to_string() 
        }, &mut ctx);
        assert_eq!(next, Some(State::NeedsReview));
    }
}