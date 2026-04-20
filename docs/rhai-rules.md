# Rhai Classification Rules

## Overview

tax-ledger uses the [`rhai`](https://docs.rs/rhai/1.24.0/rhai/) scripting engine (v1.24.0) to classify financial transactions at runtime without requiring a Rust recompile. Classification rules live in `.rhai` files under `rules/` at the project root. An agent or developer can edit, add, or replace rule files independently of the Rust build.

The engine classifies each transaction into a **TaxCategory** string and optionally raises a **review flag** that surfaces the transaction for CPA inspection in the audit workbook.

Rules are agent-editable by design: the Rhai syntax is simple, the function contract is narrow (one `fn classify(tx)` per file), and rule files can be tested in isolation without touching the Rust codebase.

---

## Rule File Conventions

### Location

```
rules/
  classify_foreign_income.rhai
  classify_self_employment.rhai
  classify_fallback.rhai
  ...
```

Place rules at the project root under `rules/`. The Rust engine resolves rule file paths explicitly — there is no auto-discovery scan. Add a new rule file and wire it into your pipeline call site manually.

### Naming

Use `classify_<category_name>.rhai`, lowercase with underscores. The filename is for humans and agents; the Rhai engine only cares about the exported `fn classify(tx)` function inside.

### Required export

Each rule file must export exactly one function with this signature:

```rhai
fn classify(tx) { ... }
```

`tx` is a Rhai object map (`#{ ... }`) with these string fields (all strings, no numeric types):

| Field | Type | Example | Notes |
|---|---|---|---|
| `tx_id` | String | `"a3f9..."` | Blake3 hex hash |
| `account_id` | String | `"HSBC--BH-CHK--2024-03"` | `VENDOR--ACCOUNT--YYYY-MM` format |
| `date` | String | `"2024-03-15"` | ISO 8601 |
| `amount` | String | `"4250.00"` or `"-99.99"` | Decimal as string — see Gotchas |
| `description` | String | `"Wire from DE employer"` | Free text from statement |

### Required return shape

`fn classify(tx)` must return an object map with exactly these four fields:

```rhai
#{
    category:   "ForeignIncome",   // String — one of the valid TaxCategory values
    confidence: 0.90,              // f64 in [0.0, 1.0]
    review:     false,             // bool — true raises a ReviewFlag in Rust
    reason:     "matched HSBC"     // String — audit trail explanation
}
```

If any field is missing or the wrong type, the Rust engine returns `ClassificationError::InvalidOutput` and the pipeline aborts for that transaction.

### Valid TaxCategory strings

Use exactly these strings — they round-trip through `strum`-derived enum parsing:

```
ForeignIncome
SelfEmployment
Investment
OfficeSupplies
Travel
MealsEntertainment
HealthInsurance
Unclassified
Transfer
```

---

## Engine Registration Reference

The Rhai engine is created with `Engine::new()` — no custom type registration beyond Rhai's standard library. Key implications:

| Concern | Detail |
|---|---|
| `rust_decimal::Decimal` | **Not registered.** The Rust side serializes `Decimal` to a string before passing it into Rhai. Use `parse_float()` in rules for numeric comparisons. |
| Standard functions available | `parse_float()`, `parse_int()`, String methods (`.contains()`, `.to_lower()`, etc.), arithmetic operators |
| Custom Rust functions | None registered at this time. All logic lives inside the `.rhai` file. |
| Operation limits | **None set.** See Gotchas. |
| Modules | No external Rhai modules loaded. Standard packages only. |

The Rust call site (simplified):

```rust
let engine = Engine::new();
let ast = engine.compile(src)?;          // ParseError surfaced as ClassificationError::Compile
let output: Map = engine.call_fn(        // EvalAltResult surfaced as ClassificationError::Eval
    &mut scope, &ast, "classify", (tx_map,)
)?;
```

No `.unwrap()` is used on eval results — errors propagate through `ClassificationError`.

---

## Review Flag Mechanism

There is no separate `get_flags()` function. Flags work as follows:

1. The rule sets `review: true` in its return map (or `review: false`).
2. The Rust engine also applies a `review_threshold: f64` parameter. If `confidence < review_threshold`, `needs_review` is forced to `true` regardless of what the rule returned.
3. When `needs_review == true`, `classify_rows_from_file` calls `upsert_open_flag`, which stores a `ReviewFlag` in the engine's in-memory flag list.
4. Flags are queryable via `engine.query_flags(year, FlagStatus::Open)`.

Do not call `get_flags()` in any rule — it is not registered.

---

## Sample Test Workflow

### Run all Rhai integration tests

```sh
cargo test -p ledger-core rhai_rules
```

Tests live in `crates/ledger-core/tests/rhai_rules.rs` and resolve rule files from the workspace root via `CARGO_MANIFEST_DIR`.

### Run a single test

```sh
cargo test -p ledger-core rhai_01_foreign_income_happy_path
```

### Test a rule in isolation (without Cargo)

If `rhai-run` is available (from the `rhai` crate's CLI feature), you can execute a rule file directly:

```sh
# Not currently wired — use cargo test instead
rhai-run rules/classify_foreign_income.rhai
```

Without `rhai-run`, write a small Rust harness or use the existing test infrastructure.

---

## Writing a New Rule: Step-by-Step

**1. Create the rule file.**

```sh
touch rules/classify_investment.rhai
```

**2. Write the `fn classify(tx)` function.**

```rhai
// rules/classify_investment.rhai
fn classify(tx) {
    let description = "";
    if tx.contains("description") {
        description = tx["description"].to_lower();
    }

    let is_investment = description.contains("dividend")
        || description.contains("brokerage")
        || description.contains("etf");

    if !is_investment {
        return #{
            category:   "Unclassified",
            confidence: 0.0,
            review:     false,
            reason:     "no investment keyword"
        };
    }

    #{
        category:   "Investment",
        confidence: 0.88,
        review:     false,
        reason:     "investment keyword matched"
    }
}
```

**3. Write an integration test in `crates/ledger-core/tests/rhai_rules.rs`.**

```rust
#[test]
fn rhai_investment_dividend_classified() {
    let sample = SampleTransaction {
        tx_id: "test-inv-01".into(),
        account_id: "SCHWAB--BH-BROK--2024-01".into(),
        date: "2024-01-31".into(),
        amount: "45.00".into(),
        description: "SCHB dividend reinvestment".into(),
    };

    let outcome = engine()
        .run_rule_from_file(&rule_path("classify_investment.rhai"), &sample)
        .unwrap_or_else(|e| panic!("rule execution failed: {e}"));

    assert_eq!(outcome.category, "Investment");
    assert!(!outcome.needs_review);
}
```

**4. Run the test.**

```sh
cargo test -p ledger-core rhai_investment_dividend_classified
```

**5. Wire the rule into your pipeline call site** in `classify_rows_from_file` or the equivalent chain runner once it passes.

---

## Gotchas and Known Limitations

### No operation limits (runaway script risk)

`Engine::new()` is used without `engine.set_max_operations(n)` or `engine.set_max_call_levels(n)`. A malformed or adversarial rule file could loop indefinitely and hang the pipeline.

**Recommended mitigation:** Add the following to the engine setup in `classify.rs`:

```rust
let mut engine = Engine::new();
engine.set_max_operations(100_000);  // ~100k ops is ample for a classification rule
engine.set_max_call_levels(32);
```

Until this is added, do not load `.rhai` files from untrusted sources.

### Amount is a string — use `parse_float()` for comparisons only

`rust_decimal::Decimal` is not a registered Rhai type. The Rust side converts amounts to strings before passing them into the engine. In rule scripts:

- **Do** use `parse_float(tx["amount"])` for threshold comparisons (e.g. `> 10000.0`).
- **Do not** return float amounts from rules or use them for anything other than comparisons.
- **Do not** assume the string is always positive — amounts may carry a leading `-` for debits.

### Map field access — always guard for missing keys

Rhai maps return unit `()` for missing keys; `()` is not an empty string. A guard pattern prevents type errors:

```rhai
let description = "";
if tx.contains("description") {
    description = tx["description"];
}
```

### `confidence` must be an f64 literal, not an integer

Rhai distinguishes integer and float literals. Writing `confidence: 1` returns an integer `Dynamic`, which will fail `try_cast::<f64>()` on the Rust side. Always write `1.0`, `0.85`, `0.0` — never bare integers for the confidence field.

### Rule files do not chain automatically

There is no built-in rule chaining. The Rust call site loads and runs one rule file at a time. If you want priority-ordered chains (run foreign_income first, fall through to fallback), implement that in Rust by running multiple rule files in sequence and stopping at the first non-Unclassified result.
