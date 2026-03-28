---
phase: 06-local-deployment-release-readiness
plan: 01
type: execute
wave: 1
depends_on: ["05-cpa-workbook-outputs"]
files_modified:
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
  - .githooks/commit-msg
  - scripts/install-hooks.sh
  - scripts/e2e_mvp.sh
  - crates/turbo-mcp/tests/e2e_mvp_flow.rs
  - README.md
  - docs/release-process.md
autonomous: true
requirements: ["REL-01", "REL-02", "REL-03"]
must_haves:
  truths:
    - "Container build is covered in CI."
    - "Release process uses Cocogitto + Conventional Commits."
    - "Behavior-driven e2e MVP flow is executable as a single command."
---

<objective>
Ship deployment/release readiness with CI, release automation, and e2e flow validation.
</objective>

<task_checklist>
- [x] Task 1: Add release-readiness tests and e2e flow.
- [x] Task 2: Add CI workflows and container build checks.
- [x] Task 3: Document and enforce conventional-commit + Cocogitto release process.
</task_checklist>

