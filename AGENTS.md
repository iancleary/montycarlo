# AGENTS.md

`montycarlo` is a small public Rust crate: a generic Monte Carlo simulation
engine plus statistical result helpers. Keep changes minimal, tested, and
aligned to that purpose.

Start with:

- `README.md` for the public crate promise and user-facing examples.
- `src/lib.rs` for the full API surface; this crate is intentionally
  single-module.
- `tests/engine_integration.rs` for external-use coverage.
- `docs/agent-operating-loop.md` for change modes, invariants, and verification
  expectations.

Prefer focused API additions over broad framework changes. Preserve the crate's
core shape unless the task explicitly calls for a larger design change:

- users implement `Simulation`;
- `MonteCarloEngine` owns trial execution and optional seeding;
- `MonteCarloResult` owns sorted `f64` outputs and cached aggregate statistics;
- `parallel` remains an optional feature, enabled by default.

Release workflow maintenance uses the portable `create-release-process` skill.
Ordinary release execution uses the repo-local `cut-release` workflow described
in `docs/release.md`; prefer `just cut-release` when `just` is available.
