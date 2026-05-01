# PRD-5: Operator Simplification and Hidden Layer Grouping

**Status**: Draft - present-state synthesis and product simplification plan  
**Target capabilities**: simplified operator host, progressive disclosure, grouped diagnostics, selectable local model providers, audit-first bookkeeping flow  
**Author**: Codex  
**Date**: 2026-05-01

---

## Problem

`l3dg3rr` now has enough capability surface to be useful, but the product is at risk of showing its implementation structure instead of the operator's work structure.

The current system includes deterministic ledger primitives, MCP capability families, mdBook visual playbooks, a Slint host window, an emerging Tauri surface, local Phi-4 fallback, optional GGUF-backed Phi-4, Windows AI / Foundry Local setup, cloud OpenAI-compatible settings, ontology state, Xero direction, and generated MCP docs.

Those are valid layers, but they are not all operator concepts. A demo or working session should not require the operator to understand Rust crates, feature flags, dynamic Foundry ports, generated schemas, MCP aliasing, Slint versus Tauri, or model backend mechanics before they can answer one question:

> "What financial evidence is ready, what needs review, and what can I safely ask the local agent to do next?"

This PRD uses an IDEO-style human-centered simplification pass to group hidden layers behind task-oriented surfaces while preserving auditability and developer diagnostic access.

## Present Status

### Implemented Or In Flight

- **Bookkeeping core**: deterministic transaction IDs, workbook-oriented exports, classification/edit/audit primitives.
- **MCP surface**: compact advertised tool families under `ledgerr_*`, with `ledgerr_xero` now visible and legacy aliases treated as compatibility-only.
- **Ontology direction**: `ledger-core::ontology` is the canonical primitive layer; MCP legacy storage shape remains backward compatible.
- **Visual docs**: mdBook playbook with Rhai diagram DSL, Mermaid reference view, and isometric renderer.
- **Host UI**: Slint desktop window with chat, logs, settings, docs playbook, internal Phi-4 selector, cloud selector, and Windows AI / Foundry Local selector.
- **Local model paths**:
  - deterministic internal Phi-4-compatible fallback,
  - optional local GGUF Phi-4 Mini backend through host features,
  - selectable Windows AI / Foundry Local provider using `phi-4-mini`,
  - cloud OpenAI-compatible endpoint settings.
- **Windows setup recipes**: `just windows-ai-install`, `just windows-ai-setup`, `just windows-ai-status`, and `just windows-ai-smoke`.
- **Tauri surface**: workspace member exists with parallel chat/settings/docs controls, but it should be treated as an experimental shell until product policy chooses a primary desktop host.

### Known Friction

- The operator sees too many provider mechanics: endpoint URL, API key placeholder, local fallback, Foundry Local install state, cloud model, and logs all compete for attention.
- "Internal Phi-4", "Windows AI / Foundry Local", and "Cloud / OpenAI" are technically clear but not yet framed around privacy, readiness, and expected setup effort.
- Diagnostics are valuable but too prominent. Transport payloads, backend status, docs status, and review diffs should be grouped by intent.
- Slint and Tauri duplicate UX concepts. This is useful for exploration but confusing as a product story.
- CI now needs Linux desktop dependencies because Tauri is a workspace member; this is an implementation concern that should not leak into operator docs.
- `foundry` may not be installed on Windows, so the Windows AI path needs obvious readiness and recovery messaging.

---

## IDEO Framing

### Desirability

Operators want a calm local control plane for bookkeeping work:

- know which source evidence has been ingested,
- see which transactions are classified or blocked,
- ask a local model for bounded help,
- approve or reject model suggestions,
- export CPA-auditable workbook artifacts,
- inspect why a result exists.

They do not want to configure model transports during normal bookkeeping work.

### Feasibility

The system already has the needed technical primitives:

- `ChatSettings` can switch OpenAI-compatible providers without changing model-call code.
- `internal_openai.rs` owns local demo endpoints and docs routes.
- `Justfile` owns repeatable setup and validation recipes.
- mdBook already renders operator playbooks.
- MCP tools are already collapsed into top-level capability families.

The next step is grouping and progressive disclosure, not a new architecture.

### Viability

The simplified product should preserve the local-first value proposition:

