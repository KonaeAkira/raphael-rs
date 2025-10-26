use raphael_sim::{Action, ActionMask, Settings, SimulationState};
use raphael_solver::{AtomicFlag, MacroSolver, SolverSettings};

fn main() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let simulator_settings = Settings {
        max_cp: 59,
        max_durability: 35,
        max_progress: 1100,
        max_quality: 100,
        base_progress: 100,
        base_quality: 100,
        job_level: 99,
        allowed_actions: ActionMask::regular().add(Action::HeartAndSoul),
        adversarial: false,
        backload_progress: false,
    };

    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: false,
    };

    let mut solver = MacroSolver::new(
        solver_settings,
        Box::new(|_| {}),
        Box::new(|_| {}),
        AtomicFlag::new(),
    );
    let initial_state = SimulationState::from_macro(
        &simulator_settings,
        &[
            Action::MuscleMemory,
            Action::Observe,
            Action::Observe,
            Action::Observe,
            Action::Observe,
        ],
    )
    .unwrap();
    // enough for FinalAppraisal + H&S + IntensiveSynth + BasicTouch + BasicSynth
    assert_eq!(initial_state.durability, 25);
    assert_eq!(initial_state.cp, 1 + 6 + 18);
    let actions = solver.solve_with_initial_state(initial_state).unwrap();

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

    log::info!("{:?}", &actions);
}
