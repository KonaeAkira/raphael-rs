use clap::Args;
use game_data::{get_game_settings, CrafterStats, RECIPES};
use simulator::SimulationState;
use solvers::MacroSolver;

#[derive(Args, Debug)]
pub struct SolveArgs {
    /// Item ID
    #[arg(short, long)]
    pub item_id: u32,

    /// Craftsmanship rating
    #[arg(short, long)]
    pub craftsmanship: u16,

    /// Control rating
    #[arg(short = 'o', long)]
    pub control: u16,

    /// Crafting points
    #[arg(short = 'p', long)]
    pub cp: u16,

    /// Crafter level
    #[arg(short, long, default_value_t = 100)]
    pub level: u8,

    /// Enable Manipulation
    #[arg(short, long, default_value_t = false)]
    pub manipulation: bool,

    /// Enable Heart and Soul
    #[arg(short = 's', long, default_value_t = false)]
    pub heart_and_soul: bool,

    /// Enable Quick Innovation
    #[arg(short, long, default_value_t = false)]
    pub quick_innovation: bool,

    /// Enable adversarial simulator (ensure 100% reliability)
    #[arg(long, default_value_t = false)]
    pub adversarial: bool,

    /// Only use Progress-increasing actions at the end of the macro
    #[arg(long, default_value_t = false)]
    pub backload_progress: bool,

    /// Enable unsound branch pruning
    #[arg(long, default_value_t = false)]
    pub unsound: bool,
}



pub fn execute(args: &SolveArgs) {
    let recipe = RECIPES
        .iter()
        .find(|r| r.item_id == args.item_id)
        .expect("Recipe not found");

    let crafter_stats = CrafterStats {
        craftsmanship: args.craftsmanship,
        control: args.control,
        cp: args.cp,
        level: args.level,
        manipulation: args.manipulation,
        heart_and_soul: args.heart_and_soul,
        quick_innovation: args.quick_innovation,
    };

    let settings = get_game_settings(*recipe, crafter_stats, None, None, args.adversarial);
    let state = SimulationState::new(&settings);

    let mut solver = MacroSolver::new(
        settings,
        args.backload_progress,
        args.unsound,
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
