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
    let rlvl_records: Vec<_> =
        read_csv_data::<RecipeLevelRecord>("data/RecipeLevelTable.csv").collect();
    let mut writer = BufWriter::new(
        File::create(Path::new(&std::env::var("OUT_DIR")?).join("rlvls.rs")).unwrap(),
    );
    writeln!(writer, "[")?;
    for record in &rlvl_records {
        writeln!(
            writer,
            "RecipeLevel {{ job_level: {}, max_progress: {}, max_quality: {}, progress_div: {}, quality_div: {}, progress_mod: {}, quality_mod: {}, conditions_flag: {} }},",
            record.level,
            record.progress,
            record.quality,
            record.progress_divider,
            record.quality_divider,
            record.progress_modifier,
            record.quality_modifier,
            record.conditions_flag
        )?;
    }
    writeln!(writer, "]")?;
    Ok(rlvl_records)
}

fn import_recipe_records(
    relevant_items: &mut HashSet<u32>,
    rlvls: &[RecipeLevelRecord],
) -> Result<(), Box<dyn std::error::Error>> {
    fn apply_factor(base: u32, factor: u32) -> u32 {
        base * factor / 100
    }

    let mut recipes = Vec::new();

    for recipe_record in read_csv_data::<RecipeRecord>("data/Recipe.csv") {
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
        let recipe = format!(
            "Recipe {{ job_id: {job_id}, item_id: {item_id}, level: {level}, max_level_scaling: {max_level_scaling}, recipe_level: {recipe_level}, progress_factor: {progress_factor}, quality_factor: {quality_factor}, durability: {durability}, material_quality_factor: {material_quality_factor}, ingredients: {ingredients}, is_expert: {is_expert} }}",
            job_id = recipe_record.job_id,
            item_id = recipe_record.resulting_item,
            level = rlvl_record.level,
            max_level_scaling = recipe_record.max_level_scaling,
            recipe_level = recipe_record.recipe_level,
            progress_factor = recipe_record.progress_factor,
            quality_factor = recipe_record.quality_factor,
            durability = apply_factor(rlvl_record.durability, recipe_record.durability_factor),
            material_quality_factor = recipe_record.material_quality_factor,
            ingredients = ingredients,
            is_expert = recipe_record.is_expert,
        );

        recipes.push(recipe);
    }

    let out_path = Path::new(&std::env::var("OUT_DIR")?).join("recipes.rs");
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writeln!(writer, "&[")?;
    for recipe in recipes {
        writeln!(writer, "{},", recipe)?;
    }
    writeln!(writer, "]")?;

    Ok(())
}
