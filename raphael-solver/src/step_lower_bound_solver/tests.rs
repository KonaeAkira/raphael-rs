use rand::Rng;
use raphael_sim::*;

use crate::{
    SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
};

use super::*;

fn solve(simulator_settings: Settings, actions: &[Action]) -> u8 {
    let mut state = SimulationState::from_macro(&simulator_settings, actions).unwrap();
    state.effects.set_combo(Combo::None);
    let solver_settings = SolverSettings { simulator_settings };
    StepLbSolver::new(solver_settings, Default::default())
        .step_lower_bound(state, 0)
        .unwrap()
}

#[test]
fn test_01() {
    let settings = Settings {
        max_cp: 553,
        max_durability: 70,
        max_progress: 2400,
        max_quality: 1700,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(
        settings,
        &[
            Action::MuscleMemory,
            Action::PrudentTouch,
            Action::Manipulation,
            Action::Veneration,
            Action::WasteNot2,
            Action::Groundwork,
            Action::Groundwork,
            Action::Groundwork,
            Action::PreparatoryTouch,
        ],
    );
    assert_eq!(result, 5);
}

#[test]
fn test_adversarial_01() {
    let settings = Settings {
        max_cp: 553,
        max_durability: 70,
        max_progress: 2400,
        max_quality: 1700,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let result = solve(
        settings,
        &[
            Action::MuscleMemory,
            Action::PrudentTouch,
            Action::Manipulation,
            Action::Veneration,
            Action::WasteNot2,
            Action::Groundwork,
            Action::Groundwork,
            Action::Groundwork,
            Action::PreparatoryTouch,
        ],
    );
    assert_eq!(result, 6);
}

#[test]
fn test_02() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 2500,
        max_quality: 5000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(
        settings,
        &[
            Action::MuscleMemory,
            Action::Manipulation,
            Action::Veneration,
            Action::WasteNot,
            Action::Groundwork,
            Action::Groundwork,
        ],
    );
    assert_eq!(result, 15);
}

#[test]
fn test_adversarial_02() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 2500,
        max_quality: 5000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let result = solve(
        settings,
        &[
            Action::MuscleMemory,
            Action::Manipulation,
            Action::Veneration,
            Action::WasteNot,
            Action::Groundwork,
            Action::Groundwork,
        ],
    );
    assert_eq!(result, 15);
}

#[test]
fn test_03() {
    let settings = Settings {
        max_cp: 617,
        max_durability: 60,
        max_progress: 2120,
        max_quality: 5000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(
        settings,
        &[
            Action::MuscleMemory,
            Action::Manipulation,
            Action::Veneration,
            Action::WasteNot,
            Action::Groundwork,
            Action::CarefulSynthesis,
            Action::Groundwork,
            Action::PreparatoryTouch,
            Action::Innovation,
            Action::BasicTouch,
            Action::StandardTouch,
        ],
    );
    assert_eq!(result, 13);
}

#[test]
fn test_adversarial_03() {
    let settings = Settings {
        max_cp: 617,
        max_durability: 60,
        max_progress: 2120,
        max_quality: 5000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let result = solve(
        settings,
        &[
            Action::MuscleMemory,
            Action::Manipulation,
            Action::Veneration,
            Action::WasteNot,
            Action::Groundwork,
            Action::CarefulSynthesis,
            Action::Groundwork,
            Action::PreparatoryTouch,
            Action::Innovation,
            Action::BasicTouch,
            Action::StandardTouch,
        ],
    );
    assert_eq!(result, 13);
}

#[test]
fn test_04() {
    let settings = Settings {
        max_cp: 411,
        max_durability: 60,
        max_progress: 1990,
        max_quality: 5000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::MuscleMemory]);
    assert_eq!(result, 19);
}

#[test]
fn test_adversarial_04() {
    let settings = Settings {
        max_cp: 411,
        max_durability: 60,
        max_progress: 1990,
        max_quality: 2900,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::MuscleMemory]);
    assert_eq!(result, 14);
}

#[test]
fn test_05() {
    let settings = Settings {
        max_cp: 450,
        max_durability: 60,
        max_progress: 1970,
        max_quality: 2000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::MuscleMemory]);
    assert_eq!(result, 12);
}

#[test]
fn test_adversarial_05() {
    let settings = Settings {
        max_cp: 450,
        max_durability: 60,
        max_progress: 1970,
        max_quality: 2000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::MuscleMemory]);
    assert_eq!(result, 12);
}

#[test]
fn test_06() {
    let settings = Settings {
        max_cp: 673,
        max_durability: 60,
        max_progress: 2345,
        max_quality: 3500,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::MuscleMemory]);
    assert_eq!(result, 16);
}

#[test]
fn test_adversarial_06() {
    let settings = Settings {
        max_cp: 673,
        max_durability: 60,
        max_progress: 2345,
        max_quality: 1200,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::MuscleMemory]);
    assert_eq!(result, 11);
}

