---
name: ledgerr-ingest
description: Use this skill to ingest PDF financial statements into the l3dg3rr tax ledger via MCP. Covers filename contract, Docling extraction, row ingestion, idempotency verification, and raw context sidecar handling.
---

# ledgerr-ingest

## Filename Contract (required before any ingest)

Every source file must follow: `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE.ext`

Examples:
- `WF--BH-CHK--2023-01--statement.pdf`  ✓
- `chase--savings--2023-06--statement.pdf`  ✓
- `statement_jan2023.pdf`  ✗ — will be rejected by preflight

Validate first:
```json
{ "name": "proxy_docling_ingest_pdf", "arguments": { "source_ref": "WF--BH-CHK--2023-01--statement.pdf" } }
```

## Ingest Flow

### Step 1 — Docling extraction (PDF → rows)
Call `proxy_docling_ingest_pdf` with:
- `source_ref`: filename following contract above
- `raw_context_bytes`: PDF bytes as integer array (optional if already stored)
- `extracted_rows`: pre-extracted transactions (optional if Docling handles it)

Each row needs: `account_id`, `date` (YYYY-MM-DD), `amount` (decimal string e.g. `"-42.11"`), `description`, `source_ref`.

### Step 2 — Verify idempotency
Re-ingesting the same rows must return `inserted_count: 0`. If it returns > 0 on second call, something is wrong with the content-hash IDs.

```json
{ "name": "l3dg3rr_list_accounts", "arguments": {} }
```
Check the account appears in the list.

### Step 3 — Retrieve raw context
```json
{ "name": "l3dg3rr_get_raw_context", "arguments": { "path": "path/to/stmt.rkyv" } }
```

## Transaction ID Format

IDs are Blake3 content hashes over `account_id + date + amount + description`. Same input always yields the same ID — this is the dedup guarantee.

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `invalid_filename` | Filename doesn't match contract | Rename file to VENDOR--ACCOUNT--YYYY-MM--DOCTYPE |
| `inserted_count: 0` on first ingest | Rows already exist (good — idempotent) | Check if prior session already ingested |
| `missing raw_context_bytes` | First ingest without bytes, no prior sidecar | Pass `raw_context_bytes` |
