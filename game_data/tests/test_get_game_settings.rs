use game_data::{
    get_game_settings, get_initial_quality, get_item_name, CrafterStats, Locale, Recipe, RECIPES,
};
use simulator::{Action, ActionMask, Settings};

fn find_recipe(item_name: &'static str) -> Option<Recipe> {
    for recipe in RECIPES.iter() {
        if get_item_name(recipe.item_id, false, Locale::EN) == item_name {
            return Some(*recipe);
        }
    }
    None
}

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
    let recipe = find_recipe("Turali Pineapple Ponzecake").unwrap();
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
    let crafter_stats = CrafterStats {
        craftsmanship: 4321,
        control: 4321,
        cp: 600,
        level: 94,
        manipulation: true,
        heart_and_soul: true,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, crafter_stats, None, None, false);
    assert_eq!(
        settings,
        Settings {
            max_cp: 600,
            max_durability: 80,
            max_progress: 5100,
            max_quality: 9800,
            base_progress: 280,
            base_quality: 355,
            job_level: 94,
            allowed_actions: ActionMask::from_level(94)
                .remove(Action::TrainedEye)
                .remove(Action::QuickInnovation),
            adversarial: false,
        }
    );
    let initial_quality = get_initial_quality(recipe, [0, 0, 1, 0, 0, 0]);
    assert_eq!(initial_quality, 2180);
}

#[test]
fn test_smaller_water_otter_hardware() {
    let recipe = find_recipe("Smaller Water Otter Fountain Hardware").unwrap();
    assert_eq!(
        ingredient_names(recipe),
        [
            "Pure Igneous Glioaether",
            "Manganese Ore",
            "Raw Blue Zircon"
        ]
    );
    let crafter_stats = CrafterStats {
        craftsmanship: 3858,
        control: 4057,
        cp: 687,
        level: 100,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, crafter_stats, None, None, false);
    assert_eq!(
        settings,
        Settings {
            max_cp: 687,
            max_durability: 60,
            max_progress: 7920,
            max_quality: 17240,
            base_progress: 216,
            base_quality: 260,
            job_level: 100,
            // Trained Eye is not available for expert recipes
            allowed_actions: ActionMask::from_level(100)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        }
    );
    let initial_quality = get_initial_quality(recipe, [0, 0, 0, 0, 0, 0]);
    assert_eq!(initial_quality, 0);
}

#[test]
fn test_grade_8_tincture() {
    let recipe = find_recipe("Grade 8 Tincture of Intelligence").unwrap();
    assert_eq!(
        ingredient_names(recipe),
        [
            "Alche-mist",
            "Grade 5 Intelligence Alkahest",
            "Earthbreak Aethersand"
        ]
    );
    let crafter_stats = CrafterStats {
        craftsmanship: 3858,
        control: 4057,
        cp: 687,
        level: 100,
        manipulation: true,
        heart_and_soul: true,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, crafter_stats, None, None, false);
    assert_eq!(
        settings,
        Settings {
            max_cp: 687,
            max_durability: 70,
            max_progress: 6600,
            max_quality: 14040,
            base_progress: 298,
            base_quality: 387,
            job_level: 100,
            // Trained Eye is available
            allowed_actions: ActionMask::from_level(100).remove(Action::QuickInnovation),
            adversarial: false,
        }
    );
    let initial_quality = get_initial_quality(recipe, [0, 0, 0, 0, 0, 0]);
    assert_eq!(initial_quality, 0);
}

#[test]
fn test_claro_walnut_spinning_wheel() {
    let recipe = find_recipe("Claro Walnut Spinning Wheel").unwrap();
    assert_eq!(
        ingredient_names(recipe),
        ["Claro Walnut Lumber", "Black Star", "Magnesia Whetstone"]
    );
    let crafter_stats = CrafterStats {
        craftsmanship: 4000,
        control: 3962,
        cp: 594,
        level: 99,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: true,
    };
    let settings = get_game_settings(recipe, crafter_stats, None, None, false);
    assert_eq!(
        settings,
        Settings {
            max_cp: 594,
            max_durability: 80,
            max_progress: 6300,
            max_quality: 11400,
            base_progress: 241,
            base_quality: 304,
            job_level: 99,
            allowed_actions: ActionMask::from_level(99)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul),
            adversarial: false,
        }
    );
    let initial_quality = get_initial_quality(recipe, [0, 0, 0, 0, 0, 0]);
    assert_eq!(initial_quality, 0);
}
