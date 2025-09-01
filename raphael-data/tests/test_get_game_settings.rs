use raphael_data::*;
use raphael_sim::{Action, ActionMask, Settings};

fn find_recipe(item_name: &'static str) -> Option<Recipe> {
    for recipe in RECIPES.values() {
        if let Some(name) = get_item_name(recipe.item_id, false, Locale::EN) {
            if name == item_name {
                return Some(*recipe);
            }
        }
    }
    None
}

fn ingredient_names(recipe: Recipe) -> Vec<String> {
    recipe
        .ingredients
        .into_iter()
        .filter_map(|ingr| get_item_name(ingr.item_id, false, Locale::EN))
        .collect()
}

#[test]
/// Verified in-game (patch 7.05)
fn test_roast_chicken() {
    let recipe = find_recipe("Roast Chicken").unwrap();
    assert_eq!(ingredient_names(recipe), ["Mountain Salt", "Frantoio Oil",]);
    let crafter_stats = CrafterStats {
        craftsmanship: 4956,
        control: 4963,
        cp: 627,
        level: 100,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 627,
            max_durability: 70,
            max_progress: 7500,
            max_quality: 16500,
            base_progress: 264,
            base_quality: 274,
            job_level: 100,
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_turali_pineapple_ponzecake() {
    let recipe = find_recipe("Turali Pineapple Ponzecake").unwrap();
    assert_eq!(
        ingredient_names(recipe),
        ["Whipped Cream", "Garlean Cheese",]
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
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
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
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
    let initial_quality = get_initial_quality(crafter_stats, recipe, [0, 1, 0, 0, 0, 0]);
    assert_eq!(initial_quality, 2180);
}

#[test]
fn test_smaller_water_otter_hardware() {
    let recipe = find_recipe("Smaller Water Otter Fountain Hardware").unwrap();
    assert!(ingredient_names(recipe).is_empty());
    let crafter_stats = CrafterStats {
        craftsmanship: 3858,
        control: 4057,
        cp: 687,
        level: 100,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
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
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_grade_8_tincture() {
    let recipe = find_recipe("Grade 8 Tincture of Intelligence").unwrap();
    assert_eq!(ingredient_names(recipe), ["Grade 5 Intelligence Alkahest",]);
    let crafter_stats = CrafterStats {
        craftsmanship: 3858,
        control: 4057,
        cp: 687,
        level: 100,
        manipulation: true,
        heart_and_soul: true,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
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
            allowed_actions: ActionMask::all().remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
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
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
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
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_habitat_chair_lv100() {
    let recipe = find_recipe("Habitat Chair \u{e03d}").unwrap();
    let crafter_stats = CrafterStats {
        craftsmanship: 3849,
        control: 4282,
        cp: 614,
        level: 100,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 614,
            max_durability: 70,
            max_progress: 3564,
            max_quality: 10440,
            base_progress: 205,
            base_quality: 240,
            job_level: 100,
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_habitat_chair_lv97() {
    // https://github.com/KonaeAkira/raphael-rs/issues/117#issuecomment-2825555081
    let recipe = find_recipe("Habitat Chair \u{e03d}").unwrap();
    let crafter_stats = CrafterStats {
        craftsmanship: 3796,
        control: 3447,
        cp: 504,
        level: 97,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 504,
            max_durability: 70,
            max_progress: 3078,
            max_quality: 9222,
            base_progress: 237,
            base_quality: 279,
            job_level: 97,
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_habitat_chair_lv98() {
    // https://github.com/KonaeAkira/raphael-rs/issues/117#issuecomment-2825559687
    let recipe = find_recipe("Habitat Chair \u{e03d}").unwrap();
    let crafter_stats = CrafterStats {
        craftsmanship: 3796,
        control: 3447,
        cp: 504,
        level: 98,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 504,
            max_durability: 70,
            max_progress: 3240,
            max_quality: 9570,
            base_progress: 233,
            base_quality: 274,
            job_level: 98,
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_standard_indurate_rings_lv93() {
    // https://github.com/KonaeAkira/raphael-rs/issues/117#issuecomment-2825560998
    let recipe = find_recipe("Standard Indurate Rings").unwrap();
    let crafter_stats = CrafterStats {
        craftsmanship: 3796,
        control: 3447,
        cp: 504,
        level: 93,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 504,
            max_durability: 40,
            max_progress: 2790,
            max_quality: 4500,
            base_progress: 256,
            base_quality: 302,
            job_level: 93,
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_lunar_alloy_ingots_lv90() {
    // https://github.com/KonaeAkira/raphael-rs/issues/117#issuecomment-2825562688
    let recipe = find_recipe("Lunar Alloy Ingot").unwrap();
    let crafter_stats = CrafterStats {
        craftsmanship: 3796,
        control: 3447,
        cp: 504,
        level: 90,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 504,
            max_durability: 80,
            max_progress: 2345,
            max_quality: 4248,
            base_progress: 264,
            base_quality: 267,
            job_level: 90,
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_standard_high_density_fiberboard_lv91() {
    // https://github.com/KonaeAkira/raphael-rs/issues/117#issuecomment-2825723068
    let recipe = find_recipe("Standard High-density Fiberboard").unwrap();
    let crafter_stats = CrafterStats {
        craftsmanship: 3796,
        control: 3447,
        cp: 504,
        level: 91,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 504,
            max_durability: 40,
            max_progress: 2440,
            max_quality: 3936,
            base_progress: 267,
            base_quality: 315,
            job_level: 91,
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}

#[test]
fn test_lunar_alloy_ingots_lv10() {
    let recipe = find_recipe("Lunar Alloy Ingot").unwrap();
    let crafter_stats = CrafterStats {
        craftsmanship: 3796,
        control: 3447,
        cp: 504,
        level: 10,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };
    let settings = get_game_settings(recipe, None, crafter_stats, None, None);
    assert_eq!(
        settings,
        Settings {
            max_cp: 504,
            max_durability: 80, // test that durability is correct at low levels
            max_progress: 30,
            max_quality: 147,
            base_progress: 761,
            base_quality: 1184,
            job_level: 10,
            allowed_actions: ActionMask::all()
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
            backload_progress: false,
        }
    );
}
