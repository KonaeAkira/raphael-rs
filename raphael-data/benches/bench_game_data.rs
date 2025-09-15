use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rand::seq::IteratorRandom;

use raphael_data::*;

struct RandomRecipeID {
    recipe_id: u32,
}

impl RandomRecipeID {
    fn access_game_data(self) -> Result<(), ()> {
        let recipe = RECIPES.get(self.recipe_id).ok_or(())?;
        ITEMS.get(recipe.item_id).ok_or(())?;
        ITEM_NAMES_EN.get(recipe.item_id).ok_or(())?;
        Ok(())
    }
}

fn bench_random_access(c: &mut Criterion) {
    fn random_permutation() -> RandomRecipeID {
        let mut rng = rand::rng();

        RandomRecipeID {
            recipe_id: RECIPES.indices().choose(&mut rng).unwrap_or_default(),
        }
    }

    c.bench_function("random_access", |b| {
        b.iter_batched(
            random_permutation,
            RandomRecipeID::access_game_data,
            BatchSize::SmallInput,
        );
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
    config = Criterion::default().warm_up_time(std::time::Duration::new(6, 0)).measurement_time(std::time::Duration::new(10, 0));
    targets = bench_random_access, bench_find_recipes, bench_find_stellar_missions
}
criterion_main!(bench_game_data);
