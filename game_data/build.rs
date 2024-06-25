use serde::{de, Deserialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{env, process};

fn main() {
    if let Err(error) = import_game_data() {
        println!("{}", error);
        process::exit(1);
    }
}

fn import_game_data() -> Result<(), Box<dyn std::error::Error>> {
    let rlvls = import_rlvl_records()?;
    import_recipe_records(&rlvls)?;
    import_item_records()?;
    Ok(())
}

fn import_rlvl_records() -> Result<Vec<RecipeLevelRecord>, Box<dyn std::error::Error>> {
    let mut rlvl_table_csv = csv::Reader::from_path("data/RecipeLevelTable.csv")?;
    let rlvl_records: Vec<_> = rlvl_table_csv
        .deserialize::<RecipeLevelRecord>()
        .map(|record| record.unwrap())
        .collect();
    let mut writer =
        BufWriter::new(File::create(Path::new(&env::var("OUT_DIR")?).join("rlvls.rs")).unwrap());
    writeln!(writer, "[")?;
    for record in rlvl_records.iter() {
        writeln!(writer, "RecipeLevel {{ progress_div: {}, quality_div: {}, progress_mod: {}, quality_mod: {} }},", record.progress_divider, record.quality_divider, record.progress_modifier, record.quality_modifier)?;
    }
    writeln!(writer, "]")?;
    Ok(rlvl_records)
}

fn import_recipe_records(rlvls: &[RecipeLevelRecord]) -> Result<(), Box<dyn std::error::Error>> {
    fn apply_factor(base: u32, factor: u32) -> u32 {
        ((base * factor) as f64 / 100.0).floor() as u32
    }

    let mut items_with_recipe = HashSet::new();
    let mut recipes = phf_codegen::OrderedMap::new();

    let mut recipes_csv = csv::Reader::from_path("data/Recipe.csv")?;
    for recipe_record in recipes_csv.deserialize::<RecipeRecord>() {
        let recipe_record = recipe_record?;

        // skip if a recipe for this item already exists
        // might be a problem if an item has multiple recipes with different ingredients
        if items_with_recipe.contains(&recipe_record.resulting_item) {
            continue;
        }

        // skip the debug recipe (item id 0)
        if recipe_record.resulting_item == 0 {
            continue;
        }

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

    let out_path = Path::new(&env::var("OUT_DIR")?).join("recipes.rs");
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writeln!(writer, "{}", recipes.build())?;

    Ok(())
}

fn import_item_records() -> Result<(), Box<dyn std::error::Error>> {
    let mut items_csv = csv::Reader::from_path("data/Item.csv")?;
    let mut items = phf_codegen::OrderedMap::new();
    for item_record in items_csv.deserialize::<ItemRecord>() {
        let item_record = item_record?;
        items.entry(
            item_record.id,
            &format!(
                "Item {{ name: \"{name}\", item_level: {item_level}, can_be_hq: {can_be_hq} }}",
                name = item_record.name,
                item_level = item_record.item_level,
                can_be_hq = item_record.can_be_hq
            ),
        );
    }

    let out_path = Path::new(&env::var("OUT_DIR")?).join("items.rs");
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writeln!(writer, "{}", items.build())?;

    Ok(())
}

#[derive(Deserialize)]
struct ItemRecord {
    #[serde(rename = "#")]
    id: u32,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Level{Item}")]
    item_level: u32,
    #[serde(rename = "CanBeHq")]
    #[serde(deserialize_with = "bool_string")]
    can_be_hq: bool,
}

#[derive(Deserialize)]
struct RecipeRecord {
    #[serde(rename = "Item{Result}")]
    resulting_item: u32,
    #[serde(rename = "RecipeLevelTable")]
    recipe_level: u32,
    #[serde(rename = "DifficultyFactor")]
    progress_factor: u32,
    #[serde(rename = "QualityFactor")]
    quality_factor: u32,
    #[serde(rename = "DurabilityFactor")]
    durability_factor: u32,
    #[serde(rename = "MaterialQualityFactor")]
    material_quality_factor: u32,

    #[serde(rename = "Item{Ingredient}[0]")]
    ingredient_id_0: u32,
    #[serde(rename = "Amount{Ingredient}[0]")]
    ingredient_amount_0: u32,
    #[serde(rename = "Item{Ingredient}[1]")]
    ingredient_id_1: u32,
    #[serde(rename = "Amount{Ingredient}[1]")]
    ingredient_amount_1: u32,
    #[serde(rename = "Item{Ingredient}[2]")]
    ingredient_id_2: u32,
    #[serde(rename = "Amount{Ingredient}[2]")]
    ingredient_amount_2: u32,
    #[serde(rename = "Item{Ingredient}[3]")]
    ingredient_id_3: u32,
    #[serde(rename = "Amount{Ingredient}[3]")]
    ingredient_amount_3: u32,
    #[serde(rename = "Item{Ingredient}[4]")]
    ingredient_id_4: u32,
    #[serde(rename = "Amount{Ingredient}[4]")]
    ingredient_amount_4: u32,
    #[serde(rename = "Item{Ingredient}[5]")]
    ingredient_id_5: u32,
    #[serde(rename = "Amount{Ingredient}[5]")]
    ingredient_amount_5: u32,
}

#[derive(Deserialize)]
struct RecipeLevelRecord {
    #[serde(rename = "Durability")]
    durability: u32,
    #[serde(rename = "Difficulty")]
    progress: u32,
    #[serde(rename = "Quality")]
    quality: u32,
    #[serde(rename = "ProgressDivider")]
    progress_divider: u32,
    #[serde(rename = "QualityDivider")]
    quality_divider: u32,
    #[serde(rename = "ProgressModifier")]
    progress_modifier: u32,
    #[serde(rename = "QualityModifier")]
    quality_modifier: u32,
}

fn bool_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: de::Deserializer<'de>,
{
    let b = String::deserialize(deserializer)?;
    match b.trim().to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(de::Error::custom("invalid boolean string")),
    }
}
