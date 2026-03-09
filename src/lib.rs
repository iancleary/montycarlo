//! Generic Monte Carlo simulation engine.
//!
//! This module provides a domain-agnostic Monte Carlo framework that can be used
//! for any statistical analysis. The design is generic over the sample type and
//! result type, making it reusable beyond RF regulatory analysis.
//!
//! # Architecture
//!
//! - [`Simulation`] — defines how to generate a random sample and evaluate it
//! - [`MonteCarloEngine`] — runs N trials, collects results, computes statistics
//! - [`MonteCarloResult`] — holds raw results and provides statistical queries
//!
//! # Example
//!
//! ```
//! use montycarlo::{MonteCarloEngine, Simulation};
//! use rand::Rng;
//!
//! // Simulate rolling two dice and summing them
//! struct DiceRoll;
//!
//! impl Simulation for DiceRoll {
//!     type Sample = (u32, u32);
//!     type Output = f64;
//!
//!     fn sample(&self, rng: &mut impl Rng) -> Self::Sample {
//!         (rng.r#gen_range(1..=6), rng.r#gen_range(1..=6))
//!     }
//!
//!     fn evaluate(&self, sample: &Self::Sample) -> Self::Output {
//!         (sample.0 + sample.1) as f64
//!     }
//! }
//!
//! let engine = MonteCarloEngine::new(DiceRoll, 10_000);
//! let result = engine.run();
//! assert!((result.mean() - 7.0).abs() < 0.5);
//! assert_eq!(result.len(), 10_000);
//! ```

use rand::Rng;
use rand::SeedableRng;

/// Defines a Monte Carlo simulation.
///
/// Implement this trait to specify how random samples are generated
/// and how each sample is evaluated to produce a numeric output.
pub trait Simulation: Send + Sync {
    /// The random input drawn each trial (e.g., antenna pointing errors, rain fade values).
    type Sample: Send;
    /// The numeric output of evaluating one sample (must be convertible to f64 for statistics).
    type Output: Into<f64> + Copy + Send;

    /// Generate a random sample using the provided RNG.
    fn sample(&self, rng: &mut impl Rng) -> Self::Sample;

    /// Evaluate a sample to produce an output value.
    fn evaluate(&self, sample: &Self::Sample) -> Self::Output;
}

/// Engine that runs a [`Simulation`] for N trials and collects results.
pub struct MonteCarloEngine<S: Simulation> {
    simulation: S,
    num_trials: usize,
    seed: Option<u64>,
}

impl<S: Simulation> MonteCarloEngine<S> {
    /// Create a new engine with the given simulation and trial count.
    #[must_use]
    pub fn new(simulation: S, num_trials: usize) -> Self {
        Self {
            simulation,
            num_trials,
            seed: None,
        }
    }

    /// Set a deterministic seed for reproducible results.
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Run all trials sequentially and return collected results.
    #[must_use]
    pub fn run(&self) -> MonteCarloResult {
        let mut rng: rand::rngs::StdRng = match self.seed {
            Some(s) => SeedableRng::seed_from_u64(s),
            None => SeedableRng::from_entropy(),
        };

        let values: Vec<f64> = (0..self.num_trials)
            .map(|_| {
                let sample = self.simulation.sample(&mut rng);
                self.simulation.evaluate(&sample).into()
            })
            .collect();

        MonteCarloResult::new(values)
    }

    /// Run trials in parallel using rayon. Each thread gets its own RNG
    /// derived from the base seed (or random if no seed is set).
    ///
    /// Requires the `parallel` feature (enabled by default).
    #[cfg(feature = "parallel")]
    #[must_use]
    pub fn run_parallel(&self) -> MonteCarloResult {
        use rayon::prelude::*;

        let base_seed: u64 = self.seed.unwrap_or_else(|| {
            let mut rng = rand::rngs::StdRng::from_entropy();
            rng.r#gen::<u64>()
        });

        let chunk_size = (self.num_trials / rayon::current_num_threads()).max(1);

        let values: Vec<f64> = (0..self.num_trials)
            .into_par_iter()
            .chunks(chunk_size)
            .flat_map(|chunk| {
                // Each chunk gets a deterministic RNG seeded from base + chunk index
                let chunk_seed = base_seed.wrapping_add(chunk[0] as u64);
                let mut rng: rand::rngs::StdRng = SeedableRng::seed_from_u64(chunk_seed);
                chunk
                    .into_iter()
                    .map(|_| {
                        let sample = self.simulation.sample(&mut rng);
                        self.simulation.evaluate(&sample).into()
                    })
                    .collect::<Vec<f64>>()
            })
            .collect();

        MonteCarloResult::new(values)
    }
}

