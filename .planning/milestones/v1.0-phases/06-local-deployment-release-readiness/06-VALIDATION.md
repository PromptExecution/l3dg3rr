# Phase 06 Validation Strategy

## Scope

- Local container execution path and CI container-build coverage.
- Cocogitto release automation and conventional-commit enforcement.
- Single-command BDD e2e MVP flow script.

## Validation Matrix

| Requirement | What to validate | Primary evidence |
| --- | --- | --- |
| REL-01 | Local Docker run path with mounted dirs is documented and container build is CI-gated | `README.md`, `.github/workflows/ci.yml`, `Dockerfile` |
| REL-02 | Versioned release/changelog workflow is Cocogitto-based | `.github/workflows/release.yml`, `docs/release-process.md`, `.cog.toml` |
| REL-03 | End-to-end MVP behavior flow has dedicated BDD entrypoint | `scripts/e2e_mvp.sh`, `e2e_mvp_flow` test |

## Execution Steps

1. `cargo test -p turbo-mcp --test e2e_mvp_flow -- --nocapture`
2. `./scripts/e2e_mvp.sh`
3. `cargo test --workspace -- --nocapture`

## Pass Criteria

- All commands pass and CI/release configuration files are present.

