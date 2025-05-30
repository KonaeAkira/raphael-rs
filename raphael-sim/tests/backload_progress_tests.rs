use raphael_sim::{Action, ActionMask, Condition, Settings, SimulationState};

const SETTINGS: Settings = Settings {
    max_cp: 500,
    max_durability: 80,
    max_progress: 2000,
    max_quality: 2000,
    base_progress: 100,
    base_quality: 100,
    job_level: 100,
    allowed_actions: ActionMask::all(),
    adversarial: true,
    backload_progress: true,
};

#[test]
/// Test that effects are updated correctly under `backload_progress` after using a Progress-increasing action:
/// - Effects that only influence Quality should be removed.
/// - The `allow_quality_actions` effect must be `false`.
fn test_effects() {
    let mut state = SimulationState::new(&SETTINGS);
    state.unreliable_quality = 100;
    state.effects.set_inner_quiet(2);
    state.effects.set_innovation(2);
    state.effects.set_great_strides(2);
    state.effects.set_adversarial_guard(true);
    state.effects.set_quick_innovation_available(true);
    let state = state
        .use_action(Action::MuscleMemory, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(state.unreliable_quality, 0);
    assert_eq!(state.effects.inner_quiet(), 0);
    assert_eq!(state.effects.innovation(), 0);
    assert_eq!(state.effects.great_strides(), 0);
    assert_eq!(state.effects.adversarial_guard(), false);
    assert_eq!(state.effects.quick_innovation_available(), false);
    assert_eq!(state.effects.allow_quality_actions(), false);
}

#[test]
/// Test that Quality-increasing actions are forbidden under `backload_progress` after using a Progress-increasing action.
fn test_quality_actions_forbidden() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[Action::Reflect, Action::BasicSynthesis, Action::Observe],
    )
    .unwrap();

    let result = state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS);
    assert_eq!(result, Err("Forbidden by backload_progress setting"));

    let result = state.use_action(Action::PreciseTouch, Condition::Good, &SETTINGS);
    assert_eq!(result, Err("Forbidden by backload_progress setting"));

    let result = state.use_action(Action::Innovation, Condition::Normal, &SETTINGS);
    assert_eq!(result, Err("Forbidden by backload_progress setting"));

    let result = state.use_action(Action::GreatStrides, Condition::Normal, &SETTINGS);
    assert_eq!(result, Err("Forbidden by backload_progress setting"));

    // Using a Progress-action resets Inner Quiet.
    let result = state.use_action(Action::ByregotsBlessing, Condition::Normal, &SETTINGS);
    assert_eq!(
        result,
        Err("Cannot use Byregot's Blessing when Inner Quiet is 0.")
    );
}

#[test]
/// Test that Delicate Synthesis can be used under `backload_progress`.
/// It is the only action that increases Progress and Quality at the same time.
fn test_delicate_synthesis() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::DelicateSynthesis]).unwrap();
    assert_eq!(state.effects.allow_quality_actions(), false);
}
