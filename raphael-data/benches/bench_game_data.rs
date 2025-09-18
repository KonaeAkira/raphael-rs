use criterion::{Criterion, criterion_group, criterion_main};
use rand::seq::SliceRandom;

use raphael_data::*;

fn bench_random_access(c: &mut Criterion) {
    let mut recipe_ids = RECIPES.indices().collect::<Vec<_>>();
    recipe_ids.shuffle(&mut rand::rng());

    let mut i = 0;
    c.bench_function("random_access", |b| {
        b.iter(|| {
            let recipe_id = recipe_ids[i % recipe_ids.len()];
            i += 1;

            let recipe = RECIPES.get(recipe_id);
            if let Some(recipe_ref) = recipe {
                let item = ITEMS.get(recipe_ref.item_id);
                let item_name = ITEM_NAMES_EN.get(recipe_ref.item_id);
                (recipe, item, item_name)
            } else {
                (recipe, None, None)
            }
        });
    });
}

fn bench_find_recipes(c: &mut Criterion) {
    c.bench_function("find_recipes", |b| {
        b.iter(|| find_recipes("", Locale::EN));
    });
}

fn bench_find_stellar_missions(c: &mut Criterion) {
    c.bench_function("find_stellar_missions", |b| {
        b.iter(|| find_stellar_missions("", Locale::EN));
    });
}

criterion_group! {
    name = bench_game_data;
    config = Criterion::default().warm_up_time(std::time::Duration::new(6, 0)).measurement_time(std::time::Duration::new(10, 0)).sample_size(1024);
    targets = bench_random_access, bench_find_recipes, bench_find_stellar_missions
}
criterion_main!(bench_game_data);
