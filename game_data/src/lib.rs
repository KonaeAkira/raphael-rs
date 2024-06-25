use simulator::{ActionMask, Settings};

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub name: &'static str,
    pub item_level: u32,
    pub can_be_hq: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Ingredient {
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy)]
struct RecipeLevel {
    progress_div: u32,
    quality_div: u32,
    progress_mod: u32,
    quality_mod: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Recipe {
    pub recipe_level: u32,
    pub progress: u32,
    pub quality: u32,
    pub durability: i8,
    pub material_quality_factor: u32,
    pub ingredients: [Ingredient; 6],
}

#[derive(Debug, Clone, Copy)]
pub struct RecipeConfiguration {
    pub item_id: u32,
    pub recipe: Recipe,
    pub hq_ingredients: [u8; 6],
}

#[derive(Debug, Clone, Copy)]
pub struct CrafterConfiguration {
    pub craftsmanship: u16,
    pub control: u16,
    pub cp: u16,
    pub job_level: u8,
    pub manipulation: bool,
}

pub const LEVELS: [u32; 90] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    120, 125, 130, 133, 136, 139, 142, 145, 148, 150, 260, 265, 270, 273, 276, 279, 282, 285, 288,
    290, 390, 395, 400, 403, 406, 409, 412, 415, 418, 420, 517, 520, 525, 530, 535, 540, 545, 550,
    555, 560,
];

const RLVLS: [RecipeLevel; 651] = include!(concat!(env!("OUT_DIR"), "/rlvls.rs"));

pub static ITEMS: phf::OrderedMap<u32, Item> = include!(concat!(env!("OUT_DIR"), "/items.rs"));
pub static RECIPES: phf::OrderedMap<u32, Recipe> =
    include!(concat!(env!("OUT_DIR"), "/recipes.rs"));

pub fn get_game_settings(
    recipe_config: RecipeConfiguration,
    crafter_config: CrafterConfiguration,
) -> Settings {
    let recipe = recipe_config.recipe;
    let rlvl = &RLVLS[recipe.recipe_level as usize];

    let mut base_progress: f64 =
        crafter_config.craftsmanship as f64 * 10.0 / rlvl.progress_div as f64 + 2.0;
    let mut base_quality: f64 =
        crafter_config.control as f64 * 10.0 / rlvl.quality_div as f64 + 35.0;
    if LEVELS[crafter_config.job_level as usize - 1] <= recipe.recipe_level {
        base_progress = base_progress * rlvl.progress_mod as f64 / 100.0;
        base_quality = base_quality * rlvl.quality_mod as f64 / 100.0;
    }

    let hq_ingredients: Vec<(Item, u32)> = recipe
        .ingredients
        .iter()
        .filter_map(|ingredient| match ingredient.item_id {
            0 => None,
            id => {
                let item = *ITEMS.get(&id).unwrap();
                match item.can_be_hq {
                    true => Some((item, ingredient.amount)),
                    false => None,
                }
            }
        })
        .collect();
    let initial_quality = match hq_ingredients.is_empty() {
        true => 0,
        false => {
            // let total_ilvl: u32 = hq_ingredients.iter().map(|(item, max_amount)| item.item_level * max_amount).sum();
            let mut max_ilvl: u64 = 0;
            let mut provided_ilvl: u64 = 0;
            for (index, (item, max_amount)) in hq_ingredients.into_iter().enumerate() {
                max_ilvl += max_amount as u64 * item.item_level as u64;
                provided_ilvl +=
                    recipe_config.hq_ingredients[index] as u64 * item.item_level as u64;
            }
            (recipe.quality as u64 * recipe.material_quality_factor as u64 * provided_ilvl
                / max_ilvl
                / 100) as u32
        }
    };

    Settings {
        max_cp: crafter_config.cp as i16,
        max_durability: recipe.durability as i16,
        max_progress: recipe.progress,
        max_quality: recipe.quality,
        base_progress: base_progress.floor() as u32,
        base_quality: base_quality.floor() as u32,
        initial_quality,
        job_level: crafter_config.job_level,
        allowed_actions: ActionMask::from_level(
            crafter_config.job_level as u32,
            crafter_config.manipulation,
        ),
    }
}

const HQ_LOOKUP: [u8; 101] = [
    1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7, 8, 8, 8,
    9, 9, 9, 10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 14, 14, 14, 15, 15, 15, 16, 16, 17,
    17, 17, 18, 18, 18, 19, 19, 20, 20, 21, 22, 23, 24, 26, 28, 31, 34, 38, 42, 47, 52, 58, 64, 68,
    71, 74, 76, 78, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 94, 96, 98, 100,
];

pub fn hq_percentage(quality: u32, max_quality: u32) -> u8 {
    // TODO: switch to std::num::NonZeroU32 at some point
    assert!(max_quality != 0, "max_quality must be non-zero");
    let ratio = quality as f64 / max_quality as f64;
    HQ_LOOKUP[(ratio * 100.0).floor() as usize]
}
