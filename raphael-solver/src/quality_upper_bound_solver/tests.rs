use raphael_sim::*;

use crate::{
    SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
    test_utils::*,
};

use super::QualityUbSolver;

fn solve(simulator_settings: Settings, actions: &[Action]) -> u32 {
    let mut state = SimulationState::from_macro(&simulator_settings, actions).unwrap();
    state.effects.set_combo(Combo::None);
    let solver_settings = SolverSettings { simulator_settings };
    let mut solver = QualityUbSolver::new(solver_settings, Default::default());
    solver.quality_upper_bound(state).unwrap()
}

#[test]
fn test_manipulation_refund() {
    // https://github.com/KonaeAkira/raphael-rs/pull/128#discussion_r2062585163
    let settings = Settings {
        max_cp: 500,
        max_durability: 80,
        max_progress: 700,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::Manipulation]);
    assert_eq!(result, 4975);
}

/// Test that the QualityUbSolver is consistent and admissible.
/// It is consistent if the step-lb of a parent state is never greater than the step-lb of a child state.
/// It is admissible if the quality-ub of a state is never less than the quality of a reachable final state.
fn check_consistency(solver_settings: SolverSettings) {
    let mut solver = QualityUbSolver::new(solver_settings, Default::default());
    solver.precompute();
    let mut rng = rand::rng();
    for _ in 0..100000 {
        let state = random_state(&solver_settings, &mut rng);
        let state_upper_bound = solver.quality_upper_bound(state).unwrap();
        for action in FULL_SEARCH_ACTIONS {
            let child_upper_bound = match use_action_combo(&solver_settings, state, *action) {
                Ok(child) => match child.is_final(&solver_settings.simulator_settings) {
                    false => solver.quality_upper_bound(child).unwrap(),
                    true if child.progress >= u32::from(solver_settings.max_progress()) => {
                        std::cmp::min(u32::from(solver_settings.max_quality()), child.quality)
                    }
                    true => 0,
                },
                Err(_) => 0,
            };
            if state_upper_bound < child_upper_bound {
                dbg!(state, action, state_upper_bound, child_upper_bound);
                panic!("Parent's upper bound is less than child's upper bound");
            }
        }
    }
}

#[test_case::test_matrix(
    [20, 60, 80],
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
