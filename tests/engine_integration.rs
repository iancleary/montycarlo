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

#[test]
fn dice_mean_is_reasonable() {
    let result = MonteCarloEngine::new(DiceRoll, 100_000).with_seed(7).run();
    assert!((result.mean() - 7.0).abs() < 0.1);
}
