use raphael_sim::{ActionMask, Settings, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let simulator_settings = Settings {
        max_cp: 753,
        max_durability: 20,
        max_progress: 4700,
        max_quality: 14900,
        base_progress: 310,
        base_quality: 324,
        job_level: 100,
        allowed_actions: ActionMask::regular(),
        adversarial: false,
        backload_progress: false,
    };

    let solver_settings = SolverSettings { simulator_settings };

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
