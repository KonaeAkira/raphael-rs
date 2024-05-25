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
pub struct Recipe {
    pub recipe_level: u32,
    pub progress: u32,
    pub quality: u32,
    pub durability: i8,
    pub progress_div: u32,
    pub progress_mod: u32,
    pub quality_div: u32,
    pub quality_mod: u32,
    pub material_quality_factor: u32,
    pub ingredients: [Ingredient; 6],
}

pub const LEVELS: [u32; 90] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    120, 125, 130, 133, 136, 139, 142, 145, 148, 150, 260, 265, 270, 273, 276, 279, 282, 285, 288,
    290, 390, 395, 400, 403, 406, 409, 412, 415, 418, 420, 517, 520, 525, 530, 535, 540, 545, 550,
    555, 560,
];

pub static ITEM_IDS: phf::OrderedMap<&'static str, u32> =
    include!(concat!(env!("OUT_DIR"), "/item_ids.rs"));
pub static ITEMS: phf::OrderedMap<u32, Item> = include!(concat!(env!("OUT_DIR"), "/items.rs"));
pub static RECIPES: phf::OrderedMap<u32, Recipe> = include!(concat!(env!("OUT_DIR"), "/recipes.rs"));

pub fn get_craftable_item_names() -> impl Iterator<Item = &'static str> {
    RECIPES
        .keys()
        .into_iter()
        .map(|item_id| ITEMS.get(item_id).unwrap().name)
}

pub fn get_ingredients(item_name: String) -> [Ingredient; 6] {
    let item_id = ITEM_IDS.get(&item_name).unwrap();
    let recipe = RECIPES.get(item_id).unwrap();
    recipe.ingredients
}
