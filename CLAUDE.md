# CLAUDE.md

montycarlo: generic Monte Carlo simulation engine for Rust.

For agent workflow, invariants, and verification expectations, start with
`AGENTS.md` and `docs/agent-operating-loop.md`.

## Commands
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo bench`

## Shared Just Interface

Use `just help` to discover supported recipes. Use `just fmt-check`, `just lint`,
`just test`, and `just doc-check` for focused verification. `just check` also
verifies packaging and requires a clean checkout; `just ci` adds a release build.
`just fmt` (alias `just format`) and `just lint-fix` explicitly modify source.
Release arguments are forwarded literally by `just cut-release`; quote paths
that contain spaces. Project-specific recipes remain optional.

`just dev` runs the dice simulation example; this crate has no standalone binary.
