use clap::Args;
use game_data::{get_item_name, Locale, RECIPES};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search pattern
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

impl Into::<Locale> for SearchLanguage {
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
    matches = game_data::find_recipes(&args.pattern, locale);
    if matches.is_empty() {
        println!("No matches found");
        return;
    }

    for recipe_idx in matches {
        let recipe = &RECIPES[recipe_idx];
        let name = get_item_name(recipe.item_id, false, locale);
        println!(
            "{item_id}{separator}{name}",
            item_id = recipe.item_id,
            separator = args.output_field_separator,
            name = name
        );
    }
}