- private data stays local by default,
- cloud model use is explicit,
- Windows AI is optional and selectable,
- workbook remains the accountant-facing artifact,
- diagnostics remain available for support and audit.

---

## Product Principle

Show the operator workflow. Hide the implementation stack until it explains a problem.

The primary UI should expose five groups:

1. **Work Queue** - what needs action now.
2. **Evidence** - documents, rows, transaction IDs, workbook projections.
3. **Review** - model proposals, confidence, validation issues, approvals.
4. **Model** - privacy mode and readiness, not raw transport first.
5. **Diagnostics** - endpoint, logs, feature flags, setup checks, generated docs.

Everything else is a hidden layer.

---

## Proposed Information Architecture

### Layer 1 - Operator Tasks

Visible by default.

| Group | Operator Question | Primary Actions |
|---|---|---|
| Work Queue | What needs my attention? | Review blocked items, continue ingest, export workbook |
| Evidence | What supports this transaction? | Open source context, view workbook row, inspect graph |
| Review | What did the agent suggest? | Approve, reject, edit classification, request explanation |
| Model | Is local assistance ready? | Choose privacy mode, test local model, switch provider |
| Docs Playbook | What is the guided flow? | Open playbook, load prompt seed |

### Layer 2 - Progressive Detail

Visible after selecting a group.

| Group | Revealed Details |
|---|---|
| Work Queue | status, counts, latest failure, next safe command |
| Evidence | document filename, normalized rows, transaction hash, workbook sheet/row |
| Review | confidence, validation disposition, proposal provenance, audit event |
| Model | provider choice, readiness, test result, setup command |
| Docs Playbook | rendered guide, prompt seed, current DSL block |

### Layer 3 - Diagnostics

Hidden behind "Details", "Troubleshoot", or "Developer".

| Diagnostic Layer | Examples |
|---|---|
| Transport | endpoint URL, API key marker, request payload preview |
| Runtime | `mistralrs` compiled, Candle compiled, Foundry status, fallback mode |
| Build | feature flags, crate membership, CI desktop packages |
| MCP | generated schemas, hidden aliases, raw `tools/list` payload |
| Storage | sidecar snapshot version, ontology JSON shape, raw context paths |

---

## Model Provider UX

### Current Provider Names

- Internal Phi-4 Mini
- Windows AI / Foundry Local
- Cloud / OpenAI

### Proposed Operator Labels

| Visible Label | Technical Provider | Default Meaning |
|---|---|---|
| Local Demo | internal fallback or GGUF Phi-4 | Works immediately; private; may be deterministic fallback |
| Windows AI | Foundry Local `phi-4-mini` | Private; requires Windows setup; better demo of actual local model path |
| Cloud | OpenAI-compatible endpoint | Explicit external call; requires user-provided endpoint/model/key |

### Readiness States

Each provider should show one of:

- **Ready** - can send now.
- **Setup Needed** - show one command or button-equivalent recipe.
- **Unavailable** - explain missing dependency.
- **Diagnostic** - endpoint/service exists but model load or smoke test failed.

The operator should not need to infer readiness from API key text or endpoint URL.

### Acceptance Criteria

**AC-5.1** - Selecting Windows AI does not auto-install or auto-select external services.

**AC-5.2** - If `foundry` is missing, the UI says "Setup Needed" and points to `just windows-ai-install`.

**AC-5.3** - If Foundry Local is running, the UI shows the discovered endpoint only in details.

**AC-5.4** - A cloud model named `phi-4-mini` is not visually labeled as Windows AI unless the provider marker is local Foundry.

---

## Simplified Host Navigation

### Current Shape

- Chat
- Logs
- Settings
- Docs Playbook

### Proposed Shape

- **Today** - work queue and next safe actions.
- **Review** - classifications, proposals, validation issues.
- **Evidence** - transaction lineage and visual audit graph.
- **Model** - provider mode, readiness, test.
- **Developer** - logs, endpoints, raw prompts, settings file, docs route.

Chat remains available, but it should sit inside Review or Model rather than being the whole first screen. The first screen should answer "what needs doing?" before "what do you want to ask?"

---

## Hidden Layer Grouping Rules

### Hide By Default

- endpoint URLs,
- API key placeholders,
- feature flags,
- crate names,
- generated schemas,
- raw JSON payloads,
- internal compatibility aliases,
- exact sidecar paths,
- CI/native package details.

