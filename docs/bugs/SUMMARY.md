# l3dg3rr MCP Server — Interface Test Summary

**Date**: 2026-04-16
**Binary**: `target/debug/ledgerr-mcp-server`
**Protocol**: MCP 2025-11-25
**Commit**: `faba56d`
**Tools advertised**: 28 | **Tools probed**: 27

## Bug Registry

| ID | Severity | Tool | Issue | Status |
|----|----------|------|-------|--------|
| BUG-001 | P1 | `proxy_docling_ingest_pdf` | Schema required `source_ref`; impl required `pdf_path`+`journal_path`+`workbook_path` | ✅ Fixed — schema updated to match impl; `extracted_rows` made truly optional |
| BUG-002 | P1 | `l3dg3rr_ontology_export_snapshot` | Schema advertised no params; impl required `ontology_path` | ✅ Fixed — schema updated; handler routed through `TurboLedgerService` |
| BUG-003 | P0 | ALL tools (27) | All tools returned `"type": "json"` content blocks — not a valid MCP 2025-11-25 content type; rejected by spec-compliant clients | ✅ Fixed — all content blocks converted to `"type": "text"` with JSON serialized as string; `text_content()` helper centralises the pattern |

## Pass Results by Tool Group

| Group | Tools | Result |
|-------|-------|--------|
| Core | `list_accounts`, `get_pipeline_status`, `hsm_status`, `query_audit_log`, `proxy_rustledger_ingest_statement_rows` | ✅ All pass |
| Reconciliation | `validate_reconciliation`, `reconcile_postings`, `commit_guarded` | ✅ All pass (balanced/unbalanced/edge cases) |
| HSM | `hsm_transition`, `hsm_resume`, `hsm_status` | ✅ Respond correctly (blocked transitions expected by lifecycle guard) |
| Events | `event_history`, `event_replay` | ✅ All pass |
| Classification | `classify_ingested`, `classify_transaction`, `reconcile_excel_classification`, `query_flags` | ✅ All pass |
| Tax | `tax_assist`, `tax_evidence_chain`, `tax_ambiguity_review`, `get_schedule_summary`, `export_cpa_workbook` | ✅ All pass |
| Ontology (read) | `ontology_query_path`, `ontology_upsert_entities`, `ontology_upsert_edges` | ✅ Pass |
| Ontology (export) | `ontology_export_snapshot` | ✅ Pass (BUG-002 fixed) |
| Docling proxy | `proxy_docling_ingest_pdf` | ✅ Pass (BUG-001 fixed) |
| All tools (content type) | all 27 tools | ✅ Pass (BUG-003 fixed) |

## Protocol Compliance

- Initialize/initialized handshake: ✅
- `tools/list` returns complete schema: ✅
- JSON-RPC error codes: ✅ (`-32601` for unknown method)
- Response envelope (`result.content[].type`, `isError`): ✅

## Input Validation Coverage

| Input class | Behavior |
|-------------|----------|
| Valid decimal string `"-42.11"` | ✅ Accepted |
| Non-numeric decimal | ✅ Rejected with `InvalidInput` |
| ISO 8601 date `"2024-01-15"` | ✅ Accepted |
| Slash-delimited date `"2024/01/15"` | ✅ Rejected |
| Valid schedule enum | ✅ Accepted |
| Invalid enum value | ✅ Rejected with descriptive message |
| Missing required field | ✅ Rejected with `"missing or invalid \`fieldname\`"` |
| Unknown tool name | ✅ Returns `unknown tool: <name>` with `isError:true` |

## Recommendations

1. **Systematic schema audit**: BUG-001 and BUG-002 reveal a pattern — schemas may have been written against an earlier or future API shape. Audit all 28 tool schemas against their `parse_*` functions.
2. **Schema generation**: Derive `inputSchema` from the same struct that drives parsing (e.g., `schemars`) to prevent drift.
3. **CI gate**: Add a test that calls every tool with schema-compliant minimal args and asserts `isError:false` (or an expected domain error, not a parse error).
