use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rand::Rng;
use raphael_sim::*;

fn bench_use_action(c: &mut Criterion) {
    let settings = Settings {
        max_cp: 1000,
        max_durability: 80,
        max_progress: 50000,
        max_quality: 50000,
        base_progress: 123,
        base_quality: 321,
        job_level: 100,
        allowed_actions: ActionMask::all(),
        adversarial: false,
        backload_progress: false,
    };
    let state = SimulationState::new(&settings);

    let bench_actions = &[
        Action::BasicSynthesis,
        Action::BasicTouch,
        Action::Innovation,
    ];

    let mut group = c.benchmark_group("use_action");
    group.sample_size(1000);
    group.warm_up_time(std::time::Duration::from_millis(100));
    group.measurement_time(std::time::Duration::from_millis(400));

    for &action in bench_actions {
        let bench_id = BenchmarkId::from_parameter(format!("{:?}", action));
        group.bench_function(bench_id, |b| {
            b.iter(|| state.use_action(black_box(action), Condition::Normal, &settings));
        });
    }
    group.finish();
}

fn bench_tick_effects(c: &mut Criterion) {
    fn random_effects() -> Effects {
        let mut rng = rand::rng();
        Effects::new()
            .with_veneration(rng.random_range(0..=4))
            .with_innovation(rng.random_range(0..=4))
            .with_manipulation(rng.random_range(0..=8))
            .with_waste_not(rng.random_range(0..=8))
    }
    c.bench_function("tick_effects", |b| {
        b.iter_batched(random_effects, Effects::tick_down, BatchSize::SmallInput);
    });
}

criterion_group!(bench_simulator, bench_use_action, bench_tick_effects);
criterion_main!(bench_simulator);
