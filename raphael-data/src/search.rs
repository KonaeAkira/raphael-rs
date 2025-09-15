use unicode_normalization::UnicodeNormalization;

use crate::{
    CL_ICON_CHAR, HQ_ICON_CHAR, MEALS, POTIONS, RECIPES, STELLAR_MISSIONS, get_raw_item_name,
    get_stellar_mission_name,
};

fn is_subsequence(text: impl Iterator<Item = char>, pattern: impl Iterator<Item = char>) -> bool {
    let mut pattern = pattern.peekable();
    for text_char in text {
        pattern.next_if_eq(&text_char);
    }
    pattern.peek().is_none()
}

fn preprocess_text(pattern: &str) -> impl Iterator<Item = char> {
    pattern
        .chars()
        .filter(|&c| !c.is_whitespace() && c != HQ_ICON_CHAR && c != CL_ICON_CHAR)
        .flat_map(char::to_lowercase)
        .nfd() // Unicode Normalization Form D (canonical decomposition)
}

pub type RecipeSearchEntry = (u32, &'static crate::Recipe);
pub fn find_recipes(
    search_string: &str,
    locale: crate::Locale,
) -> impl Iterator<Item = RecipeSearchEntry> {
    let pattern = preprocess_text(search_string).collect::<String>();
    RECIPES.entries().filter_map(move |(recipe_id, recipe)| {
        let item_name = get_raw_item_name(recipe.item_id, locale)?;
        match is_subsequence(preprocess_text(item_name), pattern.chars()) {
            true => Some((recipe_id, recipe)),
            false => None,
        }
    })
}

pub type StellarMissionSearchEntry = (u32, &'static crate::StellarMission);
pub fn find_stellar_missions(
    search_string: &str,
    locale: crate::Locale,
) -> impl Iterator<Item = StellarMissionSearchEntry> {
    let pattern = preprocess_text(search_string).collect::<String>();
    STELLAR_MISSIONS
        .entries()
        .filter_map(move |(mission_id, mission)| {
            let mission_name = get_stellar_mission_name(mission_id, locale)?;
            match is_subsequence(preprocess_text(mission_name), pattern.chars()) {
                true => Some((mission_id, mission)),
                false => mission
                    .recipe_ids
                    .iter()
                    .filter_map(|recipe_id| {
                        let recipe = RECIPES.get(*recipe_id)?;
                        let item_name = get_raw_item_name(recipe.item_id, locale)?;
                        match is_subsequence(preprocess_text(item_name), pattern.chars()) {
                            true => Some((mission_id, mission)),
                            false => None,
                        }
                    })
                    .next(),
            }
        })
}

fn find_consumables(
    search_string: &str,
    locale: crate::Locale,
    consumables: &'static [crate::Consumable],
) -> impl Iterator<Item = &'static crate::Consumable> {
    let pattern = preprocess_text(search_string).collect::<String>();
    consumables.iter().filter_map(move |consumable| {
        let item_name = get_raw_item_name(consumable.item_id, locale)?;
        match is_subsequence(preprocess_text(item_name), pattern.chars()) {
            true => Some(consumable),
            false => None,
        }
    })
}

pub fn find_meals(
    search_string: &str,
    locale: crate::Locale,
) -> impl Iterator<Item = &'static crate::Consumable> {
    find_consumables(search_string, locale, MEALS)
}

pub fn find_potions(
    search_string: &str,
    locale: crate::Locale,
) -> impl Iterator<Item = &'static crate::Consumable> {
    find_consumables(search_string, locale, POTIONS)
}
