use raphael::game::{Action, ActionMask, Condition, Settings, State};

fn from_action_sequence(settings: &Settings, actions: &[Action]) -> State {
    let mut state: State = State::new(&settings);
    for action in actions {
        state = state.as_in_progress().unwrap().use_action(
            action.clone(),
            Condition::Normal,
            &settings,
        );
    }
    return state;
}

#[test]
fn test_random_3ba90d3a() {
    let settings = Settings {
        max_cp: 427,
        max_durability: 60,
        max_progress: 1080,
        max_quality: 9900,
        base_progress: 204,
        base_quality: 253,
        job_level: 81,
        allowed_actions: ActionMask::none(),
    };
    let state = from_action_sequence(
        &settings,
        &[
            Action::Veneration,
            Action::CarefulSynthesis,
            Action::CarefulSynthesis,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 395);
            assert_eq!(state.durability, 40);
            assert_eq!(settings.max_progress - state.missing_progress, 918);
            assert_eq!(settings.max_quality - state.missing_quality, 0);
        }
        _ => panic!(),
    }
}

#[test]
fn test_random_cf2bca5c() {
    let settings = Settings {
        max_cp: 427,
        max_durability: 60,
        max_progress: 1080,
        max_quality: 9900,
        base_progress: 204,
        base_quality: 253,
        job_level: 81,
        allowed_actions: ActionMask::none(),
    };
    let state = from_action_sequence(
        &settings,
        &[
            Action::Reflect,
            Action::PreparatoryTouch,
            Action::Innovation,
            Action::PrudentTouch,
            Action::BasicTouch,
            Action::ComboStandardTouch,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 302);
            assert_eq!(state.durability, 5);
            assert_eq!(settings.max_progress - state.missing_progress, 0);
            assert_eq!(settings.max_quality - state.missing_quality, 2719);
        }
        _ => panic!(),
    }
}

#[test]
fn test_random_bce2650c() {
    // Diadochos Wristband of Healing
    // 4020 Craftsmanship, 4042 Control
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 6600,
        max_quality: 14040,
        base_progress: 248,
        base_quality: 270,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, false),
    };
    let state = from_action_sequence(
        &settings,
        &[
            Action::MuscleMemory,
            Action::WasteNot,
            Action::Veneration,
            Action::Groundwork,
            Action::Groundwork,
            Action::Groundwork,
            Action::PrudentSynthesis,
            Action::MasterMend,
            Action::PrudentTouch,
            Action::Innovation,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::MasterMend,
            Action::Innovation,
            Action::PrudentTouch,
            Action::BasicTouch,
            Action::ComboStandardTouch,
            Action::ComboAdvancedTouch,
            Action::GreatStrides,
            Action::Innovation,
            Action::Observe,
            Action::FocusedTouch,
            Action::GreatStrides,
            Action::ByregotsBlessing,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 1);
            assert_eq!(state.durability, 5);
            assert_eq!(settings.max_progress - state.missing_progress, 6323);
            assert_eq!(settings.max_quality - state.missing_quality, 11475);
        }
        _ => panic!(),
    }
}
