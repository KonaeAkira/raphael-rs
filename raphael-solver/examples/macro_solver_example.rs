use raphael_sim::{Action, ActionMask, Settings, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    // Ra'Kaznar Lapidary Hammer
    // 4462 Craftsmanship, 4391 Control
    let simulator_settings = Settings {
        max_cp: 569,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 10000,
        base_progress: 237,
        base_quality: 245,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };

    let solver_settings = SolverSettings {
        simulator_settings,
        backload_progress: false,
        allow_unsound_branch_pruning: false,
    };

    let mut solver = MacroSolver::new(
        solver_settings,
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    );
    let actions = solver.solve().unwrap();

    let quality = SimulationState::from_macro(&simulator_settings, &actions)
        .unwrap()
        .quality;
    let steps = actions.len();
    let duration: u8 = actions.iter().map(|action| action.time_cost()).sum();

    log::info!(
        "Solution - quality: {}, steps: {}, duration: {}",
        quality,
        steps,
        duration
    );
}
