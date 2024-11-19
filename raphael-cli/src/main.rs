use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "A command-line interface for the Raphael-XIV crafting solver."
)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Search for recipes by name
    Search {
        /// Search pattern
        pattern: String,
    },
    /// Solve a crafting rotation
    Solve {
        /// Item ID
        #[arg(short, long)]
        item_id: u32,

        /// Craftsmanship rating
        #[arg(short, long)]
        craftsmanship: u16,

        /// Control rating
        #[arg(short = 'o', long)]
        control: u16,

        /// Crafting points
        #[arg(short = 'p', long)]
        cp: u16,

        /// Crafter level
        #[arg(short, long, default_value_t = 100)]
        level: u8,

        /// Enable Manipulation
        #[arg(short, long, default_value_t = false)]
        manipulation: bool,

        /// Enable Heart and Soul
        #[arg(short = 's', long, default_value_t = false)]
        heart_and_soul: bool,

        /// Enable Quick Innovation
        #[arg(short, long, default_value_t = false)]
        quick_innovation: bool,

        /// Enable adversarial simulator (ensure 100% reliability)
        #[arg(long, default_value_t = false)]
        adversarial: bool,

        /// Only use Progress-increasing actions at the end of the macro
        #[arg(long, default_value_t = false)]
        backload_progress: bool,

        /// Enable unsound branch pruning
        #[arg(long, default_value_t = false)]
        unsound: bool,
    },
}

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let args = Args::parse();

    match args.command {
        Commands::Search { pattern } => {
            commands::search::execute(pattern);
        }
        Commands::Solve {
            item_id,
            craftsmanship,
            control,
            cp,
            level,
            manipulation,
            heart_and_soul,
            quick_innovation,
            adversarial,
            backload_progress,
            unsound,
        } => {
            commands::solve::execute(
                item_id,
                craftsmanship,
                control,
                cp,
                level,
                manipulation,
                heart_and_soul,
                quick_innovation,
                adversarial,
                backload_progress,
                unsound,
            );
        }
    }
}
