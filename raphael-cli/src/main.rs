use clap::{Parser, Subcommand};
use game_data::{get_game_settings, get_item_name, CrafterStats, Locale, RECIPES};
use simulator::SimulationState;
use solvers::MacroSolver;

#[derive(Parser, Debug)]
#[command(version, about = "A command-line interface for the Raphael-XIV crafting solver.")]
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
            let matches = game_data::find_recipes(&pattern, Locale::EN);
            if matches.is_empty() {
                println!("No matches found");
                return;
            }

            println!("Found {} matches:", matches.len());
            for recipe_idx in matches {
                let recipe = &RECIPES[recipe_idx];
                let name = get_item_name(recipe.item_id, false, Locale::EN);
                println!("{}: {}", recipe.item_id, name);
            }
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
            let recipe = RECIPES
                .iter()
                .find(|r| r.item_id == item_id)
                .expect("Recipe not found");

            let crafter_stats = CrafterStats {
                craftsmanship,
                control,
                cp,
                level,
                manipulation,
                heart_and_soul,
                quick_innovation,
            };

            let settings = get_game_settings(*recipe, crafter_stats, None, None, adversarial);
            let state = SimulationState::new(&settings);

            let mut solver = MacroSolver::new(
                settings,
                backload_progress,
                unsound,
                Box::new(|_| {}),
                Box::new(|_| {}),
            );
            let actions = solver.solve(state).expect("Failed to solve");

            let final_state = SimulationState::from_macro(&settings, &actions).unwrap();
            let quality = final_state.quality;
            let steps = actions.len();
            let duration: i16 = actions.iter().map(|action| action.time_cost()).sum();

            println!("Item ID: {}", recipe.item_id);
            println!("Quality: {}/{}", quality, settings.max_quality);
            println!(
                "Progress: {}/{}",
                final_state.progress, settings.max_progress
            );
            println!("Steps: {}", steps);
            println!("Duration: {} seconds", duration);
            println!("\nActions:");
            for action in actions {
                println!("{:?}", action);
            }
        }
    }
}
