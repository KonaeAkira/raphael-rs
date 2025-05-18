use raphael_sim::*;

use crate::{
    AtomicFlag, SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
    test_utils::*,
};

use super::*;

/// Test that the StepLbSolver is consistent and admissible.
/// It is consistent if the step-lb of a parent state is never greater than the step-lb of a child state.
/// It is admissible if the step-lb of a state is never greater than the step count of a reachable final state.
fn check_consistency(solver_settings: SolverSettings) {
    let mut solver = StepLbSolver::new(solver_settings, AtomicFlag::default());
    let mut rng = rand::rng();
    for _ in 0..100000 {
        let state = random_state(&solver_settings, &mut rng);
        let state_step_lb = solver.step_lower_bound(state, 0).unwrap();
        for action in FULL_SEARCH_ACTIONS {
            if let Ok(child_state) = use_action_combo(&solver_settings, state, *action) {
                let child_step_lb = if child_state.is_final(&solver_settings.simulator_settings) {
                    let progress_maxed = child_state.progress >= solver_settings.max_progress();
                    let quality_maxed = child_state.quality >= solver_settings.max_quality();
                    if progress_maxed && quality_maxed {
                        0
                    } else {
                        u8::MAX
                    }
                } else {
                    solver.step_lower_bound(child_state, 0).unwrap()
                };
                if state_step_lb > child_step_lb.saturating_add(action.steps()) {
                    dbg!(state, action, state_step_lb, child_step_lb);
                    panic!("StepLbSolver is not consistent");
                }
            };
        }
    }
}

const REGULAR_ACTIONS: ActionMask = ActionMask::all()
    .remove(Action::TrainedEye)
    .remove(Action::HeartAndSoul)
    .remove(Action::QuickInnovation);
const NO_MANIPULATION: ActionMask = REGULAR_ACTIONS.remove(Action::Manipulation);
const WITH_SPECIALIST_ACTIONS: ActionMask = REGULAR_ACTIONS
    .add(Action::HeartAndSoul)
    .add(Action::QuickInnovation);

#[test_case::test_matrix(
    [20, 35, 60, 80],
    [REGULAR_ACTIONS, NO_MANIPULATION, WITH_SPECIALIST_ACTIONS]
)]
fn consistency(max_durability: u16, allowed_actions: ActionMask) {
    let simulator_settings = Settings {
        max_progress: 2000,
        max_quality: 2000,
        max_durability,
        max_cp: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions,
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    check_consistency(solver_settings);
}
