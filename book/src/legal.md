# Legal Verification

The legal module verifies hard tax predicates with Z3. Rhai rules can propose a category and confidence score, but legal verification answers a stricter question: "are the known facts compatible with this rule's required conditions?"

## Solver Role

```rhai
fn classified_transaction() -> transaction_facts
fn transaction_facts() -> legal_rule
fn legal_rule() -> z3_solver
fn z3_solver() -> z3_result
match z3_result => Satisfied -> continue_pipeline
match z3_result => Violated -> create_review_flag
match z3_result => Unknown -> request_more_facts
```

`LegalSolver::verify()` currently covers:

- AU GST Act s38-190 style SaaS tax-code checks.
- US Schedule C ordinary-and-necessary deduction checks.

## Z3 Integration

The Rust crate `z3` provides an idiomatic wrapper over Microsoft's Z3 theorem prover. The repo currently pins `z3 = "0.8"` behind the `ledger-core/legal-z3` feature because local developer machines may not have native `libz3` installed. When that feature is enabled, `ledger-core` checks for common native Z3 library locations and emits a Cargo warning if the library is missing. On Ubuntu/WSL, install it with `sudo apt install -y libz3-dev`.

docs.rs currently shows newer `z3` 0.20.0 APIs, so examples in this book describe the application pattern rather than relying on newest-version syntax.

The core pattern is:

1. Convert known `TransactionFacts` into boolean predicates.
2. Build a violation formula.
3. When `legal-z3` is enabled, ask Z3 whether the violation formula is satisfiable.
4. Interpret `unsat` as `Z3Result::Satisfied`, `sat` as `Z3Result::Violated`, and solver unknown as `Z3Result::Unknown`.

This makes the result explainable: a violation is not just a failed `if` branch; it is a satisfiable counterexample to the rule obligation.

When `legal-z3` is not enabled, `LegalSolver` preserves the same public result semantics with a deterministic boolean mirror so default builds do not require system Z3.

## Hard vs Soft Constraints

Z3 should handle hard proof obligations:

- tax rule implication checks
- mutually exclusive classifications
- reconciliation arithmetic that must balance
- workflow commit guards
- workbook export invariants

Kasuari remains the right tool for soft plausibility constraints and layout constraints:

- vendor amount ranges
- weak/medium/strong historical expectations
- graph and isometric placement constraints

## Rule Examples

### AU GST

For a foreign SaaS vendor, the hard predicate is:

```text
foreign_vendor AND saas_supply AND NOT tax_code_BASEXCLUDED
```

If Z3 says that violation is satisfiable, the transaction is flagged. If it is unsatisfiable, the tax code satisfies the rule.

### US Schedule C

For a business expense, the hard predicate is:

```text
business_activity AND NOT (ordinary AND necessary)
```

If satisfiable, the deduction lacks a required fact and should be reviewed or repaired.

## Transaction Facts

```rust
pub struct TransactionFacts {
    pub vendor_jurisdiction: Option<String>,
    pub supply_type: Option<String>,
    pub tax_code: Option<String>,
    pub amount: Option<String>,
    pub is_business_activity: Option<bool>,
    pub is_ordinary: Option<bool>,
    pub is_necessary: Option<bool>,
}
```

Unknown facts should produce `Z3Result::Unknown` rather than pretending the transaction passed. This keeps the legal layer conservative and audit-friendly.

## Related Chapters

- [Rule Engine](./rule-engine.md)
- [Validation](./validation.md)
- [Constraints](./constraints.md)
- [Workbook & Audit](./workbook-audit.md)
