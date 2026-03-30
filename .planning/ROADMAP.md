# Roadmap: tax-ledger

## Milestones

- ✅ **v1.0 MVP** — Phases 1-6 shipped 2026-03-29 ([archive](./milestones/v1.0-ROADMAP.md))
- ✅ **v1.1 FDKMS Integrity** — Phases 7-18 shipped 2026-03-29 ([archive](./milestones/v1.1-ROADMAP.md))
- 🚧 **v1.2 Claude Connector Interop** — Phases 19-21 (in progress)

## Overview

Milestone v1.2 focuses on Claude connector interoperability so l3dg3rr can be installed, activated, and operated through connector-style MCP workflows with deterministic capability metadata, scoped permissions, and auditable session behavior.

## Phases

- [ ] **Phase 19: Connector Capability Profile and Scope Contracts** - Define connector-facing capability metadata, deterministic tool descriptors, and permission scope contracts.
- [ ] **Phase 20: Connector Installation and Activation Workflows** - Implement and validate operator install/activation flows for Claude/Cowork/Desktop connector contexts.
- [ ] **Phase 21: Connector Session Execution and Governance Diagnostics** - Prove connector-session tool execution, deterministic denial/error semantics, and auditable interaction outcomes.

## Phase Details

### Phase 19: Connector Capability Profile and Scope Contracts
**Goal**: Expose deterministic connector capability profiles and permission scope contracts for l3dg3rr MCP tools.
**Depends on**: Phase 18
**Requirements**: CCONN-01, CCONN-04
**Success Criteria**:
  1. Connector-facing tool metadata is deterministic and concise across runs.
  2. Permission scope policy is explicit by capability class and action type.
  3. Scope-denied operations return deterministic machine-readable denial diagnostics.

### Phase 20: Connector Installation and Activation Workflows
**Goal**: Deliver clear connector install and activation pathways with deterministic verification for operator environments.
**Depends on**: Phase 19
**Requirements**: CCONN-02, CCONN-06
**Success Criteria**:
  1. Claude/Cowork/Desktop connector install paths are documented and executable.
  2. Activation checks verify connector readiness deterministically.
  3. Organization-level compatibility/readiness notes are captured without changing ledger invariants.

### Phase 21: Connector Session Execution and Governance Diagnostics
**Goal**: Validate connector-scoped tool discovery/invocation and expose governance-grade diagnostics for operations.
**Depends on**: Phase 20
**Requirements**: CCONN-03, CCONN-05
**Success Criteria**:
  1. Connector sessions can run tools/list and tools/call for supported capabilities.
  2. Session-constrained failures map to deterministic reason keys and error classes.
  3. Connector interaction outcomes are auditable by success/blocked/error categories.

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 19. Connector Capability Profile and Scope Contracts | 0/TBD | Not started | - |
| 20. Connector Installation and Activation Workflows | 0/TBD | Not started | - |
| 21. Connector Session Execution and Governance Diagnostics | 0/TBD | Not started | - |

## Backlog

- Phase 999.1: CI + Release Automation Hardening (deferred from prior cycle; can be promoted if needed)
