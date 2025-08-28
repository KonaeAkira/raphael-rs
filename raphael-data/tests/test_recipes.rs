use expect_test::expect;
use raphael_data::*;

#[track_caller]
fn assert_item_exists(item_id: u32) {
    assert!(ITEMS.has_entry(item_id));
    assert!(ITEM_NAMES_EN.has_entry(item_id));
    assert!(ITEM_NAMES_DE.has_entry(item_id));
    assert!(ITEM_NAMES_FR.has_entry(item_id));
    assert!(ITEM_NAMES_JP.has_entry(item_id));
    // KR version is not up-to-date with global version, so some item names can be missing.
    // assert!(ITEM_NAMES_KR.contains_key(&item_id));
}

#[test]
fn all_recipe_items_exist() {
    for recipe in RECIPES.values() {
        assert_item_exists(recipe.item_id);
        for ingredient in recipe.ingredients.iter() {
            if ingredient.item_id != 0 {
                assert_item_exists(ingredient.item_id);
            }
        }
    }
}

#[test]
fn medical_supplies() {
    let matching_recipes = find_recipes("Medical Supplies", Locale::EN)
        .iter()
        .map(|recipe_id| RECIPES.get(recipe_id).unwrap())
        .collect::<Vec<_>>();
    let expected = expect![[r#"
        [
            Recipe {
                job_id: 0,
                item_id: 33225,
                max_level_scaling: 0,
                recipe_level: 395,
                progress_factor: 100,
                quality_factor: 80,
                durability_factor: 75,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
            Recipe {
                job_id: 1,
                item_id: 33225,
                max_level_scaling: 0,
                recipe_level: 395,
                progress_factor: 100,
                quality_factor: 80,
                durability_factor: 75,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
            Recipe {
                job_id: 2,
                item_id: 33225,
                max_level_scaling: 0,
                recipe_level: 395,
                progress_factor: 100,
                quality_factor: 80,
                durability_factor: 75,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
            Recipe {
                job_id: 3,
                item_id: 33225,
                max_level_scaling: 0,
                recipe_level: 395,
                progress_factor: 100,
                quality_factor: 80,
                durability_factor: 75,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
            Recipe {
                job_id: 4,
                item_id: 33225,
                max_level_scaling: 0,
                recipe_level: 395,
                progress_factor: 100,
                quality_factor: 80,
                durability_factor: 75,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
            Recipe {
                job_id: 5,
                item_id: 33225,
                max_level_scaling: 0,
                recipe_level: 395,
                progress_factor: 100,
                quality_factor: 80,
                durability_factor: 75,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
            Recipe {
                job_id: 6,
                item_id: 33225,
                max_level_scaling: 0,
                recipe_level: 395,
                progress_factor: 100,
                quality_factor: 80,
                durability_factor: 75,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
            Recipe {
                job_id: 7,
                item_id: 33225,
                max_level_scaling: 0,
                recipe_level: 395,
                progress_factor: 100,
                quality_factor: 80,
                durability_factor: 75,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
        ]
    "#]];
    expected.assert_debug_eq(&matching_recipes);
}

#[test]
fn ipe_lumber() {
    let matching_recipes = find_recipes("Ipe Lumber", Locale::EN)
        .iter()
        .map(|recipe_id| RECIPES.get(recipe_id).unwrap())
        .collect::<Vec<_>>();
    let expected = expect![[r#"
        [
            Recipe {
                job_id: 0,
                item_id: 44149,
                max_level_scaling: 0,
                recipe_level: 710,
                progress_factor: 55,
                quality_factor: 80,
                durability_factor: 50,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 4740,
                req_control: 4400,
            },
        ]
    "#]];
    expected.assert_debug_eq(&matching_recipes);
}

#[test]
fn uncharted_course_resin() {
    let matching_recipes = find_recipes("Uncharted Course Resin", Locale::EN)
        .iter()
        .map(|recipe_id| RECIPES.get(recipe_id).unwrap())
        .collect::<Vec<_>>();
    let expected = expect![[r#"
        [
            Recipe {
                job_id: 6,
                item_id: 39916,
                max_level_scaling: 0,
                recipe_level: 641,
                progress_factor: 200,
                quality_factor: 200,
                durability_factor: 100,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: true,
                req_craftsmanship: 3950,
                req_control: 0,
            },
            Recipe {
                job_id: 7,
                item_id: 39916,
                max_level_scaling: 0,
                recipe_level: 641,
                progress_factor: 200,
                quality_factor: 200,
                durability_factor: 100,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: true,
                req_craftsmanship: 3950,
                req_control: 0,
            },
        ]
    "#]];
    expected.assert_debug_eq(&matching_recipes);
}

#[test]
fn habitat_chair() {
    let matching_recipes = find_recipes("Habitat Chair", Locale::EN)
        .iter()
        .map(|recipe_id| RECIPES.get(recipe_id).unwrap())
        .collect::<Vec<_>>();
    let expected = expect![[r#"
        [
            Recipe {
                job_id: 0,
                item_id: 48295,
                max_level_scaling: 100,
                recipe_level: 690,
                progress_factor: 54,
                quality_factor: 87,
                durability_factor: 88,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
            Recipe {
                job_id: 4,
                item_id: 48456,
                max_level_scaling: 100,
                recipe_level: 690,
                progress_factor: 54,
                quality_factor: 87,
                durability_factor: 88,
                material_factor: 0,
                ingredients: [
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                    Ingredient {
                        item_id: 0,
                        amount: 0,
                    },
                ],
                is_expert: false,
                req_craftsmanship: 0,
                req_control: 0,
            },
        ]
    "#]];
    expected.assert_debug_eq(&matching_recipes);
}
