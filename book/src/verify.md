# Verification

The verify module implements multi-model verification for transaction classification and rule repair.

## Multi-Model Verification Flow

```rhai
fn transaction_input() -> proposer_model
fn proposer_model() -> decision_store
fn decision_store() -> reviewer_model
if reviewer_agreed == true -> accepted_result
if reviewer_agreed == false -> human_review
fn human_review() -> accepted_result
```

The system uses a two-model approach:
1. **Proposer**: Primary model generates the classification/decision via `ModelClient::complete()`
2. **Decision Store**: Intermediate representation of the proposal
3. **Reviewer**: Second model reviews and validates the proposal
4. **Outcome**: Either `VerificationOutcome::Approved` or `VerificationOutcome::Rejected`

## Core Types

### MultiModelVerifier
```rust
pub struct MultiModelVerifier<C: ModelClient> {
    proposer: C,
    reviewer: C,
    config: MultiModelConfig,
}
```

### ModelClient Trait
```rust
pub trait ModelClient: Send + Sync {
    fn complete(&self, prompt: &str, max_tokens: usize) -> anyhow::Result<String>;
    fn extract<T: serde::de::DeserializeOwned>(&self, prompt: &str) -> anyhow::Result<T>;
}
```

### RepairProposal
```rust
pub struct RepairProposal {
    pub rule_id: String,
    pub proposed_fix: String,
    pub reasoning: String,
    pub confidence: f32,
}
```

### ReviewResult
```rust
pub struct ReviewResult {
    pub approved: bool,
    pub concerns: Vec<String>,
    pub suggestions: Vec<String>,
    pub confidence: f32,
}
```

## Verification Process

1. Input transaction with classification issue
2. **Proposal**: Proposer model generates `RepairProposal` with suggested fix and confidence
3. **Review**: Reviewer model evaluates proposal, returns `ReviewResult`
4. **Decision**: 
   - If `review.approved && review.confidence >= threshold` → `VerificationOutcome::Approved`
   - Otherwise → `VerificationOutcome::Rejected` (flags for human review)
5. Confidence score reflects agreement between both models

## Usage Example

```rust
use ledger_core::verify::{MockModelClient, MultiModelConfig, MultiModelVerifier};

// Create mock models for testing
let proposer = MockModelClient::default().with_response(
    r#"{"rule_id":"ForeignIncome","proposed_fix":"ForeignIncome",
        "reasoning":"Foreign wire transfer","confidence":0.92}"#
);
let reviewer = MockModelClient::default().with_response(
    r#"{"approved":true,"concerns":[],"suggestions":[],"confidence":0.90}"#
);

let config = MultiModelConfig::new("claude-haiku-4-5", "claude-haiku-4-5")
    .with_threshold(0.80);

let verifier = MultiModelVerifier::new(proposer, reviewer, config);

// Verify a classification issue
let outcome = verifier.verify(
    "ForeignIncome",
    r#"[{"field":"category","value":"Unclassified","confidence":0.3}]"#,
    "Wire transfer from DE employer, $5000"
)?;

match outcome {
    VerificationOutcome::Approved { proposal, review } => {
        println!("Approved: {} (confidence: {})", proposal.rule_id, review.confidence);
    }
    VerificationOutcome::Rejected { .. } => {
        println!("Rejected: needs human review");
    }
}
```

## Testing

The verify module provides `MockModelClient` for testing without real API calls. The integration test `test_llm_verification_proposes_category` demonstrates full proposer-reviewer flow with mock clients.
