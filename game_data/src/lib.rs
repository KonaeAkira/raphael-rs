mod consumables;
pub use consumables::*;

mod config;
pub use config::*;

mod locales;
pub use locales::*;

use serde::{Deserialize, Serialize};
use simulator::{ActionMask, Settings};

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub item_level: u16,
    pub can_be_hq: bool,
    pub is_collectable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Ingredient {
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct RecipeLevel {
    pub progress_div: u16,
    pub quality_div: u16,
    pub progress_mod: u16,
    pub quality_mod: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Recipe {
    pub job_id: u8,
    pub item_id: u32,
    pub level: u8,
    pub recipe_level: u16,
    pub progress: u16,
    pub quality: u16,
    pub durability: i8,
    pub material_quality_factor: u16,
    pub ingredients: [Ingredient; 6],
    pub is_expert: bool,
}

pub const RLVLS: [RecipeLevel; 800] = include!(concat!(env!("OUT_DIR"), "/rlvls.rs"));
pub const RECIPES: &[Recipe] = include!(concat!(env!("OUT_DIR"), "/recipes.rs"));

pub static ITEMS: phf::OrderedMap<u32, Item> = include!(concat!(env!("OUT_DIR"), "/items.rs"));

pub fn get_game_settings(
    recipe: Recipe,
    crafter_stats: CrafterStats,
    food: Option<Consumable>,
    potion: Option<Consumable>,
    adversarial: bool,
) -> Settings {
    let rlvl = &RLVLS[recipe.recipe_level as usize];

    let craftsmanship = crafter_stats.craftsmanship
        + craftsmanship_bonus(crafter_stats.craftsmanship, &[food, potion]);
    let control = crafter_stats.control + control_bonus(crafter_stats.control, &[food, potion]);
    let cp = crafter_stats.cp + cp_bonus(crafter_stats.cp, &[food, potion]);

    let mut base_progress = craftsmanship * 10 / rlvl.progress_div + 2;
    let mut base_quality = control * 10 / rlvl.quality_div + 35;
    if crafter_stats.level <= recipe.level {
        base_progress = base_progress * rlvl.progress_mod / 100;
        base_quality = base_quality * rlvl.quality_mod / 100;
    }

    Settings {
        max_cp: cp as _,
        max_durability: recipe.durability as _,
        max_progress: recipe.progress,
        max_quality: recipe.quality,
        base_progress,
        base_quality,
        job_level: crafter_stats.level,
        allowed_actions: ActionMask::from_level(
            crafter_stats.level as _,
            crafter_stats.manipulation,
            !recipe.is_expert && crafter_stats.level >= recipe.level + 10, // Trained Eye condition
        ),
        adversarial,
    }
}

pub fn get_initial_quality(recipe: Recipe, hq_ingredients: [u8; 6]) -> u16 {
    let ingredients: Vec<(Item, u32)> = recipe
        .ingredients
        .iter()
        .filter_map(|ingredient| match ingredient.item_id {
            0 => None,
            id => Some((*ITEMS.get(&id).unwrap(), ingredient.amount)),
        })
        .collect();

    let mut max_ilvl = 0;
    let mut provided_ilvl = 0;
    for (index, (item, max_amount)) in ingredients.into_iter().enumerate() {
        if item.can_be_hq {
            max_ilvl += max_amount as u16 * item.item_level;
            provided_ilvl += hq_ingredients[index] as u16 * item.item_level;
        }
    }

    if max_ilvl != 0 {
        (recipe.quality as u64 * recipe.material_quality_factor as u64 * provided_ilvl as u64
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

pub fn hq_percentage(quality: u16, max_quality: u16) -> u8 {
    // TODO: switch to std::num::NonZeroU32 at some point
    assert!(max_quality != 0, "max_quality must be non-zero");
    let ratio = quality as f64 / max_quality as f64;
    HQ_LOOKUP[(ratio * 100.0).floor() as usize]
}
