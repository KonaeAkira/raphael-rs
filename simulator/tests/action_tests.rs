use simulator::{Action, ActionMask, Settings, SimulationState};

const SETTINGS: Settings = Settings {
    max_cp: 200,
    max_durability: 60,
    max_progress: 2000,
    max_quality: 40000,
    base_progress: 100,
    base_quality: 100,
    initial_quality: 0,
    job_level: 90,
    allowed_actions: ActionMask::none(),
};

#[test]
fn test_standard_touch_combo() {
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::ComboStandardTouch]);
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::ComboStandardTouch]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_advanced_touch_combo() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::BasicTouch,
            Action::ComboStandardTouch,
            Action::ComboAdvancedTouch,
        ],
    );
    assert!(matches!(state, Ok(_)));
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::Observe, Action::ComboAdvancedTouch]);
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::ComboAdvancedTouch]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_reflect_opener() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::Reflect]);
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::Reflect]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_muscle_memory_opener() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::MuscleMemory]);
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::MuscleMemory]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_manipulation() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::BasicSynthesis,
            Action::Manipulation,
            Action::Manipulation,
        ],
    )
    .unwrap();
    assert_eq!(state.durability, SETTINGS.max_durability - 10);
}

#[test]
fn test_prudent_touch() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::WasteNot, Action::PrudentTouch]);
    assert!(matches!(
        state,
        Err("Action cannot be used during Waste Not")
    ));
}

#[test]
fn test_groundwork() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::PreparatoryTouch,
            Action::PreparatoryTouch,
            Action::BasicSynthesis,
            Action::Groundwork,
        ],
    );
    assert!(matches!(state, Err("Not enough durability")));
}

#[test]
fn test_prudent_synthesis() {
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::WasteNot, Action::PrudentSynthesis]);
    assert!(matches!(
        state,
        Err("Action cannot be used during Waste Not")
    ));
}

#[test]
fn test_trained_finesse() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::PreparatoryTouch,
            Action::PreparatoryTouch,
            Action::TrainedFinesse,
        ],
    );
    assert!(matches!(state, Err("Requires 10 Inner Quiet")));
}

#[test]
fn test_refined_touch() {
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::ComboRefinedTouch]);
    match state {
        Ok(state) => {
            assert_eq!(state.effects.inner_quiet, 3);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::ComboRefinedTouch]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_immaculate_mend() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::BasicTouch,
            Action::Groundwork,
            Action::Groundwork,
            Action::ImmaculateMend,
        ],
    );
    match state {
        Ok(state) => {
            assert_eq!(state.durability, SETTINGS.max_durability);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_trained_perfection() {
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::TrainedPerfection, Action::Groundwork]);
    match state {
        Ok(state) => {
            assert_eq!(state.durability, SETTINGS.max_durability);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    };
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[Action::TrainedPerfection, Action::TrainedPerfection],
    );
    assert!(matches!(
        state,
        Err("Action can only be used once per synthesis")
    ));
}
