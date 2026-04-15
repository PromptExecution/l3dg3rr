---
name: ledgerr-reconcile
description: Use this skill to run the validate → reconcile → commit guardrail sequence in l3dg3rr. Covers posting balance checks, HSM lifecycle transitions, CPA workbook export, and schedule summaries. Required before any tax output is considered final.
---

# ledgerr-reconcile

## The Guardrail Sequence

Commit is blocked until validate and reconcile pass. Never skip steps.

```
validate → reconcile → commit
```

### Step 1 — Validate

```json
{
  "name": "l3dg3rr_validate_reconciliation",
  "arguments": {
    "source_total": "-1842.50",
    "extracted_total": "-1842.50",
    "posting_amounts": ["-42.00", "-800.50", "-1000.00"]
  }
}
```

`source_total` = total from the original PDF/statement.
`extracted_total` = sum of all ingested rows.
`posting_amounts` = individual line amounts (must balance to `extracted_total`).

On mismatch: returns `isError: false` but with `status: "blocked"` and `reason` keys (`totals_mismatch`, `imbalance_postings`). Fix the discrepancy before proceeding.

### Step 2 — Reconcile

Same arguments as validate. Transitions internal state to `reconciled` if clean.

```json
{ "name": "l3dg3rr_reconcile_postings", "arguments": { ... same ... } }
```

### Step 3 — Commit (guarded)

Only succeeds if both prior stages passed. Returns `commit_ready: true` on success.

```json
{ "name": "l3dg3rr_commit_guarded", "arguments": { ... same ... } }
```

## HSM Lifecycle

The session state machine tracks the overall pipeline phase. Check current state:

```json
{ "name": "l3dg3rr_hsm_status", "arguments": {} }
```

Transition to next state:
```json
{ "name": "l3dg3rr_hsm_transition", "arguments": { "target_state": "reconciled", "target_substate": "clean" } }
```

Resume from last checkpoint if a session was interrupted:
```json
{ "name": "l3dg3rr_hsm_resume", "arguments": { "state_marker": "checkpoint-id-here" } }
```

## Tax Outputs (after commit)

### Schedule summary
```json
{ "name": "l3dg3rr_get_schedule_summary", "arguments": { "year": 2023, "schedule": "ScheduleC" } }
```
Valid schedules: `ScheduleC`, `ScheduleD`, `ScheduleE`, `Fbar`

### CPA workbook export
```json
{ "name": "l3dg3rr_export_cpa_workbook", "arguments": { "workbook_path": "/data/tax-2023-final.xlsx" } }
```

Returns `sheets_written` count and writes the `.xlsx` to the given path.

### Tax assist (full derivation)
```json
{
  "name": "l3dg3rr_tax_assist",
  "arguments": {
    "ontology_path": "/data/ontology.json",
    "from_entity_id": "WF-BH-CHK",
    "reconciliation": { "source_total": "-1842.50", "extracted_total": "-1842.50", "posting_amounts": [...] }
  }
}
```

## Blocked State Diagnostics

If any tool returns `status: "blocked"`, read `reason` and `blockers` fields. Common reasons:
- `totals_mismatch` — source_total ≠ extracted_total
- `imbalance_postings` — posting_amounts don't sum to extracted_total
- `reconciliation_not_ready` — validate/reconcile not yet passed
