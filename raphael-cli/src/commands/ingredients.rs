use clap::Args;
use raphael_data::{RECIPES, get_job_name, get_raw_item_name};

use crate::commands::Language;

#[derive(Args, Debug)]
pub struct IngredientsArgs {
    /// Recipe ID to get ingredients for
    #[arg(short, long)]
    pub recipe_id: u32,

    /// The language the output uses
    #[arg(short, long, alias = "locale", value_enum, ignore_case = true, default_value_t = Language::EN)]
    language: Language,

    /// The delimiter the output uses between fields
    #[arg(long, alias = "OFS", default_value = " ", env = "OFS")]
    output_field_separator: String,
}

pub fn execute(args: &IngredientsArgs) {
    let locale = args.language.into();

    // Get the recipe by ID
    let recipe = match RECIPES.get(args.recipe_id) {
        Some(recipe) => recipe,
        None => {
            println!("Recipe with ID {} not found", args.recipe_id);
            return;
        }
    };

    // Get the recipe item name
    let recipe_name = get_raw_item_name(recipe.item_id, locale).unwrap_or("Unknown item");
    let job_name = get_job_name(recipe.job_id, locale);

    // Print recipe header
    println!("Recipe ID: {}", args.recipe_id);
    println!("Recipe: {} ({})", recipe_name, job_name);
    println!("Item ID: {}", recipe.item_id);
    println!();

    // Print ingredients
    println!("Ingredients:");
    let mut has_ingredients = false;

    for ingredient in recipe.ingredients {
        if ingredient.item_id != 0 {
            has_ingredients = true;
            let ingredient_name =
                get_raw_item_name(ingredient.item_id, locale).unwrap_or("Unknown item");

            println!(
                "{amount}{separator}{item_id}{separator}{name}",
                amount = ingredient.amount,
                item_id = ingredient.item_id,
                name = ingredient_name,
                separator = args.output_field_separator
            );
        }
    }

    if !has_ingredients {
        println!("No ingredients found for this recipe.");
    }
}
