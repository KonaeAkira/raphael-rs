use raphael_sim::*;
use test_case::test_matrix;

use crate::{
    AtomicFlag, SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
};

use super::*;

fn random_effects(settings: &Settings, rng: &mut impl rand::Rng) -> Effects {
    let mut effects = Effects::new()
        .with_inner_quiet(rng.random_range(0..=10))
        .with_great_strides(rng.random_range(0..=3))
        .with_innovation(rng.random_range(0..=4))
        .with_veneration(rng.random_range(0..=4))
        .with_waste_not(rng.random_range(0..=8))
        .with_adversarial_guard(rng.random() && settings.adversarial)
        .with_allow_quality_actions(rng.random() || !settings.backload_progress);
    if settings.is_action_allowed::<Manipulation>() {
        effects.set_manipulation(rng.random_range(0..=8));
    }
    if settings.is_action_allowed::<TrainedPerfection>() {
        effects.set_trained_perfection_available(rng.random());
        effects
            .set_trained_perfection_active(!effects.trained_perfection_available() && rng.random());
    }
    if settings.is_action_allowed::<HeartAndSoul>() {
        effects.set_heart_and_soul_available(rng.random());
        effects.set_heart_and_soul_active(!effects.heart_and_soul_available() && rng.random());
    }
    if settings.is_action_allowed::<QuickInnovation>() {
        effects.set_quick_innovation_available(rng.random());
    }
    effects
}

fn random_state(settings: &SolverSettings, rng: &mut impl rand::Rng) -> SimulationState {
    SimulationState {
        cp: rng.random_range(0..=settings.max_cp()),
        durability: rng
            .random_range(1..=settings.max_durability())
            .next_multiple_of(5),
        progress: rng.random_range(0..settings.max_progress()),
        quality: rng.random_range(0..=settings.max_quality()),
        unreliable_quality: 0,
        effects: random_effects(&settings.simulator_settings, rng),
    }
    .try_into()
    .unwrap()
}

/// Test that the StepLbSolver is consistent.
/// It is consistent if the step-lb of a parent state is never greater than the step-lb of a child state.
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

#[test_matrix(
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
