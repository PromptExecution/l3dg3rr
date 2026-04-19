# Workflow Capabilities Gap Analysis

**Date:** 2026-04-19
**Context:** Agent orchestration review for l3dg3rr tax-ledger system

## Executive Summary

The system exposes 8 MCP tool families (`ledgerr_documents`, `ledgerr_review`, `ledgerr_reconciliation`, `ledgerr_workflow`, `ledgerr_audit`, `ledgerr_tax`, `ledgerr_ontology`, `ledgerr_xero`) providing discrete operations. However, there is **no capability to define the "shape" and "workflow" of data** — agents cannot:

1. **Describe required schema** — define "what is success" vs "fail" for composed operations (e.g., "bank statement import → associate to account")
2. **Compose multi-step workflows** — perform a series of steps/chores, including loops and conditional branching
3. **Execute deterministic validation** — validate truth with tools rather than relying solely on LLM non-determinism

---

## Existing Capabilities (What's There)

### Shape/Schema Capabilities

| Capability | Location | Purpose |
|-----------|----------|---------|
| MCP contract | `crates/ledgerr-mcp/src/contract.rs` | 7 top-level tools with action enums, JSON schemas via schemars |
| Filename validation | `validate_source_filename` | Enforces `VENDOR--ACCOUNT--YYYY-MM--DOCTYPE.ext` |
| Content-hash IDs | `blake3` over account/date/amount/desc | Deterministic transaction IDs for idempotent ingest |
| Decimal validation | `rust_decimal::Decimal` | Money semantics (no `f64` in domain structs) |
| Rhai rules | `ledger-core` | Runtime-editable classification/flag rules |
| CPA workbook schema | `REQUIRED_SHEETS` | Fixed sheets, validation dropdowns, schedule summaries |

### Workflow Capabilities

| Capability | Location | Purpose |
|-----------|----------|---------|
| HSM state machine | Phase 16 (v1.1) | Lifecycle: `ingest → normalize → validate → reconcile → commit → summarize` |
| MCP actions | Each tool family | Discrete operations: `ingest_pdf`, `commit`, `resume`, etc. |
| Reconciliation guardrails | `ledgerr_reconciliation::commit` | Totals validation before commit |
| Audit log | `ledgerr_audit` | Append-only event history and replay |
| Rig integration | `crates/ledgerr-host/src/chat.rs` | LLM chat completion via `rig` crate |

---

## Gaps (What's Missing)

### 1. No Workflow Definition Language

- No way to compose a series of steps/chores into a higher-level task
- No DSL or schema for defining "task templates" like "ingest monthly statement"
- HSM is a **single-document linear state machine** — cannot express iteration over a queue of documents

### 2. No Loops or Conditional Branching

- Only blocked/unblocked responses, not expressive control flow
- Cannot express: "for each statement in queue: ingest → classify → reconcile → on failure: flag → continue"
- No `while`, `if/else`, `map/filter` semantics for workflow composition

### 3. No Success/Failure Schema Definition

- Cannot define "what is success" vs "fail" for composed operations
- Example: "bank statement import → associate to account" — what are the success criteria?
  - All rows extracted? Linked to account? Classified? Balances reconciled?
- No way to declare exit conditions for a task

### 4. No Deterministic Validation Tools

- Classification relies on LLM non-determinism
- No tool to **deterministically validate** that classification is correct
- Agents must trust LLM output rather than verify with rules/queries

### 5. No Task/Chore Abstraction

- Just individual tool calls, no composed operations
- Agents must manually figure out which sequence of tools to call
- No "macro" or "recipe" capability

### 6. Rig is Chat-Only, Not Agent Orchestration

- `rig` crate used only for LLM completion (`send_chat_message`)
- No agent runtime (no LangGraph/LangChain-style orchestration)
- No tool-use loop, no planning, no reflection

---

## Recommendations

### Short-Term

1. **Add a `TaskDefinition` schema** to contract — allow declaring input/output shape for a composed operation
2. **Add `WorkflowExecutor` capability** — accept a DAG of steps, execute with checkpointing, return success/failure per step
3. **Add deterministic validation tools** — e.g., `query_flags` with rule-based filters, not just LLM classification

### Medium-Term

4. **Introduce workflow templates** — JSON/YAML definition of common sequences (monthly statement ingest, quarterly tax prep)
5. **Add loop semantics to HSM** — allow "resume with next document in queue" rather than single-file flow
6. **Add conditional branching** — "if classification confidence < threshold → flag for review"

### Long-Term

7. **Agent runtime integration** — consider langchain-rs or custom tool-use loop with reflection
8. **Task success criteria DSL** — declarative "what good looks like" for each task type

---

## Notes

- This analysis does not recommend abandoning the MCP tool surface — rather, layer workflow orchestration **above** the existing tools
- The HSM provides solid lifecycle foundation; the gap is **multi-document orchestration** and **task-level success criteria**
- CPA workbook contract is well-defined; task-level success should align with workbook validation