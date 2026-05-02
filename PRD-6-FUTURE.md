# PRD-6-FUTURE: Type Attestation System — Dual-Solver Formally Provable Invariant Ledger

**Status:** Concept Definition | **Priority:** Research / Post-PRD-8 | **Date:** 2026-05-02

---

## 1. What Is This?

This document defines a future capability where a type in `l3dg3rr` that claims a formal property — such as "this amount is GST-exclusive", "this transaction is Schedule-C deductible", or "this pipeline state has passed legal review" — must **prove** that claim through testable invariant assertions recorded in an immutable, dual-solver-verifiable audit ledger.

It is not just documentation. It is a machine-checkable, self-extensible knowledge system where every formal type claim produces an entry in an append-only invariant record that can be re-verified at any time by both Z3 (symbolic/logical predicates) and Kasuari (numerical/bounds constraints), and formally proved correct by Kani.

---

## 2. Problem It Solves

Today, the type system enforces *structural* correctness (a `PipelineState<Classified>` cannot be confused with a `PipelineState<Reconciled>`), but it cannot enforce *semantic* correctness — there is no machine-readable record that `PipelineState<Classified>` implies the transaction passed legal review, that the amount is within a known historical range, or that the GST arithmetic is valid.

Without attestations:

| Claim | How it is currently verified | Gap |
|---|---|---|
| "This transaction is legally compliant" | Ad-hoc test, reviewer memory | No persistent, machine-queryable record |
| "This amount is within historical bounds" | `VendorConstraintSet::evaluate()` | Result is discarded after the pipeline stage |
| "This invoice GST arithmetic is valid" | `InvoiceConstraintSolver::verify()` | Result is not recorded against the type |
| "This type transition is formally safe" | Kani proofs in CI | Proofs are not linked to runtime invariant records |

---

## 3. Concept: The Attestation System

### 3.1 `#[attested]` — A Lint/Proc-Macro Gate

A proc-macro attribute `#[attested("predicate")]` applied to a type or function declares that the annotated item has passed the named invariant. The compiler (via a custom lint or `#[proc_macro_attribute]`) enforces that:

1. The type/function provides a corresponding `Attestation` impl.
2. The `Attestation` impl supplies at least one **Z3 predicate** (logical) or **Kasuari constraint** (numerical bound), or both.
3. A Kani harness exists that formally verifies the attestation.

```rust
#[attested("gst_arithmetic_valid")]
pub struct InvoiceVerification { ... }

impl Attested for InvoiceVerification {
    fn attestation() -> AttestationSpec {
        AttestationSpec::new("gst_arithmetic_valid")
            .z3_predicate("total = subtotal + gst")
            .kasuari_bound("gst", 0.0, subtotal * 0.12)  // 10% ± 20% tolerance
            .kani_proof_module("kani_proofs::invoice_arithmetic")
    }
}
```

The custom lint `deny(unattested_formal_claim)` ensures that any type decorated with a formal claim annotation but missing an `Attested` impl is a compile error.

### 3.2 The Invariant Ledger

Every time a type with an `#[attested]` annotation is instantiated (or a function with an attestation is called), the runtime records an `InvariantEntry` in an append-only in-process ledger:

```rust
pub struct InvariantEntry {
    /// Which invariant was claimed.
    pub invariant: String,
    /// The type or function that made the claim.
    pub source: String,
    /// Timestamp of the claim.
    pub at: chrono::DateTime<chrono::Utc>,
    /// Z3 verification result (if applicable).
    pub z3_result: Option<Z3Result>,
    /// Kasuari constraint evaluation (if applicable).
    pub constraint_eval: Option<ConstraintEvaluation>,
    /// Kani proof status from last CI run (static, not runtime).
    pub kani_proof: KaniProofStatus,
    /// Content hash of the claim (Blake3 over source + invariant + timestamp).
    pub entry_id: String,
}
```

The ledger is an in-process `Vec<InvariantEntry>` that is flushed to the `_invariants` sheet in the Excel workbook on export. CPAs can inspect the full invariant record alongside transactions.

### 3.3 Self-Extensible: The Invariant Registry

