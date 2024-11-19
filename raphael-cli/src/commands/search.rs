use clap::Args;
use game_data::{get_item_name, Locale, RECIPES};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search pattern
    pub pattern: String,
}


pub fn execute(args: &SearchArgs) {
    let output_separator = std::env::var("OFS").unwrap_or(" ".to_string());
    let matches = game_data::find_recipes(&args.pattern, Locale::EN);
    if matches.is_empty() {
        println!("No matches found");
        return;
    }

    for recipe_idx in matches {
        let recipe = &RECIPES[recipe_idx];
        let name = get_item_name(recipe.item_id, false, Locale::EN);
        println!(
            "{item_id}{separator}{name}",
            item_id = recipe.item_id,
            separator = output_separator,
            name = name
        );
    }
}
