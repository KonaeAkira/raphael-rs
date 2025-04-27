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
    Search(commands::search::SearchArgs),
    /// Solve a crafting rotation
    Solve(commands::solve::SolveArgs),
}

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Search(args) => commands::search::execute(args),
        Commands::Solve(args) => commands::solve::execute(args),
    }
}
