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
    stellar_steady_hand_charges: 0,
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

#[test]
fn test_stellar_steady_hand() {
    let mut state = SimulationState {
        effects: Effects::new().with_stellar_steady_hand(3),
        ..SimulationState::new(&SETTINGS)
    };
    // Test that the duration of Stellar Steady Hand decreases after every action.
    while state.effects.stellar_steady_hand() > 0 {
        let new_state = state
            .use_action(Action::Observe, Condition::Normal, &SETTINGS)
            .unwrap();
        assert_eq!(
            new_state.effects.stellar_steady_hand(),
            state.effects.stellar_steady_hand() - 1
        );
        state = new_state;
    }
}

#[test]
fn test_expedience() {
    let initial_state = SimulationState {
        effects: Effects::new()
            .with_expedience(true)
            .with_heart_and_soul_available(true),
        ..SimulationState::new(&SETTINGS)
    };
    // Expedience goes away after using an action.
    let state = initial_state
        .use_action(Action::BasicSynthesis, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(state.effects.expedience(), false);
    // Expedience effect doesn't wear off when using an action that doesn't tick effects
    // such as Heart and Soul.
    let state = initial_state
        .use_action(Action::HeartAndSoul, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(state.effects.expedience(), true);
}
