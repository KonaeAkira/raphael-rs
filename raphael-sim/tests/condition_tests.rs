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
fn test_good_omen() {
    let initial_state = SimulationState {
        effects: Effects::new().with_condition(Condition::GoodOmen),
        ..SimulationState::new(&SETTINGS)
    };

    let state = initial_state
        .use_action(Action::BasicSynthesis, &SETTINGS)
        .unwrap();

    assert_eq!(primary_stats(&state, &SETTINGS), (120, 0, 10, 0));
    assert_eq!(state.effects.condition(), Condition::Good);
}

#[test]
fn test_malleable() {
    let initial_state = SimulationState {
        effects: Effects::new().with_condition(Condition::Malleable),
        ..SimulationState::new(&SETTINGS)
    };

    let state = initial_state
        .use_action(Action::BasicSynthesis, &SETTINGS)
        .unwrap();

    assert_eq!(primary_stats(&state, &SETTINGS), (180, 0, 10, 0))
}

#[test]
fn test_pliant() {
    let initial_state = SimulationState {
        effects: Effects::new().with_condition(Condition::Pliant),
        ..SimulationState::new(&SETTINGS)
    };

    let state = initial_state
        .use_action(Action::Manipulation, &SETTINGS)
        .unwrap();

    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 48))
}

#[test]
fn test_primed() {
    let initial_state = SimulationState {
        effects: Effects::new().with_condition(Condition::Primed),
        ..SimulationState::new(&SETTINGS)
    };

    let state = initial_state
        .use_action(Action::Innovation, &SETTINGS)
        .unwrap();

    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 18));
    assert_eq!(state.effects.innovation(), 6);
}

#[test]
fn test_sturdy() {
    let initial_state = SimulationState {
        effects: Effects::new().with_condition(Condition::Sturdy),
        ..SimulationState::new(&SETTINGS)
    };

    let state = initial_state
        .use_action(Action::BasicSynthesis, &SETTINGS)
        .unwrap();

    assert_eq!(primary_stats(&state, &SETTINGS), (120, 0, 5, 0));
}
