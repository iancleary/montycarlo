# montycarlo

[![Crates.io](https://img.shields.io/crates/v/montycarlo.svg)](https://crates.io/crates/montycarlo)
[![Documentation](https://docs.rs/montycarlo/badge.svg)](https://docs.rs/montycarlo)

A generic Monte Carlo simulation engine for Rust.

`montycarlo` provides a reusable framework to:
- define simulations via a trait,
- run large trial sets (sequential or parallel),
- compute useful statistics (mean, variance, percentile, CDF, exceedance).

## When to use it

Use `montycarlo` when the random sampling logic belongs in your own domain model, but you want a
small engine to handle trial execution and common summary statistics. The crate is intentionally
generic over sample types, while requiring simulation outputs to be convertible to `f64` for
analysis.

## Install

```toml
[dependencies]
montycarlo = "0.1.2"
```

## Quick example

```rust
use montycarlo::{MonteCarloEngine, Simulation};
use rand::Rng;

struct DiceRoll;

impl Simulation for DiceRoll {
    type Sample = (u32, u32);
    type Output = f64;

    fn sample(&self, rng: &mut impl Rng) -> Self::Sample {
        (rng.gen_range(1..=6), rng.gen_range(1..=6))
    }

    fn evaluate(&self, sample: &Self::Sample) -> Self::Output {
        (sample.0 + sample.1) as f64
    }
}

let engine = MonteCarloEngine::new(DiceRoll, 100_000).with_seed(42);
let result = engine.run();

assert_eq!(result.len(), 100_000);
assert!((result.mean() - 7.0).abs() < 0.1);
```

Run the same dice simulation as an example:

```bash
cargo run --example dice
```

## What you get

- `Simulation` trait for domain-specific sampling/evaluation
- `MonteCarloEngine` for running trials
- `MonteCarloResult` helpers:
  - `mean`, `variance`, `std_dev`
  - `percentile`, `median`
  - `cdf`, `exceedance`
  - `min`, `max`, `sorted_values`

## Features

- `parallel` (default): enables parallel execution via Rayon (`run_parallel`)

Disable default features for sequential-only builds:

```toml
[dependencies]
montycarlo = { version = "0.1.2", default-features = false }
```

## Reproducibility

Use `with_seed` to make sequential runs deterministic:

```rust
# use montycarlo::{MonteCarloEngine, Simulation};
# use rand::Rng;
# struct Unit;
# impl Simulation for Unit {
#     type Sample = f64;
#     type Output = f64;
#     fn sample(&self, rng: &mut impl Rng) -> Self::Sample { rng.r#gen() }
#     fn evaluate(&self, sample: &Self::Sample) -> Self::Output { *sample }
# }
let first = MonteCarloEngine::new(Unit, 1_000).with_seed(123).run();
let second = MonteCarloEngine::new(Unit, 1_000).with_seed(123).run();

assert_eq!(first.sorted_values(), second.sorted_values());
```

## Development

Common checks are available through `just`:

```bash
just fmt-check
just lint
just test
just ci
```

## License

MIT
