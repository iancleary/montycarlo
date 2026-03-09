# montycarlo

[![Crates.io](https://img.shields.io/crates/v/montycarlo.svg)](https://crates.io/crates/montycarlo)

A generic Monte Carlo simulation engine for Rust.

`montycarlo` provides a reusable framework to:
- define simulations via a trait,
- run large trial sets (sequential or parallel),
- compute useful statistics (mean, variance, percentile, CDF, exceedance).

## Install

```toml
[dependencies]
montycarlo = "0.1.0"
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

## License

MIT
