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
fn test_item_name_2341() {
    let item_id = 2341;
    let item_names = [
        get_item_name(item_id, false, Locale::EN),
        get_item_name(item_id, false, Locale::DE),
        get_item_name(item_id, false, Locale::FR),
        get_item_name(item_id, false, Locale::JP),
    ];
    assert_eq!(
        item_names,
        [
            "Bronze Cross-pein Hammer",
            "Bronze-Kreuzschlaghammer", // "<SoftHyphen/>" should not appear in the item name
            "Marteau à panne croisée",
            "クロスペインハンマー"
        ]
    );
}

#[test]
fn test_item_name_44232_hq() {
    let item_id = 44232;
    let item_names = [
        get_item_name(item_id, true, Locale::EN),
        get_item_name(item_id, true, Locale::DE),
        get_item_name(item_id, true, Locale::FR),
        get_item_name(item_id, true, Locale::JP),
    ];
    assert_eq!(
        item_names,
        [
            "Rarefied Tacos de Carne Asada (HQ)",
            "Tacos de Carne Asada (Sammlerstück) (HQ)",
            "Tacos de carne asada collectionnables (HQ)",
            "収集用のタコス・カルネ・アサーダ (HQ)"
        ]
    );
}
