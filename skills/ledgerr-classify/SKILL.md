---
name: ledgerr-classify
description: Use this skill to classify transactions in the l3dg3rr tax ledger using Rhai rules, review the flag queue, and apply manual per-transaction corrections via MCP. Covers rule authoring, batch classification, flag querying, and audit trail.
---

# ledgerr-classify

## Classification Flow

### Step 1 — Write a Rhai rule file

Rules live in a `.rhai` file. The `classify` function receives a transaction map and must return a result map:

```rhai
fn classify(tx) {
    let desc = tx["description"];
    let category = if desc.contains("Coffee") { "Meals" }
                   else if desc.contains("AWS") { "Software" }
                   else { "Uncategorized" };
    let confidence = if category == "Uncategorized" { 0.33 } else { 0.92 };
    #{
        category: category,
        confidence: confidence,   // 0.0–1.0; values below review_threshold → flagged
        review: confidence < 0.80,
        reason: "auto-rule-v1"
    }
}
```

Valid categories (from `TaxCategory` enum): `Meals`, `Travel`, `Software`, `OfficeSupplies`, `Utilities`, `Salary`, `Uncategorized`, and others — check `mcp_adapter.rs` for full list.

### Step 2 — Run batch classification

```json
{
  "name": "l3dg3rr_classify_ingested",
  "arguments": {
    "rule_file": "/path/to/classify.rhai",
    "review_threshold": 0.80
  }
}
```

Response includes `classifications` array. Each entry has `tx_id`, `category`, `confidence`, `review` flag.

### Step 3 — Check the review queue

```json
{ "name": "l3dg3rr_query_flags", "arguments": { "year": 2023, "status": "open" } }
```

Returns `flags` array of transactions needing human review.

### Step 4 — Manual correction

For individual transactions that need a different category:

```json
{
  "name": "l3dg3rr_classify_transaction",
  "arguments": {
    "tx_id": "blake3-hash-here",
    "category": "Travel",
    "confidence": "0.95",
    "actor": "agent",
    "note": "override: vendor is airline"
  }
}
```

Every manual classification is appended to the audit log automatically.

### Step 5 — Verify audit trail

```json
{ "name": "l3dg3rr_query_audit_log", "arguments": {} }
```

## Confidence Invariant

`confidence` must be a decimal string in `[0.0, 1.0]`. Values stored as `rust_decimal::Decimal` — no float precision loss. Do not pass `f64` values; pass string representations like `"0.92"`.

## Excel Reconciliation

If a CPA edits categories in the Excel workbook, sync back via:

```json
{
  "name": "l3dg3rr_reconcile_excel_classification",
  "arguments": { "tx_id": "...", "category": "...", "confidence": "1.00", "actor": "cpa", "note": "CPA override" }
}
```
