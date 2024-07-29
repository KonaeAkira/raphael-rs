use game_data::{get_item_name, Locale, ITEMS, RECIPES};

#[test]
/// Test that all ingredients have an entry in the ITEMS table
fn test_recipe_ingredients_have_valid_id() {
    for recipe in RECIPES.iter() {
        for ingredient in recipe.ingredients.iter() {
            if ingredient.item_id != 0 {
                assert!(ITEMS.contains_key(&ingredient.item_id));
            }
        }
    }
}

#[test]
fn test_display_name_44232() {
    let item_id = 44232;
    assert_eq!(
        get_item_name(item_id, true, Locale::EN),
        "Rarefied Tacos de Carne Asada (HQ)"
    );
    assert_eq!(
        get_item_name(item_id, true, Locale::DE),
        "Tacos de Carne Asada (Sammlerstück) (HQ)"
    );
    assert_eq!(
        get_item_name(item_id, true, Locale::FR),
        "Tacos de carne asada collectionnables (HQ)"
    );
    assert_eq!(
        get_item_name(item_id, true, Locale::JP),
        "収集用のタコス・カルネ・アサーダ (HQ)"
    );
}
