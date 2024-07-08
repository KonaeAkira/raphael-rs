mod records;
use records::*;
use utils::read_csv_data;

mod consumables;
mod items;
mod utils;

use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn main() {
    if let Err(error) = import_game_data() {
        println!("{}", error);
        std::process::exit(1);
    }
}

fn import_game_data() -> Result<(), Box<dyn std::error::Error>> {
    // keep track of relevant item IDs so that we don't serialize items that are never used
    let mut relevant_items: HashSet<u32> = HashSet::new();

    let rlvls = import_rlvl_records()?;
    import_recipe_records(&mut relevant_items, &rlvls)?;

    consumables::import_consumable_records(&mut relevant_items)?;
    items::import_item_records(relevant_items)?;
    Ok(())
}

fn import_rlvl_records() -> Result<Vec<RecipeLevelRecord>, Box<dyn std::error::Error>> {
    let mut rlvl_table_csv = csv::Reader::from_path("data/RecipeLevelTable.csv")?;
    let rlvl_records: Vec<_> = rlvl_table_csv
        .deserialize::<RecipeLevelRecord>()
        .map(|record| record.unwrap())
        .collect();
    let mut writer = BufWriter::new(
        File::create(Path::new(&std::env::var("OUT_DIR")?).join("rlvls.rs")).unwrap(),
    );
    writeln!(writer, "[")?;
    for record in rlvl_records.iter() {
        writeln!(writer, "RecipeLevel {{ progress_div: {}, quality_div: {}, progress_mod: {}, quality_mod: {} }},", record.progress_divider, record.quality_divider, record.progress_modifier, record.quality_modifier)?;
    }
    writeln!(writer, "]")?;
    Ok(rlvl_records)
}

fn import_recipe_records(
    relevant_items: &mut HashSet<u32>,
    rlvls: &[RecipeLevelRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    fn apply_factor(base: u32, factor: u32) -> u32 {
        ((base * factor) as f64 / 100.0).floor() as u32
    }

    let mut items_with_recipe = HashSet::new();
    let mut recipes = phf_codegen::OrderedMap::new();

    for recipe_record in read_csv_data::<RecipeRecord>("data/Recipe.csv") {
        // skip if a recipe for this item already exists
        // might be a problem if an item has multiple recipes with different ingredients
        if items_with_recipe.contains(&recipe_record.resulting_item) {
            continue;
        }

        // skip the debug recipe (item id 0)
        if recipe_record.resulting_item == 0 {
            continue;
        }

        relevant_items.insert(recipe_record.resulting_item);
        relevant_items.insert(recipe_record.ingredient_id_0);
        relevant_items.insert(recipe_record.ingredient_id_1);
        relevant_items.insert(recipe_record.ingredient_id_2);
        relevant_items.insert(recipe_record.ingredient_id_3);
        relevant_items.insert(recipe_record.ingredient_id_4);
        relevant_items.insert(recipe_record.ingredient_id_5);

        let ingredients = format!(
                "[Ingredient {{ item_id: {}, amount: {} }}, Ingredient {{ item_id: {}, amount: {} }}, Ingredient {{ item_id: {}, amount: {} }}, Ingredient {{ item_id: {}, amount: {} }}, Ingredient {{ item_id: {}, amount: {} }}, Ingredient {{ item_id: {}, amount: {} }}]",
                recipe_record.ingredient_id_0,
                recipe_record.ingredient_amount_0,
                recipe_record.ingredient_id_1,
                recipe_record.ingredient_amount_1,
                recipe_record.ingredient_id_2,
                recipe_record.ingredient_amount_2,
                recipe_record.ingredient_id_3,
                recipe_record.ingredient_amount_3,
                recipe_record.ingredient_id_4,
                recipe_record.ingredient_amount_4,
                recipe_record.ingredient_id_5,
                recipe_record.ingredient_amount_5
            );

        let rlvl_record = &rlvls[recipe_record.recipe_level as usize];
        let recipe = format!("Recipe {{ recipe_level: {recipe_level}, progress: {progress}, quality: {quality}, durability: {durability}, material_quality_factor: {material_quality_factor}, ingredients: {ingredients} }}",
                recipe_level = recipe_record.recipe_level,
                progress = apply_factor(rlvl_record.progress, recipe_record.progress_factor),
                quality = apply_factor(rlvl_record.quality, recipe_record.quality_factor),
                durability = apply_factor(rlvl_record.durability, recipe_record.durability_factor),
                material_quality_factor = recipe_record.material_quality_factor,
                ingredients = ingredients
        );

        items_with_recipe.insert(recipe_record.resulting_item);
        recipes.entry(recipe_record.resulting_item, &recipe);
    }

    let out_path = Path::new(&std::env::var("OUT_DIR")?).join("recipes.rs");
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writeln!(writer, "{}", recipes.build())?;

    Ok(())
}
