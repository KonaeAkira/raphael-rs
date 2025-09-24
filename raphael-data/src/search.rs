use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
};

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};

use crate::{
    CL_ICON_CHAR, HQ_ICON_CHAR, MEALS, POTIONS, RECIPES, STELLAR_MISSIONS, get_raw_item_name,
    get_stellar_mission_name,
};

// The matcher allocates a huge chunk of heap memory on creation, so it's best
// to only create and re-use a single instance throughout the lifetime of the program.
static MATCHER: LazyLock<Mutex<nucleo_matcher::Matcher>> = LazyLock::new(|| {
    let config = nucleo_matcher::Config::DEFAULT;
    Mutex::new(nucleo_matcher::Matcher::new(config))
});

#[derive(Debug, Clone, Copy)]
struct MatcherCandidate<T> {
    haystack: &'static str,
    associated_data: T,
}

impl<T> AsRef<str> for MatcherCandidate<T> {
    fn as_ref(&self) -> &str {
        self.haystack
    }
}

fn preprocess_pattern(pattern: &str) -> String {
    pattern
        .chars()
        .filter(|&c| !c.is_whitespace() && c != HQ_ICON_CHAR && c != CL_ICON_CHAR)
        .collect()
}

pub type RecipeSearchEntry = (u32, &'static crate::Recipe);
pub fn find_recipes(
    search_string: &str,
    locale: crate::Locale,
) -> impl Iterator<Item = RecipeSearchEntry> {
    let pattern = Pattern::parse(
        &preprocess_pattern(search_string),
        CaseMatching::Ignore,
        Normalization::Smart,
    );
    let entries = RECIPES.entries().filter_map(|(recipe_id, recipe)| {
        let item_name = get_raw_item_name(recipe.item_id, locale)?;
        Some(MatcherCandidate {
            haystack: item_name,
            associated_data: (recipe_id, recipe),
        })
    });
    let matches = pattern.match_list(entries, MATCHER.lock().as_mut().unwrap());
    matches
        .into_iter()
        .map(|(entry, _score)| entry.associated_data)
}

pub type StellarMissionSearchEntry = (u32, &'static crate::StellarMission);
pub fn find_stellar_missions(
    search_string: &str,
    locale: crate::Locale,
) -> impl Iterator<Item = StellarMissionSearchEntry> {
    let pattern = Pattern::parse(
        &preprocess_pattern(search_string),
        CaseMatching::Ignore,
        Normalization::Smart,
    );
    let mission_entries = STELLAR_MISSIONS
        .entries()
        .filter_map(|(mission_id, mission)| {
            let mission_name = get_stellar_mission_name(mission_id, locale)?;
            Some(MatcherCandidate {
                haystack: mission_name,
                associated_data: (mission_id, mission),
            })
        });
    let recipe_entries = STELLAR_MISSIONS
        .entries()
        .flat_map(|(mission_id, mission)| {
            mission.recipe_ids.iter().filter_map(move |&recipe_id| {
                let recipe = RECIPES.get(recipe_id)?;
                let item_name = get_raw_item_name(recipe.item_id, locale)?;
                Some(MatcherCandidate {
                    haystack: item_name,
                    associated_data: (mission_id, mission),
                })
            })
        });
    let matches = pattern.match_list(
        mission_entries.chain(recipe_entries),
        MATCHER.lock().as_mut().unwrap(),
    );
    let mut unique_matches: HashSet<u32> = HashSet::default();
    matches.into_iter().filter_map(move |(entry, _score)| {
        match unique_matches.insert(entry.associated_data.0) {
            true => Some(entry.associated_data),
            false => None,
        }
    })
}

fn find_consumables(
    search_string: &str,
    locale: crate::Locale,
    consumables: &'static [crate::Consumable],
) -> impl Iterator<Item = &'static crate::Consumable> {
    let pattern = Pattern::parse(
        &preprocess_pattern(search_string),
        CaseMatching::Ignore,
        Normalization::Smart,
    );
    let entries = consumables.iter().filter_map(|consumable| {
        let item_name = get_raw_item_name(consumable.item_id, locale)?;
        Some(MatcherCandidate {
            haystack: item_name,
            associated_data: consumable,
        })
    });
    let matches = pattern.match_list(entries, MATCHER.lock().as_mut().unwrap());
    matches
        .into_iter()
        .map(|(entry, _score)| entry.associated_data)
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
