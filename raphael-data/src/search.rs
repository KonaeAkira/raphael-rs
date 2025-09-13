use unicode_normalization::UnicodeNormalization;

use crate::{
    CL_ICON_CHAR, Consumable, HQ_ICON_CHAR, Locale, MEALS, POTIONS, RECIPES, STELLAR_MISSIONS,
    get_item_name, get_stellar_mission_name,
};

fn is_subsequence(text: impl Iterator<Item = char>, pattern: impl Iterator<Item = char>) -> bool {
    let mut pattern = pattern.peekable();
    for text_char in text {
        pattern.next_if_eq(&text_char);
    }
    pattern.peek().is_none()
}

fn preprocess_text(pattern: &str) -> String {
    pattern
        .to_lowercase()
        .replace([HQ_ICON_CHAR, CL_ICON_CHAR], "")
        .nfd() // Unicode Normalization Form D (canonical decomposition)
        .collect()
}

pub fn find_recipes(search_string: &str, locale: Locale) -> Vec<u32> {
    let pattern = preprocess_text(search_string);
    RECIPES
        .entries()
        .filter_map(|(recipe_id, recipe)| {
            let item_name = preprocess_text(&get_item_name(recipe.item_id, false, locale)?);
            match is_subsequence(item_name.chars(), pattern.chars()) {
                true => Some(*recipe_id),
                false => None,
            }
        })
        .collect()
}

pub fn find_stellar_missions(search_string: &str, locale: Locale) -> Vec<u32> {
    let pattern = preprocess_text(search_string);
    STELLAR_MISSIONS
        .entries()
        .filter_map(|(mission_id, mission)| {
            let mission_name = preprocess_text(&get_stellar_mission_name(*mission_id, locale)?);
            match is_subsequence(mission_name.chars(), pattern.chars()) {
                true => Some(*mission_id),
                false => mission
                    .recipe_ids
                    .iter()
                    .filter_map(|recipe_id| {
                        let recipe = RECIPES.get(recipe_id)?;
                        let item_name =
                            preprocess_text(&get_item_name(recipe.item_id, false, locale)?);
                        match is_subsequence(item_name.chars(), pattern.chars()) {
                            true => Some(*mission_id),
                            false => None,
                        }
                    })
                    .next(),
            }
        })
        .collect()
}

fn find_consumables(search_string: &str, locale: Locale, consumables: &[Consumable]) -> Vec<usize> {
    let pattern = preprocess_text(search_string);
    consumables
        .iter()
        .enumerate()
        .filter_map(|(index, consumable)| {
            let item_name = preprocess_text(&get_item_name(consumable.item_id, false, locale)?);
            match is_subsequence(item_name.chars(), pattern.chars()) {
                true => Some(index),
                false => None,
            }
        })
        .collect()
}

pub fn find_meals(search_string: &str, locale: Locale) -> Vec<usize> {
    find_consumables(search_string, locale, MEALS)
}

pub fn find_potions(search_string: &str, locale: Locale) -> Vec<usize> {
    find_consumables(search_string, locale, POTIONS)
}