/// Results from a Monte Carlo simulation run.
///
/// Provides statistical queries over the collected output values:
/// mean, standard deviation, percentiles, CDF, min, max.
#[derive(Debug, Clone)]
pub struct MonteCarloResult {
    /// Sorted output values.
    sorted: Vec<f64>,
    /// Cached mean.
    mean: f64,
    /// Cached variance.
    variance: f64,
}

impl MonteCarloResult {
    /// Create a result from raw (unsorted) values.
    fn new(mut values: Vec<f64>) -> Self {
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Self {
            sorted: values,
            mean,
            variance,
        }
    }

    /// Number of trials.
    #[must_use]
    pub fn len(&self) -> usize {
        self.sorted.len()
    }

    /// Returns true if no trials were run.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sorted.is_empty()
    }

    /// Arithmetic mean of all output values.
    #[must_use]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Population standard deviation.
    #[must_use]
    pub fn std_dev(&self) -> f64 {
        self.variance.sqrt()
    }

    /// Population variance.
    #[must_use]
    pub fn variance(&self) -> f64 {
        self.variance
    }

    /// Minimum value.
    #[must_use]
    pub fn min(&self) -> f64 {
        self.sorted.first().copied().unwrap_or(f64::NAN)
    }

    /// Maximum value.
    #[must_use]
    pub fn max(&self) -> f64 {
        self.sorted.last().copied().unwrap_or(f64::NAN)
    }

    /// Percentile value (0.0 to 100.0).
    ///
    /// Uses linear interpolation between nearest ranks.
    #[must_use]
    pub fn percentile(&self, p: f64) -> f64 {
        assert!(
            (0.0..=100.0).contains(&p),
            "percentile must be between 0 and 100"
        );
        if self.sorted.is_empty() {
            return f64::NAN;
        }
        if self.sorted.len() == 1 {
            return self.sorted[0];
        }

        let rank = (p / 100.0) * (self.sorted.len() - 1) as f64;
        let lower = rank.floor() as usize;
        let upper = rank.ceil() as usize;
        let frac = rank - lower as f64;

        self.sorted[lower] * (1.0 - frac) + self.sorted[upper] * frac
    }

    /// Median (50th percentile).
    #[must_use]
    pub fn median(&self) -> f64 {
        self.percentile(50.0)
    }

    /// Fraction of values less than or equal to the threshold (empirical CDF).
    #[must_use]
    pub fn cdf(&self, threshold: f64) -> f64 {
        if self.sorted.is_empty() {
            return f64::NAN;
        }
        let count = self.sorted.partition_point(|&v| v <= threshold);
        count as f64 / self.sorted.len() as f64
    }

    /// Fraction of values that exceed the threshold.
    ///
    /// Useful for exceedance probability (e.g., "what fraction of time does
    /// interference exceed -10 dB?").
    #[must_use]
    pub fn exceedance(&self, threshold: f64) -> f64 {
        1.0 - self.cdf(threshold)
    }

    /// Reference to the sorted values for custom analysis or plotting.
    #[must_use]
    pub fn sorted_values(&self) -> &[f64] {
        &self.sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A trivial simulation: output = input drawn from uniform [0, 1)
    struct UniformSim;

    impl Simulation for UniformSim {
        type Sample = f64;
        type Output = f64;

        fn sample(&self, rng: &mut impl Rng) -> f64 {
            rng.r#gen::<f64>()
        }

        fn evaluate(&self, sample: &f64) -> f64 {
            *sample
        }
    }

    #[test]
    fn uniform_mean_converges() {
        let engine = MonteCarloEngine::new(UniformSim, 100_000).with_seed(42);
        let result = engine.run();
        assert!((result.mean() - 0.5).abs() < 0.01);
    }

    #[test]
    fn uniform_std_dev() {
        // Uniform [0,1) has std dev = 1/sqrt(12) ≈ 0.2887
        let engine = MonteCarloEngine::new(UniformSim, 100_000).with_seed(42);
        let result = engine.run();
        assert!((result.std_dev() - 0.2887).abs() < 0.01);
    }

    #[test]
    fn percentiles_ordered() {
        let engine = MonteCarloEngine::new(UniformSim, 10_000).with_seed(42);
        let result = engine.run();
        assert!(result.percentile(25.0) < result.percentile(50.0));
        assert!(result.percentile(50.0) < result.percentile(75.0));
    }

    #[test]
    fn cdf_and_exceedance_complement() {
        let engine = MonteCarloEngine::new(UniformSim, 10_000).with_seed(42);
        let result = engine.run();
        let threshold = 0.3;
        let sum = result.cdf(threshold) + result.exceedance(threshold);
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn deterministic_with_seed() {
        let r1 = MonteCarloEngine::new(UniformSim, 1_000)
            .with_seed(123)
            .run();
        let r2 = MonteCarloEngine::new(UniformSim, 1_000)
            .with_seed(123)
            .run();
        assert_eq!(r1.mean(), r2.mean());
        assert_eq!(r1.sorted_values(), r2.sorted_values());
    }

    #[cfg(feature = "parallel")]
    #[test]
    fn parallel_produces_correct_count() {
        let engine = MonteCarloEngine::new(UniformSim, 50_000).with_seed(99);
        let result = engine.run_parallel();
        assert_eq!(result.len(), 50_000);
        assert!((result.mean() - 0.5).abs() < 0.02);
    }

    #[test]
    fn min_max_bounds() {
        let engine = MonteCarloEngine::new(UniformSim, 10_000).with_seed(42);
        let result = engine.run();
        assert!(result.min() >= 0.0);
        assert!(result.max() < 1.0);
    }

    #[test]
    fn median_near_mean_for_symmetric() {
        let engine = MonteCarloEngine::new(UniformSim, 100_000).with_seed(42);
        let result = engine.run();
        assert!((result.median() - result.mean()).abs() < 0.01);
    }

    // Gaussian-like simulation using Box-Muller
    struct GaussianSim {
        mean: f64,
        std_dev: f64,
    }

    impl Simulation for GaussianSim {
        type Sample = f64;
        type Output = f64;

        fn sample(&self, rng: &mut impl Rng) -> f64 {
            // Box-Muller transform
            let u1: f64 = rng.r#gen::<f64>();
            let u2: f64 = rng.r#gen::<f64>();
            let z = (-2.0_f64 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            self.mean + self.std_dev * z
        }

        fn evaluate(&self, sample: &f64) -> f64 {
            *sample
        }
    }

    #[test]
    fn gaussian_statistics() {
        let sim = GaussianSim {
            mean: 10.0,
            std_dev: 2.0,
        };
        let engine = MonteCarloEngine::new(sim, 200_000).with_seed(42);
        let result = engine.run();
        assert!((result.mean() - 10.0).abs() < 0.05);
        assert!((result.std_dev() - 2.0).abs() < 0.05);
    }

    #[test]
    fn gaussian_percentiles() {
        let sim = GaussianSim {
            mean: 0.0,
            std_dev: 1.0,
        };
        let engine = MonteCarloEngine::new(sim, 200_000).with_seed(42);
        let result = engine.run();
        // 95th percentile of N(0,1) ≈ 1.645
        assert!((result.percentile(95.0) - 1.645).abs() < 0.05);
        // 99th percentile ≈ 2.326
        assert!((result.percentile(99.0) - 2.326).abs() < 0.05);
    }

    #[test]
    fn empty_result_returns_nan() {
        let result = MonteCarloResult::new(vec![]);
        assert!(result.mean().is_nan());
        assert!(result.min().is_nan());
        assert!(result.max().is_nan());
        assert!(result.is_empty());
    }

    #[test]
    fn single_value() {
        let result = MonteCarloResult::new(vec![42.0]);
        assert_eq!(result.mean(), 42.0);
        assert_eq!(result.median(), 42.0);
        assert_eq!(result.min(), 42.0);
        assert_eq!(result.max(), 42.0);
        assert_eq!(result.std_dev(), 0.0);
    }
}