#[test]
fn test_07() {
    let settings = Settings {
        max_cp: 673,
        max_durability: 60,
        max_progress: 2345,
        max_quality: 3123,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::Reflect]);
    assert_eq!(result, 15);
}

#[test]
fn test_08() {
    let settings = Settings {
        max_cp: 32,
        max_durability: 10,
        max_progress: 10000,
        max_quality: 20000,
        base_progress: 10000,
        base_quality: 10000,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[Action::PrudentTouch]);
    assert_eq!(result, 1);
}

#[test]
fn test_09() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 2500,
        max_quality: 3000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::Manipulation)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[]);
    assert_eq!(result, 17);
}

#[test]
fn test_10() {
    let settings = Settings {
        max_cp: 400,
        max_durability: 80,
        max_progress: 1200,
        max_quality: 2400,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::Manipulation)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[]);
    assert_eq!(result, 11);
}

#[test]
fn test_11() {
    let settings = Settings {
        max_cp: 320,
        max_durability: 80,
        max_progress: 1600,
        max_quality: 2000,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::Manipulation)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[]);
    assert_eq!(result, 11);
}

#[test]
fn test_12() {
    let settings = Settings {
        max_cp: 320,
        max_durability: 80,
        max_progress: 1600,
        max_quality: 2100,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::Manipulation)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
        backload_progress: false,
    };
    let result = solve(settings, &[]);
    assert_eq!(result, 11);
}

fn random_effects(settings: &Settings) -> Effects {
    Effects::new()
        .with_inner_quiet(rand::thread_rng().gen_range(0..=10))
        .with_great_strides(rand::thread_rng().gen_range(0..=3))
        .with_innovation(rand::thread_rng().gen_range(0..=4))
        .with_veneration(rand::thread_rng().gen_range(0..=4))
        .with_waste_not(rand::thread_rng().gen_range(0..=8))
        .with_manipulation(rand::thread_rng().gen_range(0..=8))
        .with_quick_innovation_available(rand::random())
        .with_adversarial_guard(if settings.adversarial {
            rand::random()
        } else {
            false
        })
        .with_allow_quality_actions(if settings.backload_progress {
            rand::random()
        } else {
            true
        })
}

fn random_state(settings: &Settings) -> SimulationState {
    SimulationState {
        cp: rand::thread_rng().gen_range(0..=settings.max_cp),
        durability: rand::thread_rng().gen_range(1..=(settings.max_durability / 5)) * 5,
        progress: rand::thread_rng().gen_range(0..u32::from(settings.max_progress)),
        quality: 0,
        unreliable_quality: 0,
        effects: random_effects(settings),
    }
    .try_into()
    .unwrap()
}

/// Test that the upper-bound solver is monotonic,
/// i.e. the quality UB of a state is never less than the quality UB of any of its children.
fn monotonic_fuzz_check(simulator_settings: Settings) {
    let solver_settings = SolverSettings { simulator_settings };
    let mut solver = StepLbSolver::new(solver_settings, Default::default());
    solver.precompute();
    for _ in 0..10000 {
        let state = random_state(&simulator_settings);
        let state_lower_bound = solver.step_lower_bound(state, 0).unwrap();
        for action in FULL_SEARCH_ACTIONS {
            let child_lower_bound = match use_action_combo(&solver_settings, state, *action) {
                Ok(child) => match child.is_final(&simulator_settings) {
                    false => solver.step_lower_bound(child, 0).unwrap(),
                    true if child.progress >= u32::from(simulator_settings.max_progress)
                        && child.quality >= u32::from(simulator_settings.max_quality) =>
                    {
                        0
                    }
                    true => u8::MAX,
                },
                Err(_) => u8::MAX,
            };
            if state_lower_bound > child_lower_bound.saturating_add(action.steps()) {
                dbg!(state, action, state_lower_bound, child_lower_bound);
                panic!("Parent's step lower bound is greater than child's step lower bound");
            }
        }
    }
}

#[test]
fn test_monotonic_normal_sim() {
    let settings = Settings {
        max_cp: 360,
        max_durability: 70,
        max_progress: 1000,
        max_quality: 2600,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::all(),
        adversarial: false,
        backload_progress: false,
    };
    monotonic_fuzz_check(settings);
}

#[test]
fn test_monotonic_backload_progress_sim() {
    let settings = Settings {
        max_cp: 360,
        max_durability: 70,
        max_progress: 1000,
        max_quality: 2600,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::all(),
        adversarial: false,
        backload_progress: true,
    };
    monotonic_fuzz_check(settings);
}

#[test]
fn test_monotonic_adversarial_sim() {
    let settings = Settings {
        max_cp: 360,
        max_durability: 70,
        max_progress: 1000,
        max_quality: 2400,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::all(),
        adversarial: true,
        backload_progress: false,
    };
    monotonic_fuzz_check(settings);
}
