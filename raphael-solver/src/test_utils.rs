use raphael_sim::*;

use crate::SolverSettings;

pub const REGULAR_ACTIONS: ActionMask = ActionMask::all()
    .remove(Action::TrainedEye)
    .remove(Action::HeartAndSoul)
    .remove(Action::QuickInnovation);
pub const NO_MANIPULATION: ActionMask = REGULAR_ACTIONS.remove(Action::Manipulation);
pub const WITH_SPECIALIST_ACTIONS: ActionMask = REGULAR_ACTIONS
    .add(Action::HeartAndSoul)
    .add(Action::QuickInnovation);

fn random_effects(settings: &Settings, rng: &mut impl rand::Rng) -> Effects {
    let mut effects = Effects::new()
        .with_muscle_memory(rng.random_range(0..=5))
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

pub fn random_state(settings: &SolverSettings, rng: &mut impl rand::Rng) -> SimulationState {
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
