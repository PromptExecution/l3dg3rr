# Release Process

## Preconditions

- CI workflow is green on `main`.
- Commits follow Conventional Commits.
- Local hooks installed:

```bash
./scripts/install-hooks.sh
```

## Local Dry-Run

```bash
cargo test --workspace --all-targets --all-features
./scripts/e2e_mvp.sh
docker build -t tax-ledger:dev .
```

## Create Release

```bash
cog check
cog bump --auto
cog changelog --at "$(git describe --tags --abbrev=0)"
git push --follow-tags
```

The GitHub `release.yml` workflow publishes the corresponding GitHub release after CI succeeds.

