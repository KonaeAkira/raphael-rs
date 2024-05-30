use serde::{de, Deserialize};
use std::collections::{HashMap, HashSet};
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
    let mut items_csv = csv::Reader::from_path("data/Item.csv")?;
    let mut recipes_csv = csv::Reader::from_path("data/Recipe.csv")?;
    let mut recipe_levels_csv = csv::Reader::from_path("data/RecipeLevelTable.csv")?;

    let mut recipe_levels = HashMap::new();

    for record in recipe_levels_csv.deserialize::<RecipeLevelRecord>() {
        let recipe_level = record?;
        recipe_levels.insert(recipe_level.recipe_level, recipe_level);
    }

    fn apply_factor(base: u32, factor: u32) -> u32 {
        ((base * factor) as f64 / 100.0).floor() as u32
    }

    let mut items_with_recipe = HashSet::new();
    let mut recipes = phf_codegen::OrderedMap::new();

    for recipe_record in recipes_csv.deserialize::<RecipeRecord>() {
        let recipe_record = recipe_record?;

        // skip if a recipe for this item already exists
        // might be a problem if an item has multiple recipes with different ingredients
        if items_with_recipe.contains(&recipe_record.resulting_item) {
            continue;
        }

        let rlvl_record = recipe_levels.get(&recipe_record.recipe_level).unwrap();

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

        let recipe = format!("Recipe {{ recipe_level: {recipe_level}, progress: {progress}, quality: {quality}, durability: {durability}, progress_div: {progress_div}, quality_div: {quality_div}, progress_mod: {progress_mod}, quality_mod: {quality_mod}, material_quality_factor: {material_quality_factor}, ingredients: {ingredients} }}",
            recipe_level = recipe_record.recipe_level,
            progress = apply_factor(rlvl_record.progress, recipe_record.progress_factor),
            quality = apply_factor(rlvl_record.quality, recipe_record.quality_factor),
            durability = apply_factor(rlvl_record.durability, recipe_record.durability_factor),
            progress_div = rlvl_record.progress_divider,
            quality_div = rlvl_record.quality_divider,
            progress_mod = rlvl_record.progress_modifier,
            quality_mod = rlvl_record.quality_modifier,
            material_quality_factor = recipe_record.material_quality_factor,
            ingredients = ingredients
        );

        items_with_recipe.insert(recipe_record.resulting_item);
        recipes.entry(recipe_record.resulting_item, &recipe);
    }

    let out_path = Path::new(&env::var("OUT_DIR")?).join("recipes.rs");
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writeln!(writer, "{}", recipes.build())?;

    let mut item_ids = phf_codegen::OrderedMap::new();
    let mut items = phf_codegen::OrderedMap::new();

    let mut seen_item_ids = HashSet::new();

    for item_record in items_csv.deserialize::<ItemRecord>() {
        let item_record = item_record?;

        if !seen_item_ids.contains(&item_record.name) {
            seen_item_ids.insert(item_record.name.clone());
            item_ids.entry(item_record.name.clone(), &format!("{}", item_record.id));
        }
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

    let out_path = Path::new(&env::var("OUT_DIR")?).join("item_ids.rs");
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writeln!(writer, "{}", item_ids.build())?;

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
    #[serde(rename = "#")]
    recipe_level: u32,
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
