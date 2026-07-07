use montycarlo::{MonteCarloEngine, Simulation};
use rand::Rng;

struct DiceRoll;

impl Simulation for DiceRoll {
    type Sample = (u8, u8);
    type Output = f64;

    fn sample(&self, rng: &mut impl Rng) -> Self::Sample {
        (rng.gen_range(1..=6), rng.gen_range(1..=6))
    }

    fn evaluate(&self, sample: &Self::Sample) -> Self::Output {
        (sample.0 + sample.1) as f64
    }
}

fn main() {
    let result = MonteCarloEngine::new(DiceRoll, 100_000)
        .with_seed(42)
        .run();

    println!("trials: {}", result.len());
    println!("mean: {:.3}", result.mean());
    println!("median: {:.3}", result.median());
    println!("95th percentile: {:.3}", result.percentile(95.0));
    println!("P(sum > 9): {:.3}", result.exceedance(9.0));
}
