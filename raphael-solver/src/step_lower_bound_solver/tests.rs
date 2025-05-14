use rand::Rng;
use raphael_sim::*;
use test_case::test_matrix;

use crate::{
    AtomicFlag, SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
};

use super::*;

fn random_effects(settings: &Settings) -> Effects {
    let mut rng = rand::thread_rng();
    let mut effects = Effects::new()
        .with_inner_quiet(rng.gen_range(0..=10))
        .with_great_strides(rng.gen_range(0..=3))
        .with_innovation(rng.gen_range(0..=4))
        .with_veneration(rng.gen_range(0..=4))
        .with_waste_not(rng.gen_range(0..=8))
        .with_manipulation(rng.gen_range(0..=8))
        .with_adversarial_guard(rng.r#gen() && settings.adversarial)
        .with_allow_quality_actions(rng.r#gen() || !settings.backload_progress);
    if settings.is_action_allowed::<Manipulation>() {
        effects.set_manipulation(rng.gen_range(0..=8));
    }
    if settings.is_action_allowed::<TrainedPerfection>() {
        effects.set_trained_perfection_available(rng.r#gen());
        effects
            .set_trained_perfection_active(!effects.trained_perfection_available() && rng.r#gen());
    }
    if settings.is_action_allowed::<HeartAndSoul>() {
        effects.set_heart_and_soul_available(rng.r#gen());
        effects.set_heart_and_soul_active(!effects.heart_and_soul_available() && rng.r#gen());
    }
    if settings.is_action_allowed::<QuickInnovation>() {
        effects.set_quick_innovation_available(rng.r#gen());
    }
    effects
}

fn random_state(settings: &SolverSettings) -> SimulationState {
    SimulationState {
        cp: rand::thread_rng().gen_range(0..=settings.max_cp()),
        durability: rand::thread_rng()
            .gen_range(1..=settings.max_durability())
            .next_multiple_of(5),
        progress: rand::thread_rng().gen_range(0..settings.max_progress()),
        quality: rand::thread_rng().gen_range(0..=settings.max_quality()),
        unreliable_quality: 0,
        effects: random_effects(&settings.simulator_settings),
    }
    .try_into()
    .unwrap()
}

/// Test that the StepLbSolver is consistent.
/// It is consistent if the step-lb of a parent state is never greater than the step-lb of a child state.
fn check_consistency(solver_settings: SolverSettings) {
    let mut solver = StepLbSolver::new(solver_settings, AtomicFlag::default());
    for _ in 0..100000 {
        let state = random_state(&solver_settings);
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

#[test_matrix(
    [1000, 2000],
    [1000, 2000, 4000],
    [20, 35, 60, 80],
    [90, 100]
)]
fn consistency(max_progress: u16, max_quality: u16, max_durability: u16, job_level: u8) {
    let simulator_settings = Settings {
        max_progress,
        max_quality,
        max_durability,
        max_cp: 1000, // max_cp does not matter for StepLbSolver
        base_progress: 100,
        base_quality: 100,
        job_level,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let solver_settings = SolverSettings { simulator_settings };
    check_consistency(solver_settings);
}
