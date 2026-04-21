# Ontology & Type Mesh

The ontology layer describes relationships between documents, accounts, transactions, tax categories, evidence references, workflow tags, and Xero entities. The type mesh describes how Rust values move through pipeline stages without losing identity or auditability.

## Ontology Role

Ontology operations are exposed through `ledgerr_ontology`:

- `query_path`: follow relationships between entities
- `export_snapshot`: produce a serializable graph snapshot
- `upsert_entities`: add or update typed entities
- `upsert_edges`: add or update relationships

```rhai
fn document_record() -> ontology_entity
fn account_record() -> ontology_entity
fn transaction() -> ontology_entity
fn xero_contact() -> ontology_entity
fn ontology_entity() -> evidence_path
fn evidence_path() -> tax_assist
```

## Type Mesh Role

The type mesh answers a different question: which Rust values are compatible between stages, and what bridge is responsible for transforming them?

Examples:

- `TransactionInput` becomes `IngestedTransaction`, `JournalTransaction`, and `SampleTransaction`.
- `ClassificationOutcome` becomes `ClassifiedTransaction` and review flags.
- `LegalRule + TransactionFacts` becomes `Z3Result`.
- `VendorConstraintSet` becomes `ConstraintEvaluation`.

## Related Tables

The detailed tables are generated:

- [Type Compatibility](./type-compatibility.md)
- [Concept Affinity](./concept-affinity.md)

Update generated tables through the repo tooling rather than hand-editing those chapters.
