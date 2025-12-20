use raphael_data::{CrafterStats, RECIPES, get_game_settings};
use raphael_sim::{Action, Condition, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Scenario: Grade 2 Gemsap of Dexterity with fixed stats
    let recipe_id = 35914u32;
    let recipe = RECIPES
        .get(recipe_id)
        .ok_or_else(|| format!("Recipe {recipe_id} not found"))?;
    let crafter_stats = CrafterStats {
        level: 100,
        craftsmanship: 5000,
        control: 5000,
        cp: 600,
        manipulation: true,
        heart_and_soul: false,
        quick_innovation: false,
    };

    let game_settings = get_game_settings(recipe.to_owned(), None, crafter_stats, None, None);

    // Baseline rotation solved previously for this scenario (all Normal conditions)
    let base_actions: Vec<Action> = vec![
        Action::Reflect,
        Action::Innovation,
        Action::PrudentTouch,
        Action::PrudentTouch,
        Action::PrudentTouch,
        Action::PrudentTouch,
        Action::TrainedPerfection,
        Action::MasterMend,
        Action::Veneration,
        Action::Groundwork,
        Action::Innovation,
        Action::DelicateSynthesis,
        Action::DelicateSynthesis,
        Action::Observe,
        Action::AdvancedTouch,
        Action::MasterMend,
        Action::Veneration,
        Action::Innovation,
        Action::DelicateSynthesis,
        Action::DelicateSynthesis,
        Action::GreatStrides,
        Action::ByregotsBlessing,
        Action::BasicSynthesis,
    ];
    println!("Baseline rotation ({} steps):", base_actions.len());
    for action in &base_actions {
        println!("{action:?}");
    }

    // Simulate baseline (all Normal)
    let (baseline_state, _) =
        SimulationState::from_macro_continue_on_error(&game_settings, &base_actions);

    // Simulate with an Excellent on step 15 (index 14) while keeping the same actions
    let mut branch_state = SimulationState::new(&game_settings);
    for (idx, action) in base_actions.iter().enumerate() {
        let condition = if idx == 14 {
            Condition::Excellent
        } else {
            Condition::Normal
        };
        branch_state = branch_state
            .use_action(*action, condition, &game_settings)
            .map_err(|e| format!("Simulation error at step {}: {e}", idx + 1))?;
        if branch_state.is_final(&game_settings) {
            break;
        }
    }

    // Re-optimize tail: lock in steps 1-14 (Reflect through Observe), force Excellent on step 15, and let the solver find the best continuation.
    let prefix_len = 14;
    let prefix: Vec<_> = base_actions.iter().cloned().take(prefix_len).collect();
    let prefix_state = SimulationState::from_macro(&game_settings, &prefix)
        .map_err(|e| format!("Failed to simulate prefix: {e}"))?;
    let solver_settings = SolverSettings {
        simulator_settings: game_settings.clone(),
        allow_non_max_quality_solutions: true,
    };
    let mut macro_solver = MacroSolver::new(
        solver_settings,
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    );
    let optimized_tail = macro_solver
        .solve_from(prefix_state, Condition::Excellent)
        .map_err(|e| format!("Solve failed from prefix: {e:?}"))?;
    let combined: Vec<_> = prefix
        .iter()
        .cloned()
        .chain(optimized_tail.iter().cloned())
        .collect();
    let combined_state = SimulationState::from_macro(&game_settings, &combined)
        .map_err(|e| format!("Combined simulation failed: {e}"))?;

    println!("\nBaseline final state: {baseline_state:?}");
    println!("Branch final state (Excellent on step 15, same actions): {branch_state:?}");
    println!("\nSolver-optimized tail from step 15 = Excellent:");
    for (idx, action) in optimized_tail.iter().enumerate() {
        println!("{:>2}: {:?}", prefix_len + idx + 1, action);
    }
    println!(
        "Combined length: {} steps (<=19 required)\nFinal combined state: {:?}",
        combined.len(),
        combined_state
    );

    Ok(())
}
