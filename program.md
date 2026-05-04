# b00t Interface Library — Autoresearch Program

You are an autonomous research agent operating on the l3dg3rr codebase.
Your single editable file is `crates/b00t-iface/src/lib.rs`.
Your eval is `cargo test -p b00t-iface && cargo clippy -p b00t-iface -- -D warnings`.
Your 5-minute wall-clock budget is per experiment.

## Ground Rules

- Only modify `crates/b00t-iface/src/lib.rs`. Never touch other files.
- Each experiment must compile, pass tests, and pass clippy.
- If an experiment fails, `git checkout -- crates/b00t-iface/src/lib.rs` and try again.
- Log every experiment outcome to `EXPERIMENTS.md` at the project root.
- The `_b00t_/` symlink points to the b00t dotfiles repo with datum files.
- The `crates/datum/` crate already provides datum file loading and validation.

## Baseline: What Exists

- `crates/datum/` — parses `.datum` files from `_b00t_/datums/`. Has tests.
- `_b00t_/datums/` — contains 4 datum files including the integration datum.
- `program.md` — this file (you are reading it).

## Your Goal

Build `crates/b00t-iface/` — a feature-configurable lifecycle manager that:

1. Defines `ProcessSurface` trait (init → operate → terminate → maintain)
2. Implements a datum file watcher surface (proves the cycle)
3. Adds governance controls (who can start, TTL, crash budget)
4. Passes all tests and clippy

## Experiment Log Format

```markdown
## Experiment N

- **Hypothesis**: what you changed and why
- **Result**: PASS / FAIL
- **Eval time**: X.Xs
- **Coverage**: XX%
- **Notes**: what you learned
```

Start when ready. Good luck.
