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
fn test_random_926ae85b() {
    // Copper Gorget
    // 10 Craftsmanship, 10 Control
    let settings = Settings {
        max_cp: 50,
        max_durability: 60,
        max_progress: 33,
        max_quality: 150,
        base_progress: 4,
        base_quality: 38,
        job_level: 10,
        allowed_actions: ActionMask::none(),
    };
    let state = from_action_sequence(
        &settings,
        &[
            Action::BasicSynthesis,
            Action::BasicTouch,
            Action::BasicTouch,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 14);
            assert_eq!(state.durability, 30);
            assert_eq!(settings.max_progress - state.missing_progress, 4);
            assert_eq!(settings.max_quality - state.missing_quality, 76);
            assert_eq!(state.effects.inner_quiet, 0);
        }
        _ => panic!(),
    }
}

#[test]
fn test_random_3c721e47() {
    // Ironwood Spear
    // 3000 Craftsmanship, 3000 Control
    let settings = Settings {
        max_cp: 500,
        max_durability: 80,
        max_progress: 3100,
        max_quality: 6800,
        base_progress: 240,
        base_quality: 307,
        job_level: 85,
        allowed_actions: ActionMask::none(),
    };
    let state = from_action_sequence(
        &settings,
        &[
            Action::MuscleMemory,
            Action::Veneration,
            Action::WasteNot,
            Action::Groundwork,
            Action::Manipulation,
            Action::Innovation,
            Action::PreparatoryTouch,
            Action::PrudentTouch,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 223);
            assert_eq!(state.durability, 60);
            assert_eq!(settings.max_progress - state.missing_progress, 2520);
            assert_eq!(settings.max_quality - state.missing_quality, 1473);
        }
        _ => panic!(),
    }
}

#[test]
fn test_random_3ba90d3a() {
    // Grade 4 Skybuilders' Stone
    // 1826 Craftsmanship, 1532 Control
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
            Action::PreparatoryTouch,
            Action::MasterMend,
            Action::Innovation,
            Action::PrudentTouch,
            Action::BasicTouch,
            Action::ComboStandardTouch,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 188);
            assert_eq!(state.durability, 25);
            assert_eq!(settings.max_progress - state.missing_progress, 918);
            assert_eq!(settings.max_quality - state.missing_quality, 2118);
            assert_eq!(state.effects.inner_quiet, 5);
            assert_eq!(state.effects.innovation, 1);
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
