# Constraints

The constraints module handles plausibility checks and visual placement rules. It deliberately sits next to, but separate from, [Legal Verification](./legal.md).

## Constraint Families

```rhai
fn transaction_input() -> vendor_constraints
fn transaction_input() -> invoice_arithmetic
fn pipeline_graph() -> layout_constraints
fn vendor_constraints() -> constraint_evaluation
fn invoice_arithmetic() -> constraint_evaluation
fn layout_constraints() -> visualization_model
match constraint_evaluation => Required -> block_pipeline
match constraint_evaluation => Strong -> recoverable_issue
match constraint_evaluation => Medium -> warning_issue
match constraint_evaluation => Weak -> advisory_issue
```

## Kasuari Use

Kasuari-style strengths are used for constraints where failure is graded:

- **Required**: must pass before the pipeline proceeds.
- **Strong**: recoverable issue; normally needs repair or review.
- **Medium**: warning; may proceed with an audit note.
- **Weak**: advisory signal.

This is appropriate for vendor plausibility and document-shape expectations because historical data is rarely a hard legal proof.

## Z3 Boundary

Use Z3 when the application needs proof-like yes/no behavior:

- tax rule satisfaction
- reconciliation balance equations
- workbook export invariants
- mutually exclusive classifications
- workflow transition guards

Use this module's constraint evaluation when the question is "how plausible is this value?" rather than "is this formula satisfiable?"

## VendorConstraintSet

```rust
pub struct VendorConstraintSet {
    pub vendor: String,
    pub constraints: Vec<Constraint>,
}
```

Typical checks:

- amount range
- date window
- description pattern
- account format

## InvoiceConstraintSolver

`InvoiceConstraintSolver` checks invoice arithmetic such as subtotal, tax, and total consistency. Today it is a lightweight plausibility solver; future work can route strict arithmetic proof obligations through Z3 where audit explanations need counterexamples.

## LayoutSolver

The visualization system also uses constraints to keep graph nodes readable. Match arms, default lanes, and rejoin points are layout constraints, not financial constraints.

## Related Chapters

- [Legal Verification](./legal.md)
- [Validation](./validation.md)
- [Visualization](./visualize.md)
- [Match Visualization Plan](./match-visualization-plan.md)
