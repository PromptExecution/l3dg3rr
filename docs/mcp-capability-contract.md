# MCP Capability Contract (Operator View)

This document describes what is currently exposed through:
- MCP transport (`ledgerr-mcp-server`)
- local CLI/runtime entrypoints (`Justfile`, Python launcher)
- internal Rust service API (`TurboLedgerTools`)

It is intentionally concrete and code-aligned.

## 1) MCP Surface (What an Agent Can Call)

MCP boundary:
- Methods: `initialize`, `tools/list`, `tools/call`
- Server: [crates/ledgerr-mcp/src/bin/ledgerr-mcp-server.rs](crates/ledgerr-mcp/src/bin/ledgerr-mcp-server.rs#L8)
- Tool catalog: [crates/ledgerr-mcp/src/mcp_adapter.rs](crates/ledgerr-mcp/src/mcp_adapter.rs#L30)

### Tool Matrix

| MCP tool name | Primary purpose | Required arguments | Backing service/API |
|---|---|---|---|
| `l3dg3rr_list_accounts` | enumerate manifest account ids for tool planning and routing | none | `TurboLedgerService::list_accounts_tool` |
| `l3dg3rr_get_raw_context` | fetch stored raw context bytes by reference path | `rkyv_ref` | `TurboLedgerTools::get_raw_context` |
| `l3dg3rr_get_pipeline_status` | readiness hint for workflow start | none | `get_pipeline_status` adapter helper |
| `proxy_docling_ingest_pdf` | ingest rows extracted from PDF, persist raw bytes fallback, emit canonical rows + tx ids | `pdf_path`, `journal_path`, `workbook_path`, `extracted_rows` | `TurboLedgerTools::ingest_pdf` |
| `proxy_rustledger_ingest_statement_rows` | ingest normalized rows through rustledger-compatible path | `journal_path`, `workbook_path`, `rows` | `TurboLedgerTools::ingest_statement_rows` |
| `l3dg3rr_ontology_query_path` | query ontology path traversal | `ontology_path`, `from_entity_id` | `TurboLedgerService::ontology_query_path_tool` |
| `l3dg3rr_ontology_export_snapshot` | export deterministic ontology snapshot | `ontology_path` | `OntologyStore::load` + snapshot |
| `l3dg3rr_validate_reconciliation` | validation stage checks | `source_total`, `extracted_total`, `posting_amounts` | `validate_stage` |
| `l3dg3rr_reconcile_postings` | reconcile stage checks | `source_total`, `extracted_total`, `posting_amounts` | `reconcile_stage` |
| `l3dg3rr_commit_guarded` | commit gate for balanced/ready stage | `source_total`, `extracted_total`, `posting_amounts` | `commit_stage` |
| `l3dg3rr_hsm_transition` | deterministic lifecycle transition | `target_state`, `target_substate` | `hsm_transition_tool` |
| `l3dg3rr_hsm_status` | current state + concise hints | none | `hsm_status_tool` |
| `l3dg3rr_hsm_resume` | resume from checkpoint marker | `state_marker` | `hsm_resume_tool` |
| `l3dg3rr_event_history` | filtered append-only event query | optional `tx_id`, `document_ref`, `time_start`, `time_end` | `event_history` |
| `l3dg3rr_event_replay` | reconstruct lifecycle state from events | optional `tx_id`, `document_ref` | `replay_lifecycle` |
| `l3dg3rr_tax_assist` | tax summary/schedule guidance gated by reconciliation | `ontology_path`, `from_entity_id`, `reconciliation` | `tax_assist_tool` |
| `l3dg3rr_tax_evidence_chain` | source->events->current_state chain | `ontology_path`, `from_entity_id` | `tax_evidence_chain_tool` |
| `l3dg3rr_tax_ambiguity_review` | ambiguity review queue payload | `ontology_path`, `from_entity_id`, `reconciliation` | `tax_ambiguity_review_tool` |

Input parsing and validation live in:
[crates/ledgerr-mcp/src/mcp_adapter.rs](crates/ledgerr-mcp/src/mcp_adapter.rs#L160)

## 2) Internal Rust Service API (Wider Than MCP)

Canonical trait:
[TurboLedgerTools in crates/ledgerr-mcp/src/lib.rs](crates/ledgerr-mcp/src/lib.rs#L275)

Important distinction:
- Some service capabilities exist but are not exposed as MCP tools yet.
- Examples currently service-only: `run_rhai_rule`, `classify_ingested`, `query_flags`, `classify_transaction`, `query_audit_log`, `export_cpa_workbook`, `get_schedule_summary`, ontology upserts.

This is the current API layering:
1. `ledgerr-mcp-server` (stdio transport)
2. `mcp_adapter` (argument parsing + envelope shaping)
3. `TurboLedgerService` (domain logic, guardrails, state/event/HSM ops)
4. `ledger-core` (ingest, filename validation, classification primitives)

## 3) CLI / Runtime Entry Points

Operational CLI helpers are lifecycle-oriented, not business-domain verbs:
- [Justfile](Justfile#L3)
  - `just mcp-build`
  - `just mcp-start`
  - `just mcp-start-release`
  - `just mcp-stop`
  - `just mcp-e2e`
- Python launcher:
  [plugins/l3dg3rr-plugin-create/python/src/l3dg3rr_mcp_launcher/__main__.py](plugins/l3dg3rr-plugin-create/python/src/l3dg3rr_mcp_launcher/__main__.py#L35)
  - `--mode cargo|binary|docker`

## 4) Functional Relationships (Contrived Sample)

Scenario: an MCP client ingests one synthetic statement row, validates reconciliation, checks lifecycle state, then asks for tax evidence.

### Step A: initialize and discover tools
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"clientInfo":{"name":"demo","version":"0.1.0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
```

### Step B: ingest a row through rustledger proxy
```json
{
  "jsonrpc":"2.0",
  "id":3,
  "method":"tools/call",
  "params":{
    "name":"proxy_rustledger_ingest_statement_rows",
    "arguments":{
      "journal_path":"/tmp/demo.beancount",
      "workbook_path":"/tmp/demo.xlsx",
      "rows":[
        {
          "account_id":"WF-BH-CHK",
          "date":"2023-01-05",
          "amount":"-42.50",
          "description":"Coffee Beans",
          "source_ref":"/tmp/raw/wf-2023-01.rkyv"
        }
      ]
    }
  }
}
```

Relationship here:
- MCP tool -> adapter normalization/provenance (`provider=rustledger`) ->
  `TurboLedgerService::ingest_statement_rows` ->
  `ledger-core` ingest/idempotency.

### Step C: run reconciliation gate
```json
{
  "jsonrpc":"2.0",
  "id":4,
  "method":"tools/call",
  "params":{
    "name":"l3dg3rr_commit_guarded",
    "arguments":{
      "source_total":"42.50",
      "extracted_total":"42.50",
      "posting_amounts":["-42.50","42.50"]
    }
  }
}
```

Relationship here:
- Reconciliation status gates downstream tax-oriented operations.
- Blocked states return deterministic `isError` envelopes with typed reasons.

### Step D: inspect HSM status and event replay
```json
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"l3dg3rr_hsm_status","arguments":{}}}
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"l3dg3rr_event_replay","arguments":{"document_ref":"/tmp/raw/wf-2023-01.rkyv"}}}
```

Relationship here:
- HSM provides concise lifecycle hints for small-model agent orchestration.
- Event replay reconstructs deterministically computable lifecycle state from append-only history.

### Step E: ask for tax evidence chain
```json
{
  "jsonrpc":"2.0",
  "id":7,
  "method":"tools/call",
  "params":{
    "name":"l3dg3rr_tax_evidence_chain",
    "arguments":{
      "ontology_path":"/tmp/ontology.json",
      "from_entity_id":"WF-BH-CHK",
      "document_ref":"/tmp/raw/wf-2023-01.rkyv"
    }
  }
}
```

Relationship here:
- Ontology path + event history + replay projection are merged into one evidence payload:
  `source -> events -> current_state (+ ambiguity)`.

## 5) Practical Gaps To Keep In Mind

1. Classification/audit/schedule methods are implemented in service API but not yet surfaced over MCP.
2. Ontology upsert methods exist in service API but are not MCP-exposed (`ontology_upsert_entities_tool`, `ontology_upsert_edges_tool`).
3. CLI is mostly runtime/start-stop orchestration; domain operations are MCP-first.

## 6) Proposal: Next MCP Exposure Gaps (Mission-Aligned)

The following are high-value MCP additions for an AI-first financial document knowledge workflow.

| Priority | Proposed MCP tool | Existing backend method | Why it matters |
|---|---|---|---|
| P0 | `l3dg3rr_classify_ingested` | `classify_ingested` | turns ingestion output into review-queue-ready classifications without direct code execution |
| P0 | `l3dg3rr_query_flags` | `query_flags` | gives agents deterministic open/resolved review queues by year |
| P0 | `l3dg3rr_query_audit_log` | `query_audit_log` | provides explainability and mutation trace required for CPA/agent trust |
| P1 | `l3dg3rr_classify_transaction` | `classify_transaction` | lets agents apply explicit corrections through guarded, auditable mutations |
| P1 | `l3dg3rr_reconcile_excel_classification` | `reconcile_excel_classification` | syncs manual Excel edits back into deterministic audit/event chain |
| P1 | `l3dg3rr_get_schedule_summary` | `get_schedule_summary` | provides compact tax summary materialization for downstream tax-agent orchestration |
| P2 | `l3dg3rr_export_cpa_workbook` | `export_cpa_workbook` | gives a single MCP-triggered handoff artifact for CPA review cycles |
| P2 | `l3dg3rr_ontology_upsert_entities` / `l3dg3rr_ontology_upsert_edges` | `ontology_upsert_entities_tool` / `ontology_upsert_edges_tool` | enables agent-driven ontology curation instead of read-only graph usage |
