use game_data::{
    get_game_settings, get_item_name, CrafterStats, Locale, Recipe, RecipeConfiguration, RECIPES,
};
use simulator::{ActionMask, Settings};

fn ingredient_names(recipe: Recipe) -> Vec<String> {
    recipe
        .ingredients
        .into_iter()
        .filter_map(|ingr| match ingr.item_id {
            0 => None,
            item_id => Some(get_item_name(item_id, false, Locale::EN)),
        })
        .collect()
}

#[test]
fn test_turali_pineapple_ponzecake() {
    let item_id = 44099;
    assert_eq!(
        get_item_name(item_id, false, Locale::EN),
        "Turali Pineapple Ponzecake"
    );
    let recipe = *RECIPES.get(&item_id).unwrap();
    assert_eq!(
        ingredient_names(recipe),
        [
            "Turali Pineapple",
            "Whipped Cream",
            "Garlean Cheese",
            "Lemonette",
            "Ovibos Milk"
        ]
    );
    let recipe_config = RecipeConfiguration {
        item_id,
        recipe,
        hq_ingredients: [0, 0, 1, 0, 0, 0],
    };
    let crafter_stats = CrafterStats {
        craftsmanship: 4321,
        control: 4321,
        cp: 600,
        level: 94,
        manipulation: true,
    };
    let settings = get_game_settings(recipe_config, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 600,
            max_durability: 80,
            max_progress: 5100,
            max_quality: 9800,
            base_progress: 280,
            base_quality: 355,
            initial_quality: 2180,
            job_level: 94,
            allowed_actions: ActionMask::from_level(94, true),
        }
    )
}
