use clap::{Args, ValueEnum};
use raphael_data::{Locale, RECIPES, get_item_name, get_job_name};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search string to use, can be partial name
    #[arg(short, long, required_unless_present_any(["recipe_id", "item_id"]), conflicts_with_all(["recipe_id", "item_id"]))]
    pub pattern: Option<String>,

    /// Recipe ID to search for
    #[arg(short, long, required_unless_present_any(["pattern", "item_id"]), conflicts_with = "item_id")]
    pub recipe_id: Option<u32>,

    /// Item ID to search for
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

impl From<SearchLanguage> for Locale {
    fn from(val: SearchLanguage) -> Self {
        match val {
            SearchLanguage::EN => Locale::EN,
            SearchLanguage::DE => Locale::DE,
            SearchLanguage::FR => Locale::FR,
            SearchLanguage::JP => Locale::JP,
        }
    }
}

pub fn execute(args: &SearchArgs) {
    let locale = args.language.into();
    let matches = if args.pattern.is_some() {
        raphael_data::find_recipes(&args.pattern.clone().unwrap(), locale)
            .iter()
            .map(|recipe_id| RECIPES.get_entry(recipe_id).unwrap())
            .collect()
    } else if args.recipe_id.is_some() {
        if let Some(entry) = RECIPES
            .entries()
            .find(|(id, _)| **id == args.recipe_id.unwrap())
        {
            vec![entry]
        } else {
            Vec::new()
        }
    } else {
        log::warn!(
            "Item IDs do not uniquely corresponds to a specific recipe config. Consider using the recipe ID instead."
        );
        raphael_data::RECIPES
            .entries()
            .filter(|(_, recipe)| recipe.item_id == args.item_id.unwrap())
            .collect()
    };
    if matches.is_empty() {
        println!("No matches found");
        return;
    }

    for (recipe_id, recipe) in matches {
        let name =
            get_item_name(recipe.item_id, false, locale).unwrap_or("Unknown item".to_owned());
        println!(
            "{recipe_id}{separator}{job_name}{separator}{item_id}{separator}{name}",
            recipe_id = recipe_id,
            job_name = get_job_name(recipe.job_id, locale),
            item_id = recipe.item_id,
            separator = args.output_field_separator,
            name = name.trim_end_matches([' ', raphael_data::CL_ICON_CHAR])
        );
    }
}