### Show When Relevant

- endpoint URL after a failed model call,
- generated MCP contract when a tool call fails validation,
- sidecar snapshot details when recovery fails,
- feature flags when a local model backend is expected but not compiled,
- CI/native package details in developer docs only.

### Always Show

- source document identity,
- account and date,
- transaction amount and description,
- classification category and confidence,
- validation disposition,
- approval state,
- audit event identity,
- whether data leaves the machine.

---

## North Star Visual

```rhai
fn operator_opens_app() -> today_queue
fn today_queue() -> review_items
fn review_items() -> evidence_graph
fn evidence_graph() -> approve_or_reject
fn approve_or_reject() -> workbook_export
fn model_mode() -> local_demo
fn model_mode() -> windows_ai
fn model_mode() -> cloud_explicit
fn windows_ai() -> readiness_check
fn readiness_check() -> setup_needed
fn readiness_check() -> ready_to_assist
fn developer_details() -> transport_logs
fn developer_details() -> mcp_contract
fn developer_details() -> build_diagnostics
```

The product should optimize for this visual flow. The operator path is short; hidden layers branch off only when needed.

---

## Phase Plan

### Phase 1 - Rename And Group The Existing Host Surface

Goal: improve comprehension without changing the model backend.

Scope:

- Rename provider labels to Local Demo, Windows AI, and Cloud.
- Add readiness status text next to each provider.
- Move endpoint/API key fields under a collapsible or clearly secondary settings area.
- Keep logs available but move them behind Developer.

Acceptance:

- Operator can identify whether the selected provider is private/local/cloud without reading endpoint text.
- Windows AI setup failure has a single next command.
- Existing chat send path still works.

### Phase 2 - Add Today Queue

Goal: make the first screen operational rather than conversational.

Scope:

- Add counts for ready-to-review, blocked, exported, and last run status.
- Add next safe actions.
- Link from each queue item to Review or Evidence.

Acceptance:

- On launch, the user sees what needs attention before seeing raw chat.
- Empty states are actionable and concise.

### Phase 3 - Evidence Graph As Review Context

Goal: make ontology traceability the normal review surface.

Scope:

- Add transaction-centered evidence view.
- Show deterministic facts, model proposals, operator approvals, and missing provenance with distinct badges.
- Keep raw graph/JSON in Developer.

Acceptance:

- A transaction can be traced from source document to workbook row visually.
- Missing provenance is explicit.

### Phase 4 - Collapse Duplicate Desktop Shells

**Status: Executed (2026-05-01).** Tauri is the primary desktop host. Slint is the legacy interface.

Goal: avoid long-term Slint/Tauri product ambiguity.

Scope:

- Choose primary host shell for operator demos. → **Tauri selected.**
- Keep the other shell as experimental or remove it from default workspace CI. → **Slint is legacy; CI checks Tauri by default.**
- Document the decision in `AGENTS.md`. → **Done.**

Acceptance:

- There is one recommended desktop launch path for users.
- CI checks only the supported default path unless an experimental job opts into more.

---

## Non-Goals

- Do not replace the workbook as the accountant-facing artifact.
- Do not expand the MCP advertised catalog beyond compact `ledgerr_*` capability families.
- Do not auto-select Windows AI or cloud providers.
- Do not expose raw credentials to models.
- Do not require users to understand Slint/Tauri/Rig/Candle/mistralrs to operate the product.

---

## Open Questions

- Should "Local Demo" describe deterministic fallback explicitly in the first-level UI, or only in details?
- Should Tauri remain in the default workspace while Slint is now the legacy host shell? → **Tauri is primary. Slint is legacy. Both remain in workspace.**
- Should Windows AI setup recipes eventually be surfaced as buttons, or stay as `Justfile` commands for operator-controlled setup?
- What is the smallest real data sample that can populate the Today queue without exposing private financial data?

---

## Success Metrics

- A first-time operator can choose a model mode in under 30 seconds.
- A failed Windows AI setup points to one command and one recovery path.
- A reviewer can answer "why does this workbook row exist?" from the Evidence group without opening raw JSON.
- The default host view contains no endpoint URL unless the operator opens details.
- Documentation and UI use the same provider labels.

