use serde::{de, Deserialize};

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

#[derive(Deserialize)]
pub struct ItemRecord {
    #[serde(rename = "#")]
    pub id: u32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Level{Item}")]
    pub item_level: u32,
    #[serde(rename = "CanBeHq")]
    #[serde(deserialize_with = "bool_string")]
    pub can_be_hq: bool,
}

#[derive(Deserialize)]
pub struct RecipeRecord {
    #[serde(rename = "Item{Result}")]
    pub resulting_item: u32,
    #[serde(rename = "RecipeLevelTable")]
    pub recipe_level: u32,
    #[serde(rename = "DifficultyFactor")]
    pub progress_factor: u32,
    #[serde(rename = "QualityFactor")]
    pub quality_factor: u32,
    #[serde(rename = "DurabilityFactor")]
    pub durability_factor: u32,
    #[serde(rename = "MaterialQualityFactor")]
    pub material_quality_factor: u32,

    #[serde(rename = "Item{Ingredient}[0]")]
    pub ingredient_id_0: u32,
    #[serde(rename = "Amount{Ingredient}[0]")]
    pub ingredient_amount_0: u32,
    #[serde(rename = "Item{Ingredient}[1]")]
    pub ingredient_id_1: u32,
    #[serde(rename = "Amount{Ingredient}[1]")]
    pub ingredient_amount_1: u32,
    #[serde(rename = "Item{Ingredient}[2]")]
    pub ingredient_id_2: u32,
    #[serde(rename = "Amount{Ingredient}[2]")]
    pub ingredient_amount_2: u32,
    #[serde(rename = "Item{Ingredient}[3]")]
    pub ingredient_id_3: u32,
    #[serde(rename = "Amount{Ingredient}[3]")]
    pub ingredient_amount_3: u32,
    #[serde(rename = "Item{Ingredient}[4]")]
    pub ingredient_id_4: u32,
    #[serde(rename = "Amount{Ingredient}[4]")]
    pub ingredient_amount_4: u32,
    #[serde(rename = "Item{Ingredient}[5]")]
    pub ingredient_id_5: u32,
    #[serde(rename = "Amount{Ingredient}[5]")]
    pub ingredient_amount_5: u32,
}

#[derive(Deserialize)]
pub struct RecipeLevelRecord {
    #[serde(rename = "Durability")]
    pub durability: u32,
    #[serde(rename = "Difficulty")]
    pub progress: u32,
    #[serde(rename = "Quality")]
    pub quality: u32,
    #[serde(rename = "ProgressDivider")]
    pub progress_divider: u32,
    #[serde(rename = "QualityDivider")]
    pub quality_divider: u32,
    #[serde(rename = "ProgressModifier")]
    pub progress_modifier: u32,
    #[serde(rename = "QualityModifier")]
    pub quality_modifier: u32,
}
