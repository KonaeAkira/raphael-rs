use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A command-line interface for the Raphael-XIV crafting solver."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for recipes by name
    Search(SearchCli),
    /// Solve a crafting rotation
    Solve(commands::solve::SolveArgs),
    /// Show ingredients for a recipe
    Ingredients(commands::ingredients::IngredientsArgs),
}

#[derive(Parser, Debug)]
#[command(args_conflicts_with_subcommands = true)]
struct SearchCli {
    #[command(subcommand)]
    command: Option<SearchCommands>,

    #[command(flatten)]
    recipe_search_args: commands::search_recipe::SearchArgs,
}

#[derive(Subcommand, Debug)]
enum SearchCommands {
    /// Search recipes. Default if no command is specified
    Recipe(commands::search_recipe::SearchArgs),
    /// Search stellar missions
    Mission(commands::search_mission::SearchArgs),
}

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Search(search_cli) => match &search_cli.command {
            Some(command) => match command {
                SearchCommands::Recipe(args) => commands::search_recipe::execute(args),
                SearchCommands::Mission(args) => commands::search_mission::execute(args),
            },
            None => commands::search_recipe::execute(&search_cli.recipe_search_args),
        },
        Commands::Solve(args) => commands::solve::execute(args),
        Commands::Ingredients(args) => commands::ingredients::execute(args),
    }
}
