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
    let settings = get_game_settings(recipe_config, crafter_stats, None, None, false);
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
            allowed_actions: ActionMask::from_level(94, true, false),
            adversarial: false,
        }
    )
}

#[test]
fn test_smaller_water_otter_hardware() {
    let item_id = 39243;
    assert_eq!(
        get_item_name(item_id, false, Locale::EN),
        "Smaller Water Otter Fountain Hardware"
    );
    let recipe = *RECIPES.get(&item_id).unwrap();
    assert_eq!(
        ingredient_names(recipe),
        [
            "Pure Igneous Glioaether",
            "Manganese Ore",
            "Raw Blue Zircon"
        ]
    );
    let recipe_config = RecipeConfiguration {
        item_id,
        recipe,
        hq_ingredients: [0, 0, 0, 0, 0, 0],
    };
    let crafter_stats = CrafterStats {
        craftsmanship: 3858,
        control: 4057,
        cp: 687,
        level: 100,
        manipulation: true,
    };
    let settings = get_game_settings(recipe_config, crafter_stats, None, None, false);
    assert_eq!(
        settings,
        Settings {
            max_cp: 687,
            max_durability: 60,
            max_progress: 7920,
            max_quality: 17240,
            base_progress: 216,
            base_quality: 260,
            initial_quality: 0,
            job_level: 100,
            // Trained Eye is not available for expert recipes
            allowed_actions: ActionMask::from_level(100, true, false),
            adversarial: false,
        }
    )
}

#[test]
fn test_grade_8_tincture() {
    let item_id = 39730;
    assert_eq!(
        get_item_name(item_id, false, Locale::EN),
        "Grade 8 Tincture of Intelligence"
    );
    let recipe = *RECIPES.get(&item_id).unwrap();
    assert_eq!(
        ingredient_names(recipe),
        [
            "Alche-mist",
            "Grade 5 Intelligence Alkahest",
            "Earthbreak Aethersand"
        ]
    );
    let recipe_config = RecipeConfiguration {
        item_id,
        recipe,
        hq_ingredients: [0, 0, 0, 0, 0, 0],
    };
    let crafter_stats = CrafterStats {
        craftsmanship: 3858,
        control: 4057,
        cp: 687,
        level: 100,
        manipulation: true,
    };
    let settings = get_game_settings(recipe_config, crafter_stats, None, None, false);
    assert_eq!(
        settings,
        Settings {
            max_cp: 687,
            max_durability: 70,
            max_progress: 6600,
            max_quality: 14040,
            base_progress: 298,
            base_quality: 387,
            initial_quality: 0,
            job_level: 100,
            // Trained Eye is available
            allowed_actions: ActionMask::from_level(100, true, true),
            adversarial: false,
        }
    )
}

#[test]
fn test_claro_walnut_spinning_wheel() {
    let item_id = 43279;
    assert_eq!(
        get_item_name(item_id, false, Locale::EN),
        "Claro Walnut Spinning Wheel"
    );
    let recipe = *RECIPES.get(&item_id).unwrap();
    assert_eq!(
        ingredient_names(recipe),
        ["Claro Walnut Lumber", "Black Star", "Magnesia Whetstone"]
    );
    let recipe_config = RecipeConfiguration {
        item_id,
        recipe,
        hq_ingredients: [0, 0, 0, 0, 0, 0],
    };
    let crafter_stats = CrafterStats {
        craftsmanship: 4000,
        control: 3962,
        cp: 594,
        level: 99,
        manipulation: true,
    };
    let settings = get_game_settings(recipe_config, crafter_stats, None, None, false);
    assert_eq!(
        settings,
        Settings {
            max_cp: 594,
            max_durability: 80,
            max_progress: 6300,
            max_quality: 11400,
            base_progress: 241,
            base_quality: 304,
            initial_quality: 0,
            job_level: 99,
            allowed_actions: ActionMask::from_level(99, true, false),
            adversarial: false,
        }
    )
}
