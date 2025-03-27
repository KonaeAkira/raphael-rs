use raphael_sim::{Action, ActionMask, Settings, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

fn main() {
    #[cfg(feature = "env_logger")]
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    log::trace!(
        "SimulationState - size: {}, align: {}",
        std::mem::size_of::<SimulationState>(),
        std::mem::align_of::<SimulationState>()
    );

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
