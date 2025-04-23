use raphael_data::*;

const NO_INGREDIENT: Ingredient = Ingredient {
    item_id: 0,
    amount: 0,
};

fn assert_item_exists(item_id: u32) {
    assert!(ITEMS.contains_key(&item_id));
    assert!(ITEM_NAMES_EN.contains_key(&item_id));
    assert!(ITEM_NAMES_DE.contains_key(&item_id));
    assert!(ITEM_NAMES_FR.contains_key(&item_id));
    assert!(ITEM_NAMES_JP.contains_key(&item_id));
}

#[test]
fn test_all_recipe_items_exist() {
    for recipe in RECIPES.iter() {
        assert_item_exists(recipe.item_id);
        for ingredient in recipe.ingredients.iter() {
            if ingredient.item_id != 0 {
                assert_item_exists(ingredient.item_id);
            }
        }
    }
}

fn find_recipe(item_name: &'static str) -> Option<Recipe> {
    for recipe in RECIPES.iter() {
        if get_item_name(recipe.item_id, false, Locale::EN) == item_name {
            return Some(*recipe);
        }
    }
    None
}

#[test]
fn test_medical_supplies() {
    let recipe = find_recipe("Medical Supplies \u{e03d}").unwrap();
    assert_eq!(
        recipe,
        Recipe {
            job_id: 0,
            item_id: 33225,
            level: 72,
            max_level_scaling: 0,
            recipe_level: 395,
            progress_factor: 100,
            quality_factor: 80,
            durability: 60,
            material_quality_factor: 0,
            ingredients: [
                Ingredient {
                    item_id: 33235,
                    amount: 1,
                },
                NO_INGREDIENT,
                NO_INGREDIENT,
                NO_INGREDIENT,
                NO_INGREDIENT,
                NO_INGREDIENT,
            ],
            is_expert: false,
        }
    );
}

#[test]
fn test_ipe_lumber() {
    let recipe = find_recipe("Ipe Lumber").unwrap();
    assert_eq!(
        recipe,
        Recipe {
            job_id: 0,
            item_id: 44149,
            level: 100,
            max_level_scaling: 0,
            recipe_level: 710,
            progress_factor: 55,
            quality_factor: 80,
            durability: 35,
            material_quality_factor: 0,
            ingredients: [
                Ingredient {
                    item_id: 44137,
                    amount: 4
                },
                Ingredient {
                    item_id: 44141,
                    amount: 2
                },
                NO_INGREDIENT,
                NO_INGREDIENT,
                NO_INGREDIENT,
                NO_INGREDIENT,
            ],
            is_expert: false,
        }
    );
}

#[test]
fn test_uncharted_course_resin() {
    let recipe = find_recipe("Uncharted Course Resin").unwrap();
    assert_eq!(
        recipe,
        Recipe {
            job_id: 6,
            item_id: 39916,
            level: 90,
            max_level_scaling: 0,
            recipe_level: 641,
            progress_factor: 200,
            quality_factor: 200,
            durability: 60,
            material_quality_factor: 0,
            ingredients: [
                Ingredient {
                    item_id: 39913,
                    amount: 1
                },
                Ingredient {
                    item_id: 36257,
                    amount: 1
                },
                Ingredient {
                    item_id: 36091,
                    amount: 1
                },
                Ingredient {
                    item_id: 36262,
                    amount: 1
                },
                NO_INGREDIENT,
                NO_INGREDIENT,
            ],
            is_expert: true,
        }
    );
}

#[test]
fn test_habitat_chair() {
    let recipe = find_recipe("Habitat Chair \u{e03d}").unwrap();
    assert_eq!(
        recipe,
        Recipe {
            job_id: 0,
            item_id: 48295,
            level: 100,
            max_level_scaling: 100,
            recipe_level: 690,
            progress_factor: 54,
            quality_factor: 87,
            durability: 70,
            material_quality_factor: 0,
            ingredients: [
                Ingredient {
                    item_id: 48233,
                    amount: 1
                },
                Ingredient {
                    item_id: 0,
                    amount: 0
                },
                Ingredient {
                    item_id: 0,
                    amount: 0
                },
                Ingredient {
                    item_id: 0,
                    amount: 0
                },
                Ingredient {
                    item_id: 0,
                    amount: 0
                },
                Ingredient {
                    item_id: 0,
                    amount: 0
                }
            ],
            is_expert: false,
        }
    );
}
