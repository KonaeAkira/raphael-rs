use std::iter::repeat;

use crate::SheetData;

#[derive(Debug, Clone, Copy, Default)]
pub struct Ingredient {
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug)]
pub struct Recipe {
    pub id: u32,
    pub job_id: u32,
    pub item_id: u32,
    pub max_level_scaling: u32,
    pub recipe_level: u32,
    pub progress_factor: u32,
    pub quality_factor: u32,
    pub durability_factor: u32,
    pub material_factor: u32,
    pub ingredients: Vec<Ingredient>,
    pub is_expert: bool,
    pub req_craftsmanship: u32,
    pub req_control: u32,
}

impl SheetData for Recipe {
    const SHEET: &'static str = "Recipe";
    const REQUIRED_FIELDS: &[&str] = &[
        "CraftType",
        "ItemResult",
        "MaxAdjustableJobLevel",
        "RecipeLevelTable",
        "DifficultyFactor",
        "QualityFactor",
        "DurabilityFactor",
        "MaterialQualityFactor",
        "IsExpert",
        "Ingredient",
        "AmountIngredient",
        "RequiredCraftsmanship",
        "RequiredControl",
    ];

    fn row_id(&self) -> u32 {
        self.id
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        let ingredients = fields["Ingredient"]
            .members()
            .zip(fields["AmountIngredient"].members())
            .filter_map(|(item, amount)| {
                Some(Ingredient {
                    item_id: item["value"].as_u32()?,
                    amount: amount.as_u32()?,
                })
            })
            .collect();
        Some(Self {
            id: value["row_id"].as_u32().unwrap(),
            job_id: fields["CraftType"]["value"].as_u32().unwrap(),
            item_id: fields["ItemResult"]["value"].as_u32().unwrap(),
            max_level_scaling: fields["MaxAdjustableJobLevel"]["value"].as_u32().unwrap(),
            recipe_level: fields["RecipeLevelTable"]["value"].as_u32().unwrap(),
            progress_factor: fields["DifficultyFactor"].as_u32().unwrap(),
            quality_factor: fields["QualityFactor"].as_u32().unwrap(),
            durability_factor: fields["DurabilityFactor"].as_u32().unwrap(),
            material_factor: fields["MaterialQualityFactor"].as_u32().unwrap(),
            ingredients,
            is_expert: fields["IsExpert"].as_bool().unwrap(),
            req_craftsmanship: fields["RequiredCraftsmanship"].as_u32().unwrap(),
            req_control: fields["RequiredControl"].as_u32().unwrap(),
        })
    }
}

impl std::fmt::Display for Recipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Recipe {{ ")?;
        write!(f, "job_id: {}, ", self.job_id)?;
        write!(f, "item_id: {}, ", self.item_id)?;
        write!(f, "max_level_scaling: {}, ", self.max_level_scaling)?;
        write!(f, "recipe_level: {}, ", self.recipe_level)?;
        write!(f, "progress_factor: {}, ", self.progress_factor)?;
        write!(f, "quality_factor: {}, ", self.quality_factor)?;
        write!(f, "durability_factor: {}, ", self.durability_factor)?;
        write!(f, "material_factor: {}, ", self.material_factor)?;
        write!(f, "ingredients: [")?;
        for ingredient in self
            .ingredients
            .iter()
            .chain(repeat(&Ingredient::default()))
            .take(6)
        {
            write!(
                f,
                "Ingredient {{ item_id: {}, amount: {} }}, ",
                ingredient.item_id, ingredient.amount
            )?;
        }
        write!(f, "], ")?;
        write!(f, "is_expert: {}, ", self.is_expert)?;
        write!(f, "req_craftsmanship: {}, ", self.req_craftsmanship)?;
        write!(f, "req_control: {}, ", self.req_control)?;
        write!(f, "}}")?;
        Ok(())
    }
}
