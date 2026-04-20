# Pipeline

The pipeline module implements a type-state pattern for the document processing workflow.

## Related Chapters

- [Validation](./validation.md) - Disposition and confidence types
- [Visualization](./visualize.md) - PipelineGraph rendering
- [Workflow](./workflow.md) - TOML DSL compilation
- [Verify](./verify.md) - Multi-model verification

## State Types

```rhai
fn ingested() -> validating
fn validating() -> classifying
fn classifying() -> reconciling
fn reconciling() -> committed
if confidence < 0.5 -> needs_review
if human_approved == true -> classifying
if fatal_error == true -> error
```

- **Ingested**: Document has been parsed
- **Validating**: Checking data integrity
- **Classifying**: Categorizing transactions
- **Reconciling**: Matching against external data
- **Committed**: Finalized andauditable
- **NeedsReview**: Awaiting human approval

## Type-State Pattern

The pipeline uses Rust's type system to enforce state transitions at compile time:

```rust
pub struct PipelineState<S> { /* ... */ }

impl PipelineState<Ingested> {
    pub fn validate(self) -> PipelineState<Validating> { ... }
}

impl PipelineState<Validating> {
    pub fn classify(self, category: String) -> PipelineState<Classified> { ... }
}
```

This ensures invalid state transitions are caught at compile time.

## Statig Integration

The pipeline uses `statig` for hierarchical state machine (HSM) implementation with:
- Superstates for grouping related states
- State-local storage for context
- Async-first design
