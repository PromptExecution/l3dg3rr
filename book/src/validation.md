# Validation

The validation module provides the core type system for pipeline stage results.

## Related Chapters

- [Pipeline](./pipeline.md) - State machine integration
- [Constraints](./constraints.md) - Kasuari constraint types
- [Verify](./verify.md) - Multi-model confidence calculation
- [Legal Verification](./legal.md) - Tax rule disposition

## Disposition

```rust
pub enum Disposition {
    Unrecoverable,  // Fatal error, cannot proceed
    Recoverable,    // Error that can be fixed
    Advisory,       // Warning or suggestion
}
```

## Issue

```rust
pub struct Issue {
    pub disposition: Disposition,
    pub message: String,
    pub source: IssueSource,
}
```

## MetaCtx

Metadata context that accumulates through pipeline stages:

```rust
pub struct MetaCtx {
    pub accumulated_confidence: f32,  // Compounded from all stages
    pub stage_history: Vec<StageResult>,
    // ...
}
```

Confidence compounds multiplicatively: `next.confidence = current.confidence * stage.confidence`

## StageResult

Captures the outcome of a pipeline stage with issues and confidence score.