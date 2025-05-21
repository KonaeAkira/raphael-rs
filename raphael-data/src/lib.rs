mod consumables;
pub use consumables::*;

mod config;
pub use config::*;

mod locales;
pub use locales::*;

mod search;
pub use search::*;

use raphael_sim::{Action, ActionMask, Settings};

pub const HQ_ICON_CHAR: char = '\u{e03c}';
pub const CL_ICON_CHAR: char = '\u{e03d}';

#[derive(Debug, Clone, Copy, Default)]
pub struct Item {
    pub item_level: u16,
    pub can_be_hq: bool,
    pub always_collectable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ingredient {
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct RecipeLevel {
    pub job_level: u8,
    pub max_progress: u32,
    pub max_quality: u32,
    pub max_durability: u16,
    pub progress_div: u32,
    pub quality_div: u32,
    pub progress_mod: u32,
    pub quality_mod: u32,
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CustomRecipeOverrides {
    pub max_progress_override: u16,
    pub max_quality_override: u16,
    pub max_durability_override: u16,
    pub base_progress_override: Option<u16>,
    pub base_quality_override: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Recipe {
    pub job_id: u8,
    pub item_id: u32,
    pub max_level_scaling: u8,
    pub recipe_level: u16,
    pub progress_factor: u32,
    pub quality_factor: u32,
    pub durability_factor: u16,
    pub material_factor: u16,
    pub ingredients: [Ingredient; 6],
    pub is_expert: bool,
    pub req_craftsmanship: u16,
    pub req_control: u16,
}

pub const RLVLS: &[RecipeLevel] = include!("../data/rlvls.rs");
pub const LEVEL_ADJUST_TABLE: &[u16] = include!("../data/level_adjust_table.rs");
pub static RECIPES: phf::OrderedMap<u32, Recipe> = include!("../data/recipes.rs");
pub const ITEMS: phf::OrderedMap<u32, Item> = include!("../data/items.rs");

pub fn get_game_settings(
    recipe: Recipe,
    custom_recipe_overrides: Option<CustomRecipeOverrides>,
    crafter_stats: CrafterStats,
    food: Option<Consumable>,
    potion: Option<Consumable>,
) -> Settings {
    let rlvl = if recipe.max_level_scaling != 0 {
        let job_level = std::cmp::min(recipe.max_level_scaling, crafter_stats.level);
        LEVEL_ADJUST_TABLE[job_level as usize] as usize
    } else {
        recipe.recipe_level as usize
    };

    let mut rlvl_record = RLVLS[rlvl];
    if recipe.max_level_scaling != 0 {
        // https://github.com/KonaeAkira/raphael-rs/pull/126#issuecomment-2832041490
        rlvl_record.max_durability = 80;
    }

    let craftsmanship = crafter_stats.craftsmanship
        + craftsmanship_bonus(crafter_stats.craftsmanship, &[food, potion]);
    let control = crafter_stats.control + control_bonus(crafter_stats.control, &[food, potion]);
    let cp = crafter_stats.cp + cp_bonus(crafter_stats.cp, &[food, potion]);

    let mut base_progress = craftsmanship as f32 * 10.0 / rlvl_record.progress_div as f32 + 2.0;
    let mut base_quality = control as f32 * 10.0 / rlvl_record.quality_div as f32 + 35.0;
    if crafter_stats.level <= rlvl_record.job_level {
        base_progress = base_progress * rlvl_record.progress_mod as f32 / 100.0;
        base_quality = base_quality * rlvl_record.quality_mod as f32 / 100.0;
    }

    let mut allowed_actions = ActionMask::all();
    if !crafter_stats.manipulation {
        allowed_actions = allowed_actions.remove(Action::Manipulation);
    }
    if recipe.is_expert || crafter_stats.level < rlvl_record.job_level + 10 {
        allowed_actions = allowed_actions.remove(Action::TrainedEye);
    }
    if !crafter_stats.heart_and_soul {
        allowed_actions = allowed_actions.remove(Action::HeartAndSoul);
    }
    if !crafter_stats.quick_innovation {
        allowed_actions = allowed_actions.remove(Action::QuickInnovation);
    }

    match custom_recipe_overrides {
        Some(overrides) => Settings {
            max_cp: cp as _,
            max_durability: overrides.max_durability_override,
            max_progress: overrides.max_progress_override,
            max_quality: overrides.max_quality_override,
            base_progress: match overrides.base_progress_override {
                Some(override_value) => override_value,
                None => base_progress as u16,
            },
            base_quality: match overrides.base_quality_override {
                Some(override_value) => override_value,
                None => base_quality as u16,
            },
            job_level: crafter_stats.level,
            allowed_actions,
            adversarial: false,
            backload_progress: false,
        },
        None => Settings {
            max_cp: cp as _,
            max_durability: rlvl_record.max_durability * recipe.durability_factor / 100,
            max_progress: (rlvl_record.max_progress * recipe.progress_factor / 100) as u16,
            max_quality: (rlvl_record.max_quality * recipe.quality_factor / 100) as u16,
            base_progress: base_progress as u16,
            base_quality: base_quality as u16,
            job_level: crafter_stats.level,
            allowed_actions,
            adversarial: false,
            backload_progress: false,
        },
    }
}

pub fn get_initial_quality(
    crafter_stats: CrafterStats,
    recipe: Recipe,
    hq_ingredients: [u8; 6],
) -> u16 {
    let ingredients: Vec<(Item, u32)> = recipe
        .ingredients
        .iter()
        .filter_map(|ingredient| Some((*ITEMS.get(&ingredient.item_id)?, ingredient.amount)))
        .collect();

    let mut max_ilvl = 0;
    let mut provided_ilvl = 0;
    for (index, (item, max_amount)) in ingredients.into_iter().enumerate() {
        if item.can_be_hq {
            max_ilvl += max_amount as u16 * item.item_level;
            provided_ilvl += hq_ingredients[index] as u16 * item.item_level;
        }
    }

    let rlvl = if recipe.max_level_scaling != 0 {
        let job_level = std::cmp::min(recipe.max_level_scaling, crafter_stats.level);
        RLVLS
            .iter()
            .position(|rlvl_record| rlvl_record.job_level == job_level)
            .unwrap()
    } else {
        recipe.recipe_level as usize
    };
    let rlvl_record = &RLVLS[rlvl];
    let max_quality = rlvl_record.max_quality * recipe.quality_factor / 100;

    if max_ilvl != 0 {
        (max_quality as u64 * recipe.material_factor as u64 * provided_ilvl as u64
            / max_ilvl as u64
            / 100) as u16
    } else {
        0
    }
}

const HQ_LOOKUP: [u8; 101] = [
    1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8,
    9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 14, 14, 14, 15, 15, 15, 16, 16, 17,
    17, 17, 18, 18, 18, 19, 19, 20, 20, 21, 22, 23, 24, 26, 28, 31, 34, 38, 42, 47, 52, 58, 64, 68,
    71, 74, 76, 78, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 94, 96, 98, 100,
];

pub fn hq_percentage(quality: impl Into<u32>, max_quality: impl Into<u32>) -> Option<u8> {
    let quality: u32 = quality.into();
    let max_quality: u32 = max_quality.into();
    let ratio = (quality * 100).checked_div(max_quality)?;
    Some(HQ_LOOKUP[std::cmp::min(ratio as usize, 100)])
}
