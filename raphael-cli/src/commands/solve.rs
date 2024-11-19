use game_data::{get_game_settings, CrafterStats, RECIPES};
use simulator::SimulationState;
use solvers::MacroSolver;

pub fn execute(
    item_id: u32,
    craftsmanship: u16,
    control: u16,
    cp: u16,
    level: u8,
    manipulation: bool,
    heart_and_soul: bool,
    quick_innovation: bool,
    adversarial: bool,
    backload_progress: bool,
    unsound: bool,
) {
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
