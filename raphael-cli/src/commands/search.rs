use clap::{Args, ValueEnum};
use raphael_data::{Locale, RECIPES, Recipe, get_item_name};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search pattern to use for search though names, can be partial name
    #[arg(short, long, required_unless_present_any(["recipe_id", "item_id"]), conflicts_with_all(["recipe_id", "item_id"]))]
    pub pattern: Option<String>,

    /// Recipe ID to search for
    #[arg(short, long, required_unless_present_any(["pattern", "item_id"]), conflicts_with = "item_id")]
    pub recipe_id: Option<u32>,

    /// Recipe ID to search for
    #[arg(short, long, required_unless_present_any(["pattern", "recipe_id"]))]
    pub item_id: Option<u32>,

    /// The delimiter the output uses between fields
    #[arg(long, alias = "OFS", default_value = " ", env = "OFS")]
    output_field_separator: String,

    /// The language the input pattern and output use
    #[arg(short, long, alias = "locale", value_enum, ignore_case = true, default_value_t = SearchLanguage::EN)]
    language: SearchLanguage,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum SearchLanguage {
    EN,
    DE,
    FR,
    JP,
}

impl Into<Locale> for SearchLanguage {
    fn into(self) -> Locale {
        match self {
            SearchLanguage::EN => Locale::EN,
            SearchLanguage::DE => Locale::DE,
            SearchLanguage::FR => Locale::FR,
            SearchLanguage::JP => Locale::JP,
        }
    }
}

pub fn execute(args: &SearchArgs) {
    let locale = args.language.into();
    let mut matches: Vec<raphael_data::Recipe>;
    if args.pattern.is_some() {
        matches = raphael_data::find_recipes(&args.pattern.clone().unwrap(), locale).iter().map(|index| RECIPES[index]).collect();
    } else if args.recipe_id.is_some() {
        matches = Vec::new();
        matches.push(*raphael_data::RECIPES.entries().find(|(id, _)| **id == args.recipe_id.unwrap()).map(|(_, recipe)| recipe).unwrap());
    } else {
        log::warn!("Item IDs do not uniquely corresponds to a specific recipe config. Consider using the recipe ID instead.");
        matches = raphael_data::RECIPES.values().filter(|recipe| recipe.item_id == args.item_id.unwrap()).map(|recipe| *recipe).collect();
    }
    if matches.is_empty() {
        println!("No matches found");
        return;
    }

    for recipe in matches {
        let name =
            get_item_name(recipe.item_id, false, locale).unwrap_or("Unknown item".to_owned());
        println!(
            "{recipe_id}{separator}{job_name}{separator}{item_id}{separator}{name}",
            recipe_id = recipe.id,
            job_name = raphael_data::get_job_name(recipe.job_id, locale),
            item_id = recipe.item_id,
            separator = args.output_field_separator,
            name = name.trim_end_matches(&[' ', raphael_data::CL_ICON_CHAR])
        );
    }
}
