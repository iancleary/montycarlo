# Agent Operating Loop

This repo should be easy for future agents to inspect, modify, verify, and
release without rediscovering the crate shape from scratch.

## Crate Shape

`montycarlo` exposes a compact Monte Carlo engine:

- `Simulation` defines domain-specific sampling and evaluation.
- `MonteCarloEngine` runs a simulation for a fixed number of trials.
- `MonteCarloResult` stores sorted `f64` outputs and cached summary statistics.
- The default `parallel` feature enables `run_parallel` through Rayon.

The API is intentionally generic over input samples and simple over outputs:
simulation outputs must be convertible to `f64` because all bundled statistics
operate on numeric results.

## Operating Loop

Before changing behavior, read these files in order:

1. `README.md` for the public promise and examples crates.io users see.
2. `src/lib.rs` for the authoritative API and unit tests.
3. `tests/engine_integration.rs` for external-use behavior.
4. `Cargo.toml` for feature flags, crate metadata, and release version.
5. `docs/release.md` before touching release automation or version handling.

Then classify the work:

- Public API change: update rustdoc, README examples when affected, and add
  either unit coverage or integration coverage.
- Statistical behavior change: use deterministic seeds in tests and assert
  tolerances that match the simulated distribution and trial count.
- Feature-gated behavior: verify both default/all-features behavior and the
  relevant no-default-features path when the change can affect compilation.
- Release workflow change: update `docs/release.md` with the concrete command
  contract and validation behavior.
- Documentation-only change: keep it tied to existing commands, API, or release
  surfaces; do not document planned behavior as current behavior.

## Design Invariants

Preserve these unless the change is deliberately redesigning the crate:

- Sequential runs with the same seed are deterministic.
- `MonteCarloResult` keeps sorted values so percentile, CDF, exceedance, min,
  max, and custom analysis share one ordered backing store.
- `mean` and population `variance` are cached at result creation.
- Empty result queries return `NaN` where no statistic exists.
- Public examples should remain small and domain-neutral; the dice example is
  the canonical quick-start shape.
- `parallel` remains optional so sequential-only consumers can disable Rayon.

## Verification Guide

Use the lightest check set that proves the change:

- Docs only: `git diff --check`.
- Rust formatting or examples changed: `just fmt-check`.
- Public API, result statistics, seeding, or feature behavior changed:
  `just test`.
- Rustdoc examples or documentation comments changed: `just doc-check`.
- Release script or release contract changed: dry-run the documented release
  command with an explicit version and notes file.

Run `just ci` before release-oriented changes or broad API work when time and
environment allow.

## Accretion Rules

Leave the repo easier for the next agent:

- Add tests near the behavior they protect rather than relying on broad random
  assertions.
- Name simulations in tests after the distribution or scenario they model.
- Keep probability tolerances explicit and explain non-obvious tolerances with a
  short comment.
- Update this document when a new durable workflow or invariant appears.
- Remove stale instructions instead of layering conflicting guidance nearby.
