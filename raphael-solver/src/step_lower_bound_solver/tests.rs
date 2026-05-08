use bump_scope::BumpPool;
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
    let allocator = BumpPool::default();
    let mut solver = StepLbSolver::new(solver_settings, AtomicFlag::default(), &allocator);
    for state in generate_random_states(solver_settings, 1_000_000)
        .filter(|state| state.effects.combo() == Combo::None)
    {
        let state_step_lb = solver.step_lower_bound(state, 0).unwrap();
        for action in FULL_SEARCH_ACTIONS {
            if let Ok(child_state) = use_action_combo(&solver_settings, state, action) {
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

/// Regression test: step_lower_bound must return u8::MAX (not panic) when max_quality is
/// unreachable. Example is a Level 5 Leatherworker trying to craft Fingerless Leather Gloves
///
/// At job level 5, only BasicSynthesis (lvl 1) and BasicTouch (lvl 5) are available.
/// With 60 max durability and 10 durability per action, the crafter can take 6 actions
/// total. Reaching max_progress=21 requires 3 BasicSynthesis (27 progress), leaving only
/// 3 quality actions that can generate at most 124 quality — never reaching max_quality=130.
#[test]
fn step_lower_bound_unreachable_quality_no_panic() {
    // Recipe: Fingerless Leather Gloves (recipe_id=309, item_id=3530)
    // Crafter: level 5 Leatherworker, craftsmanship=39, control=9, cp=180
    //
    // max_progress, max_quality, max_durability:
    //   rlvls.rs index 4: { max_progress: 21, max_quality: 130, max_durability: 60, ... }
    //   multiplied by recipe progress_factor/quality_factor/durability_factor (all 100)
    //
    // base_progress = craftsmanship*10 / progress_div + 2 = 39*10/50 + 2 = 9.8 → 9
    //   (progress_div=50 from rlvls.rs; crafter level 5 > recipe job_level 4, no modifier penalty)
    //
    // base_quality = control*10 / quality_div + 35 = 9*10/30 + 35 = 38.0 → 38
    //   (quality_div=30 from rlvls.rs; same level rule as above)
    //
    // allowed_actions: ActionMask::all() minus the four actions get_game_settings removes:
    //   Manipulation  — crafter_stats.manipulation = false (level 65 skill, not unlocked)
    //   TrainedEye    — crafter level (5) < recipe job_level (4) + 10 = 14
    //   HeartAndSoul  — crafter_stats.heart_and_soul = false
    //   QuickInnovation — crafter_stats.quick_innovation = false
    let simulator_settings = Settings {
        max_progress: 21,
        max_quality: 130,
        max_durability: 60,
        max_cp: 180,
        base_progress: 9,
        base_quality: 38,
        job_level: 5,
        allowed_actions: ActionMask::all()
            .remove(Action::Manipulation)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
        stellar_steady_hand_charges: 0,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: false,
    };
    let allocator = BumpPool::default();
    let mut solver = StepLbSolver::new(solver_settings, AtomicFlag::default(), &allocator);
    let initial_state = SimulationState::new(&simulator_settings);
    let result = solver.step_lower_bound(initial_state, 0).unwrap();
    assert_eq!(result, u8::MAX);
}

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
        stellar_steady_hand_charges: 1,
    };
    let solver_settings = SolverSettings {
        simulator_settings,
        allow_non_max_quality_solutions: true,
    };
    check_consistency(solver_settings);
}
