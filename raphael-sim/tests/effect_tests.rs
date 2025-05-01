use raphael_sim::*;

const SETTINGS: Settings = Settings {
    max_cp: 250,
    max_durability: 60,
    max_progress: 2000,
    max_quality: 40000,
    base_progress: 100,
    base_quality: 100,
    job_level: 100,
    allowed_actions: ActionMask::all(),
    adversarial: false,
    backload_progress: false,
};

/// Returns the 4 primary stats of a state:
/// - Progress
/// - Quality
/// - Durability (used)
/// - CP (used)
fn primary_stats(state: &SimulationState, settings: &Settings) -> (u32, u32, u16, u16) {
    (
        state.progress,
        state.quality,
        SETTINGS.max_durability - state.durability,
        settings.max_cp - state.cp,
    )
}

#[test]
fn test_trained_perfection() {
    let initial_state = SimulationState {
        effects: Effects::new().with_trained_perfection_active(true),
        ..SimulationState::new(&SETTINGS)
    };
    // No durability cost when trained perfection is active
    let state = initial_state
        .use_action(Action::BasicSynthesis, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (120, 0, 0, 0));
    assert_eq!(state.effects.trained_perfection_active(), false);
    // Trained Perfection effect doesn't wear off if durability cost is zero
    let state = initial_state
        .use_action(Action::Observe, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(state.effects.trained_perfection_active(), true);
}
