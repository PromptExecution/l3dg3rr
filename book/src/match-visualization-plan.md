# Match Visualization Plan

This chapter documents the planned visualization contract for a future `match` operator so the Mermaid and isometric views stay semantically aligned.

## Related Chapters

- [Theory of Operation](./theory.md) - branching examples and disposition handling
- [Workflow](./workflow.md) - TOML workflow compilation surface
- [Visualization](./visualize.md) - current live editor and view modes
- [Constraints](./constraints.md) - Kasuari placement rules
- [Verification](./verify.md) - reviewer and human escalation flows

## Problem Statement

The documentation renderer currently understands a narrow diagram DSL:
- `fn source() -> target`
- `if expression -> target`

That is enough for linear pipelines and threshold gates, but it does not yet express a true multi-arm `match` branch as one semantic node with ordered arms, fallthrough, and stable reflow behavior.

## Canonical Sample

The operator contract should be driven by small, realistic examples instead of abstract syntax fragments. The baseline sample is a disposition router:

```rust
match result.disposition {
    Disposition::Unrecoverable => halt_pipeline,
    Disposition::Recoverable => repair_and_retry,
    Disposition::Advisory => record_note,
}
```

Equivalent workflow intent:
- a single branching point reads `result.disposition`
- each arm is declaration-ordered and labeled
- terminal or review-heavy arms remain visually distinct

## Mermaid Plan

### Projection Rules

- Compile one explicit `match` node, not three unrelated `if` nodes.
- Preserve arm order from source.
- Label every outgoing edge with the arm key.
- Render a default edge when the source includes `_` or `else`.
- Keep the branch target node ids stable so diffs are readable across regenerations.

### Sample 2D Layout

```rhai
fn verify_result() -> match_result_disposition
match result.disposition => Disposition::Unrecoverable -> halt_pipeline
match result.disposition => Disposition::Recoverable -> repair_and_retry
match result.disposition => Disposition::Advisory -> record_note
fn repair_and_retry() -> requeue_validation
```

### Mermaid Acceptance Criteria

- arm labels must be the declared pattern names
- the `match` node must render once even when there are many arms
- adding a new arm should only add one new edge and one target node in the diff
- Mermaid output should stay readable when an arm routes back into the main workflow

## Isometric Plan

### Placement Model

The isometric renderer should treat `match` as a structured fan-out node:
- the `match` node sits on the main workflow spine
- each arm receives a stable lane on the `z` axis by declaration order
- the branch targets inherit the parent stage depth on `x`
- terminal or review-heavy arms may get a small `y` lift to improve readability

### Kasuari Constraint Sketch

- `x(match) == x(previous) + stage_gap`
- `x(target_arm_n) >= x(match) + branch_gap`
- `z(target_arm_n) == z(match) + arm_index * lane_gap`
- `z(target_arm_default)` should stay at the outermost lane so explicit arms keep their positions
- `x(rejoin)` should be constrained after the widest branch target, not after the first branch target

### Animated Reflow Goals

When the source changes:
- inserting a new arm should push later lanes sideways instead of replacing the full scene
- renaming an arm should preserve its lane if its identity key is unchanged
- moving an arm in source order should animate the lane swap so the operator can see what changed
- deleting an arm should collapse only the affected lane group

### Sample Isometric Intent

```rhai
fn verify_result() -> match_result_disposition
match result.disposition => Disposition::Unrecoverable -> halt_pipeline
match result.disposition => Disposition::Recoverable -> repair_and_retry
match result.disposition => Disposition::Advisory -> record_note
fn repair_and_retry() -> requeue_validation
```

The current live editor and mdBook preprocessor both accept the repeated-arm syntax above. The next implementation target is richer identity and placement semantics, not a second incompatible syntax.

## Example Gallery

### Disposition Routing

Use this as the primary worked example in docs and tests:

```rust
match result.disposition {
    Disposition::Unrecoverable => halt_pipeline,
    Disposition::Recoverable => repair_and_retry,
    Disposition::Advisory => record_note,
}
```

### Confidence Band Routing

This is the threshold analogue that should remain visually consistent with `match`:

```rust
match confidence_band {
    ConfidenceBand::High => commit_result,
    ConfidenceBand::Medium => queue_manual_review,
    ConfidenceBand::Low => escalate_to_operator,
}
```

### Keyword Match Selector

This sample ties the visualization plan back to the rule-engine roadmap:

```rust
match selector.pick(tx.description) {
    RulePick::Exact(rule) => run_rule(rule),
    RulePick::Semantic(rule) => run_rule(rule),
    RulePick::Fallback => queue_unclassified_review,
}
```

## Documentation Plan

- add at least one `match` sample to every branch-heavy chapter instead of relying on generic prose
- keep Mermaid and isometric screenshots or generated views side-by-side for the same sample
- use stable example names derived from the full expression: `match_result_disposition`, `match_confidence_band`, `match_selector_pick` — the parser generates the node ID as `match_` + `sanitize_id(expr)`, so pipeline steps must target the full sanitized form
- keep one example focused on routing to a terminal state and one focused on rejoining the main workflow

## Test Plan

- parser tests should confirm declaration order is preserved
- Mermaid snapshot tests should assert a single switch node with labeled arms
- isometric scene tests should assert stable lane ordering and animated reflow metadata
- docs validation should include at least one `match` example chapter so regressions surface in `just docgen-check`
