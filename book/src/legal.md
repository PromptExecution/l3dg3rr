# Legal Verification

The legal module provides tax rule verification across multiple jurisdictions.

```rhai
fn transaction() -> us_rules
fn transaction() -> au_rules
fn transaction() -> uk_rules
fn us_rules() -> unrecoverable
fn au_rules() -> recoverable
fn uk_rules() -> advisory
if requires_fbar_reporting == true -> advisory
```

## Jurisdiction

Supported tax jurisdictions:
- **US**: United States (Schedule C, Form 1040)
- **AU**: Australia (GST, BAS)
- **UK**: United Kingdom (VAT, CT)

## LegalSolver

```rust
pub struct LegalSolver {
    jurisdiction: Jurisdiction,
    rules: Vec<TaxRule>,
}
```

## TaxRule

```rust
pub struct TaxRule {
    pub id: String,
    pub jurisdiction: Jurisdiction,
    pub description: String,
    pub validator: fn(&Transaction) -> bool,
}
```

## Verification

The solver checks transactions against jurisdiction-specific tax rules:
- US: Schedule C business expense categorization
- AU: GST Act s38-190 compliance
- UK: VAT deductibility rules

## Usage

```rust
let solver = LegalSolver::us();
let result = solver.verify(&transaction);
match result.disposition {
    Disposition::Unrecoverable => { /* fatal tax issue */ }
    Disposition::Recoverable => { /* fixable */ }
    Disposition::Advisory => { /* suggestion */ }
}
```
