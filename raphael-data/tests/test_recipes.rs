use raphael_data::*;

#[track_caller]
fn assert_item_exists(item_id: u32) {
    assert!(ITEMS.contains_key(&item_id));
    assert!(ITEM_NAMES_EN.contains_key(&item_id));
    assert!(ITEM_NAMES_DE.contains_key(&item_id));
    assert!(ITEM_NAMES_FR.contains_key(&item_id));
    assert!(ITEM_NAMES_JP.contains_key(&item_id));
    assert!(ITEM_NAMES_KR.contains_key(&item_id));
}

#[test]
fn test_all_recipe_items_exist() {
    for recipe in RECIPES.values() {
        assert_item_exists(recipe.item_id);
        for ingredient in recipe.ingredients.iter() {
            if ingredient.item_id != 0 {
                assert_item_exists(ingredient.item_id);
            }
        }
    }
}

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

#[test]
fn test_medical_supplies() {
    let recipe = find_recipe("Medical Supplies \u{e03d}").unwrap();
    assert_eq!(
        recipe,
        Recipe {
            job_id: 0,
            item_id: 33225,
            max_level_scaling: 0,
            recipe_level: 395,
            progress_factor: 100,
            quality_factor: 80,
            durability_factor: 75,
            material_factor: 0,
            ingredients: [Ingredient::default(); 6],
            is_expert: false,
            req_craftsmanship: 0,
            req_control: 0,
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
            max_level_scaling: 0,
            recipe_level: 710,
            progress_factor: 55,
            quality_factor: 80,
            durability_factor: 50,
            material_factor: 0,
            ingredients: [Ingredient::default(); 6],
            is_expert: false,
            req_craftsmanship: 4740,
            req_control: 4400,
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
            max_level_scaling: 0,
            recipe_level: 641,
            progress_factor: 200,
            quality_factor: 200,
            durability_factor: 100,
            material_factor: 0,
            ingredients: [Ingredient::default(); 6],
            is_expert: true,
            req_craftsmanship: 3950,
            req_control: 0,
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
            max_level_scaling: 100,
            recipe_level: 690,
            progress_factor: 54,
            quality_factor: 87,
            durability_factor: 88,
            material_factor: 0,
            ingredients: [Ingredient::default(); 6],
            is_expert: false,
            req_craftsmanship: 0,
            req_control: 0,
        }
    );
}
