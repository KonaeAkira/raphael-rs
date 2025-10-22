use clap::Args;
use raphael_data::{RECIPES, Recipe, STELLAR_MISSIONS, get_job_name, get_raw_item_name};

use crate::commands::Language;

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search string to use, can be partial name
    #[arg(short, long, required_unless_present_any(["recipe_id", "item_id", "mission_id"]), conflicts_with_all(["recipe_id", "item_id"]))]
    pub pattern: Option<String>,

    /// Recipe ID to search for
    #[arg(short, long, required_unless_present_any(["pattern", "item_id", "mission_id"]), conflicts_with = "item_id")]
    pub recipe_id: Option<u32>,

    /// Item ID to search for
    #[arg(short, long, required_unless_present_any(["pattern", "recipe_id", "mission_id"]))]
    pub item_id: Option<u32>,

    /// Stellar mission ID to list associated recipes for
    #[arg(long, required_unless_present_any(["pattern", "recipe_id", "item_id"]), conflicts_with_all(["pattern", "recipe_id", "item_id"]))]
    pub mission_id: Option<u32>,

    /// The delimiter the output uses between fields
    #[arg(long, alias = "OFS", default_value = " ", env = "OFS")]
    output_field_separator: String,

    /// The language the input pattern and output use
    #[arg(short, long, alias = "locale", value_enum, ignore_case = true, default_value_t = Language::EN)]
    language: Language,
}

pub fn execute(args: &SearchArgs) {
    let locale = args.language.into();
    let mut matches: Vec<(u32, &Recipe)> = Vec::new();

    if let Some(mission_id_arg) = args.mission_id {
        match STELLAR_MISSIONS.get(mission_id_arg) {
            Some(mission) => {
                for recipe_id in mission.recipe_ids {
                    match RECIPES.get(*recipe_id) {
                        Some(recipe) => matches.push((*recipe_id, recipe)),
                        None => log::warn!(
                            "Mission {} references missing recipe id {}",
                            mission_id_arg,
                            recipe_id
                        ),
                    }
                }
            }
            None => {
                println!("No such mission: {}", mission_id_arg);
                return;
            }
        }
    }
    if let Some(pattern_arg) = &args.pattern {
        matches.extend(raphael_data::find_recipes(pattern_arg, locale));
    }
    if let Some(recipe_id_arg) = args.recipe_id {
        matches.extend(
            RECIPES
                .get(recipe_id_arg)
                .map(|recipe| (recipe_id_arg, recipe)),
        );
    }
    if let Some(item_id_arg) = args.item_id {
        log::warn!(
            "Item IDs do not uniquely corresponds to a specific recipe config. Consider using the recipe ID instead."
        );
        matches.extend(
            raphael_data::RECIPES
                .entries()
                .filter_map(|(recipe_id, recipe)| {
                    if recipe.item_id == item_id_arg {
                        Some((recipe_id, recipe))
                    } else {
                        None
                    }
                }),
        );
    };
    if matches.is_empty() {
        println!("No matches found");
        return;
    }

    for (recipe_id, recipe) in matches {
        let name = get_raw_item_name(recipe.item_id, locale).unwrap_or("Unknown item");
        println!(
            "{recipe_id}{separator}{job_name}{separator}{item_id}{separator}{name}",
            job_name = get_job_name(recipe.job_id, locale),
            item_id = recipe.item_id,
            separator = args.output_field_separator,
        );
    }
}
