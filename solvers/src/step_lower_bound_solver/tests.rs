use rand::Rng;
use simulator::*;

use crate::actions::{use_action_combo, FULL_SEARCH_ACTIONS};

use super::*;

fn solve(settings: Settings, actions: &[Action]) -> u8 {
    let state = SimulationState {
        combo: Combo::None,
        ..SimulationState::from_macro(&settings, actions).unwrap()
    };
    StepLowerBoundSolver::new(settings, false, false, Default::default())
        .step_lower_bound_with_hint(state, 0)
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
    };
    let result = solve(settings, &[]);
    assert_eq!(result, 11);
}

fn random_effects(adversarial: bool) -> Effects {
    Effects::default()
        .with_inner_quiet(rand::thread_rng().gen_range(0..=10))
        .with_great_strides(rand::thread_rng().gen_range(0..=3))
        .with_innovation(rand::thread_rng().gen_range(0..=4))
        .with_veneration(rand::thread_rng().gen_range(0..=4))
        .with_waste_not(rand::thread_rng().gen_range(0..=8))
        .with_manipulation(rand::thread_rng().gen_range(0..=8))
        .with_quick_innovation_used(rand::random())
        .with_guard(if adversarial {
            rand::thread_rng().gen_range(0..=1)
        } else {
            0
        })
}

fn random_state(settings: &Settings) -> SimulationState {
    SimulationState {
        cp: rand::thread_rng().gen_range(0..=settings.max_cp),
        durability: rand::thread_rng().gen_range(1..=(settings.max_durability / 5)) * 5,
        progress: rand::thread_rng().gen_range(0..settings.max_progress),
        quality: 0,
        unreliable_quality: 0,
        effects: random_effects(settings.adversarial),
        combo: Combo::None,
    }
    .try_into()
    .unwrap()
}

/// Test that the upper-bound solver is monotonic,
/// i.e. the quality UB of a state is never less than the quality UB of any of its children.
fn monotonic_fuzz_check(settings: Settings) {
    let mut solver = StepLowerBoundSolver::new(settings, false, false, Default::default());
    for _ in 0..10000 {
        let state = random_state(&settings);
        let state_lower_bound = solver.step_lower_bound_with_hint(state, 0).unwrap();
        for action in FULL_SEARCH_ACTIONS {
            let child_lower_bound = match use_action_combo(&settings, state, *action) {
                Ok(child) => match child.is_final(&settings) {
                    false => solver.step_lower_bound_with_hint(child, 0).unwrap(),
                    true if child.progress >= settings.max_progress
                        && child.quality >= settings.max_quality =>
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
    };
    monotonic_fuzz_check(settings);
}
