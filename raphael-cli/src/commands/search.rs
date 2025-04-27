use clap::{Args, ValueEnum};
use raphael_data::{Locale, RECIPES, Recipe, get_item_name};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search pattern, <PATTERN> can be a string or an item ID
    pub pattern: String,

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
    let matches: Vec<Recipe> = if let Ok(item_id) = u32::from_str_radix(&args.pattern, 10) {
        match RECIPES.values().find(|recipe| recipe.item_id == item_id) {
            Some(recipe) => vec![*recipe],
            None => Vec::new(),
        }
    } else {
        raphael_data::find_recipes(&args.pattern, locale)
            .into_iter()
            .map(|recipe_id| RECIPES[&recipe_id])
            .collect()
    };

    if matches.is_empty() {
        println!("No matches found");
        return;
    }

    for recipe in matches {
        let name =
            get_item_name(recipe.item_id, false, locale).unwrap_or("Unknown item".to_owned());
        println!(
            "{item_id}{separator}{name}",
            item_id = recipe.item_id,
            separator = args.output_field_separator,
            name = name.trim_end_matches(&[' ', raphael_data::CL_ICON_CHAR])
        );
    }
}
