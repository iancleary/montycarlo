# CLAUDE.md — montycarlo

## Overview

`montycarlo` is a generic Monte Carlo simulation engine for Rust.

Core API:
- `Simulation` trait: define random sampling + evaluation
- `MonteCarloEngine`: run trials sequentially or in parallel
- `MonteCarloResult`: mean, variance, std dev, percentile, median, CDF, exceedance

## Commands

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
cargo bench
cargo doc --open
```

## Metrics

- **Version:** v0.1.1
- **Current tests:** 13 total (unit + integration + doctest)

## Where to Look

- `README.md` — usage and quick-start example
- `src/lib.rs` — full engine implementation and unit tests
- `tests/engine_integration.rs` — integration-level smoke test
- `benches/engine.rs` — Criterion benchmarks for run and query performance
