# Audit Playbook

This playbook is the PRD-4 end-to-end operator path. It proves that a sample
statement row, workbook projection, ontology snapshot, audit events, and visual
graph all carry the same transaction identity.

```rhai
fn sample_statement() -> ingest_rows
fn ingest_rows() -> classify_transactions
fn classify_transactions() -> phi4_edge_proposals
fn phi4_edge_proposals() -> operator_review
fn operator_review() -> workbook_export
fn workbook_export() -> evidence_chain
fn evidence_chain() -> visual_audit_graph
fn visual_audit_graph() -> cpa_review
```

## Runnable Paths

Basic deterministic path:

```sh
just mcp-cli-basic
```

Blocked diagnostic path:

```sh
just mcp-cli-spinning-wheels
```

Host playbook path with deterministic Phi-4 fallback:

```sh
just host-playbook-window
```

Host playbook path with local Phi-4 model assets:

```sh
just host-playbook-window-phi4
```

## Related Chapters

- [Capability Map](./capability-map.md)
- [MCP Surface](./mcp-surface.md)
- [Ontology & Type Mesh](./ontology-type-mesh.md)
- [Workbook & Audit](./workbook-audit.md)
- [Visualization](./visualize.md)
