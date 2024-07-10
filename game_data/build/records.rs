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
pub struct ItemActionRecord {
    #[serde(rename = "#")]
    pub id: u32,
    #[serde(rename = "Type")]
    pub type_id: u32,
    #[serde(rename = "Data[1]")]
    pub data_1: u32,
}

#[derive(Clone, Copy, Deserialize)]
pub struct ItemFoodRecord {
    #[serde(rename = "#")]
    pub id: u32,
    #[serde(rename = "BaseParam[0]")]
    pub param_0: u32,
    #[serde(rename = "IsRelative[0]")]
    #[serde(deserialize_with = "bool_string")]
    pub param_0_relative: bool,
    #[serde(rename = "Value[0]")]
    pub param_0_value: i32,
    #[serde(rename = "Max[0]")]
    pub param_0_max: u32,
    #[serde(rename = "Value{HQ}[0]")]
    pub param_0_hq_value: i32,
    #[serde(rename = "Max{HQ}[0]")]
    pub param_0_hq_max: u32,
    #[serde(rename = "BaseParam[1]")]
    pub param_1: u32,
    #[serde(rename = "IsRelative[1]")]
    #[serde(deserialize_with = "bool_string")]
    pub param_1_relative: bool,
    #[serde(rename = "Value[1]")]
    pub param_1_value: i32,
    #[serde(rename = "Max[1]")]
    pub param_1_max: u32,
    #[serde(rename = "Value{HQ}[1]")]
    pub param_1_hq_value: i32,
    #[serde(rename = "Max{HQ}[1]")]
    pub param_1_hq_max: u32,
    #[serde(rename = "BaseParam[2]")]
    pub param_2: u32,
    #[serde(rename = "IsRelative[2]")]
    #[serde(deserialize_with = "bool_string")]
    pub param_2_relative: bool,
    #[serde(rename = "Value[2]")]
    pub param_2_value: i32,
    #[serde(rename = "Max[2]")]
    pub param_2_max: u32,
    #[serde(rename = "Value{HQ}[2]")]
    pub param_2_hq_value: i32,
    #[serde(rename = "Max{HQ}[2]")]
    pub param_2_hq_max: u32,
}

#[derive(Deserialize)]
pub struct ItemRecord {
    #[serde(rename = "#")]
    pub id: u32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Level{Item}")]
    pub item_level: u32,
    #[serde(rename = "ItemAction")]
    pub item_action: u32,
    #[serde(rename = "CanBeHq")]
    #[serde(deserialize_with = "bool_string")]
    pub can_be_hq: bool,
    #[serde(rename = "IsCollectable")]
    #[serde(deserialize_with = "bool_string")]
    pub is_collectable: bool,
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

    #[serde(rename = "IsExpert")]
    #[serde(deserialize_with = "bool_string")]
    pub is_expert: bool,
}

#[derive(Deserialize)]
pub struct RecipeLevelRecord {
    #[serde(rename = "ClassJobLevel")]
    pub level: u8,
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
