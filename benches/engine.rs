use criterion::{black_box, criterion_group, criterion_main, Criterion};
use montycarlo::{MonteCarloEngine, Simulation};
use rand::Rng;

struct UniformSim;

impl Simulation for UniformSim {
    type Sample = f64;
    type Output = f64;

    fn sample(&self, rng: &mut impl Rng) -> Self::Sample {
        rng.gen::<f64>()
    }

    fn evaluate(&self, sample: &Self::Sample) -> Self::Output {
        *sample
    }
}

fn bench_run_sequential(c: &mut Criterion) {
    c.bench_function("engine_run_sequential_100k", |b| {
        b.iter(|| {
            let engine = MonteCarloEngine::new(UniformSim, black_box(100_000)).with_seed(42);
            let result = engine.run();
            black_box(result.mean())
        })
    });
}

#[cfg(feature = "parallel")]
fn bench_run_parallel(c: &mut Criterion) {
    c.bench_function("engine_run_parallel_100k", |b| {
        b.iter(|| {
            let engine = MonteCarloEngine::new(UniformSim, black_box(100_000)).with_seed(42);
            let result = engine.run_parallel();
            black_box(result.mean())
        })
    });
}

fn bench_stats_queries(c: &mut Criterion) {
    let engine = MonteCarloEngine::new(UniformSim, 200_000).with_seed(42);
    let result = engine.run();

    c.bench_function("stats_percentile_95", |b| {
        b.iter(|| black_box(result.percentile(95.0)))
    });

    c.bench_function("stats_cdf_0_75", |b| b.iter(|| black_box(result.cdf(0.75))));

    c.bench_function("stats_exceedance_0_75", |b| {
        b.iter(|| black_box(result.exceedance(0.75)))
    });
}

#[cfg(feature = "parallel")]
criterion_group!(
    benches,
    bench_run_sequential,
    bench_run_parallel,
    bench_stats_queries
);
#[cfg(not(feature = "parallel"))]
criterion_group!(benches, bench_run_sequential, bench_stats_queries);
criterion_main!(benches);