The system is self-extensible because invariants are **runtime-registered**, not hardcoded. Any crate in the workspace can register a new invariant by calling:

```rust
InvariantRegistry::global().register(
    "my_new_invariant",
    InvariantSpec {
        description: "...",
        z3_formula: Some("..."),
        kasuari_spec: Some(KasuariSpec { ... }),
        kani_module: Some("kani_proofs::my_new"),
    }
);
```

New types or Rhai rules can declare new invariants without recompiling the core crate, as long as the `AttestationSpec` they provide matches a registered `InvariantSpec`. The Rhai rule engine can declare attestations at rule load time, making classification rules first-class participants in the invariant ledger.

### 3.4 Dual-Solver Verification Flow

```
Type instantiation
      │
      ▼
InvariantLedger::record(entry)
      │
      ├──► Z3Solver.check(z3_predicate, facts)   → Z3Result::Satisfied | Violated | Unknown
      │
      ├──► KasuariSolver.evaluate(constraints)    → ConstraintEvaluation { required_pass, ... }
      │
      └──► InvariantEntry { z3_result, constraint_eval, kani_proof: KaniProofStatus::Verified }
                │
                ▼
          workbook._invariants sheet
          (CPA-readable, Blake3-chained, append-only)
```

Both solvers must agree: a claim that passes Z3 but fails Kasuari (or vice versa) is recorded as a `PartialAttestation` with an advisory issue emitted to the pipeline.

### 3.5 Kani Integration

Each registered invariant has an optional `kani_module` pointer. The `kani.yml` CI workflow collects all registered invariants and verifies that a Kani harness exists with the matching module path. A missing harness is a CI failure, not just a warning.

At runtime, the `kani_proof` field in `InvariantEntry` is populated from a build-time manifest generated by the Kani workflow (a JSON file baked in at compile time), so the workbook ledger records which Kani proofs were passing at the build that produced the export.

---

## 4. Rust Syntax Hook: The Custom Lint

The mechanism for enforcing attestations at compile time is a custom lint implemented as a `rustc` plugin or `proc_macro` crate (`ledger-attest`):

```rust
// ledger-attest/src/lib.rs
#[proc_macro_attribute]
pub fn attested(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. Parse the invariant name from attr
    // 2. Inject a compile-time static assertion that `Attested` is implemented
    // 3. Inject registration code into the type's module init
    // 4. Emit a deny(unattested_formal_claim) if the impl is missing
}
```

The lint fires at `cargo build` and `cargo clippy`, not just at test time. A type that claims a formal property without providing a verifiable attestation is rejected before the binary is linked.

---

## 5. Relation to Existing PRDs

| PRD | Connection |
|---|---|
| PRD-6 | Self_cell and trait_variant deferred; this PRD defines the *semantic* layer above the structural type system |
| PRD-7 | `CommitGate`, `verify_legal()`, and `check_constraints()` are candidates for `#[attested]` decoration |
| PRD-8 | Kani harnesses become the formal backing for `kani_proof: KaniProofStatus::Verified` in each ledger entry |

---

## 6. Scope Gates (Not In This Document)

- **Implementation** — this document is a concept definition; implementation is gated on PRD-8 (Kani) being stable in CI.
- **Rhai attestation DSL** — out of scope for the first implementation; Rhai rules would declare attestations via a string-based API, not the proc-macro.
- **Cross-version invariant replay** — the ledger records entries but does not yet support replaying old entries against new invariant specs (schema migration is deferred).

---

## 7. Success Criteria (When This Is Done)

1. `cargo build` fails if a type decorated with `#[attested("X")]` lacks an `Attested` impl for `"X"`.
2. Every `InvoiceVerification`, `PipelineState<Classified>`, and `CommitGate::Approved` instance produces an `InvariantEntry` in the in-process ledger.
3. The workbook `_invariants` sheet is populated on export with one row per invariant claim, including `z3_result`, `constraint_score`, `kani_proof`, and `entry_id`.
4. A new invariant can be registered by any crate without modifying `ledger-core`.
5. `kani.yml` CI verifies that every registered invariant with a `kani_module` pointer has a passing Kani harness.
