use simulator::{state::InProgress, Action, ActionMask, ComboAction, Condition, Settings, State};

const SETTINGS: Settings = Settings {
    max_cp: 200,
    max_durability: 60,
    max_progress: 2000,
    max_quality: 40000,
    base_progress: 100,
    base_quality: 100,
    job_level: 90,
    allowed_actions: ActionMask::none(),
};

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
fn test_basic_synthesis() {
    let state = from_action_sequence(&SETTINGS, &[Action::BasicSynthesis]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 200);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(120)
            );
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_basic_touch() {
    let state = from_action_sequence(&SETTINGS, &[Action::BasicTouch]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 182);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(100)
            );
            assert_eq!(state.effects.inner_quiet, 1);
            assert_eq!(state.combo, Some(ComboAction::BasicTouch));
        }
        _ => panic!(),
    }
}

#[test]
fn test_standard_touch() {
    let state = from_action_sequence(&SETTINGS, &[Action::BasicTouch, Action::ComboStandardTouch]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 164);
            assert_eq!(state.durability, 40);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(237)
            );
            assert_eq!(state.effects.inner_quiet, 2);
            assert_eq!(state.combo, Some(ComboAction::StandardTouch));
        }
        _ => panic!(),
    }
    // can't use without first using basic touch
    let state = from_action_sequence(&SETTINGS, &[Action::ComboStandardTouch]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_advanced_touch() {
    let state = from_action_sequence(
        &SETTINGS,
        &[
            Action::BasicTouch,
            Action::ComboStandardTouch,
            Action::ComboAdvancedTouch,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 146);
            assert_eq!(state.durability, 30);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(417)
            );
            assert_eq!(state.effects.inner_quiet, 3);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // can't use without first using basic touch
    let state = from_action_sequence(&SETTINGS, &[Action::ComboAdvancedTouch]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_master_mend() {
    let state = from_action_sequence(
        &SETTINGS,
        &[
            Action::BasicTouch,
            Action::BasicTouch,
            Action::BasicTouch,
            Action::BasicTouch,
            Action::MasterMend,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.durability, 50);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // can't restore durability above max durability
    let state = from_action_sequence(&SETTINGS, &[Action::BasicTouch, Action::MasterMend]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.durability, 60);
        }
        _ => panic!(),
    }
}

#[test]
fn test_observe() {
    let state = from_action_sequence(&SETTINGS, &[Action::Observe]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 193);
            assert_eq!(state.combo, Some(ComboAction::Observe));
        }
        _ => panic!(),
    }
}

#[test]
fn test_tricks_of_the_trade() {
    let mut state: InProgress = InProgress::new(&SETTINGS);
    state.cp -= 50;
    let state = state.use_action(Action::TricksOfTheTrade, Condition::Good, &SETTINGS);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 170);
        }
        _ => panic!(),
    }
    // CP after action can't exceed max cp
    let mut state: InProgress = InProgress::new(&SETTINGS);
    state.cp -= 10;
    let state = state.use_action(Action::TricksOfTheTrade, Condition::Excellent, &SETTINGS);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 200);
        }
        _ => panic!(),
    }
    // can't use action when condition isn't Good or Excellent
    let state = from_action_sequence(&SETTINGS, &[Action::TricksOfTheTrade]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_waste_not() {
    let state = from_action_sequence(&SETTINGS, &[Action::WasteNot]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 144);
            assert_eq!(state.effects.waste_not, 4);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_waste_not_2() {
    let state = from_action_sequence(&SETTINGS, &[Action::WasteNot2]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 102);
            assert_eq!(state.effects.waste_not, 8);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_veneration() {
    let state = from_action_sequence(&SETTINGS, &[Action::Veneration]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 182);
            assert_eq!(state.effects.veneration, 4);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_innovation() {
    let state = from_action_sequence(&SETTINGS, &[Action::Innovation]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 182);
            assert_eq!(state.effects.innovation, 4);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_great_strides() {
    let state = from_action_sequence(&SETTINGS, &[Action::GreatStrides]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 168);
            assert_eq!(state.effects.great_strides, 3);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_byregots_blessing() {
    let state = from_action_sequence(&SETTINGS, &[Action::BasicTouch, Action::ByregotsBlessing]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 158);
            assert_eq!(state.durability, 40);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(232)
            );
            assert_eq!(state.effects.inner_quiet, 0);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // cannot use Byregot's Blessing when inner quiet is zero
    let state = from_action_sequence(&SETTINGS, &[Action::ByregotsBlessing]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_precise_touch() {
    let state: InProgress = InProgress::new(&SETTINGS);
    let state = state.use_action(Action::PreciseTouch, Condition::Good, &SETTINGS);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 182);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(225)
            );
            assert_eq!(state.effects.inner_quiet, 2);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // can't use Precise Touch when condition isn't Good or Excellent
    let state = from_action_sequence(&SETTINGS, &[Action::PreciseTouch]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_muscle_memory() {
    let state = from_action_sequence(&SETTINGS, &[Action::MuscleMemory]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 194);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(300)
            );
            assert_eq!(state.effects.muscle_memory, 5);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // invalid use (not first action)
    let state = from_action_sequence(&SETTINGS, &[Action::BasicTouch, Action::MuscleMemory]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_careful_synthesis() {
    let state = from_action_sequence(&SETTINGS, &[Action::CarefulSynthesis]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 193);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(180)
            );
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_manipulation() {
    let state = from_action_sequence(&SETTINGS, &[Action::Manipulation]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 104);
            assert_eq!(state.effects.manipulation, 8);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_prudent_touch() {
    let state = from_action_sequence(&SETTINGS, &[Action::PrudentTouch]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 175);
            assert_eq!(state.durability, 55);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(100)
            );
            assert_eq!(state.effects.inner_quiet, 1);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // invalid use (can't use during waste not)
    let state = from_action_sequence(&SETTINGS, &[Action::WasteNot, Action::PrudentTouch]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_focused_synthesis() {
    let state = from_action_sequence(&SETTINGS, &[Action::Observe, Action::FocusedSynthesis]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 188);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(200)
            );
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // invalid use (need observe combo action)
    let state = from_action_sequence(&SETTINGS, &[Action::FocusedSynthesis]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_focused_touch() {
    let state = from_action_sequence(&SETTINGS, &[Action::Observe, Action::FocusedTouch]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 175);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(150)
            );
            assert_eq!(state.effects.inner_quiet, 1);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // invalid use (need observe combo action)
    let state = from_action_sequence(&SETTINGS, &[Action::FocusedTouch]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_reflect() {
    let state = from_action_sequence(&SETTINGS, &[Action::Reflect]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 194);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(100)
            );
            assert_eq!(state.effects.inner_quiet, 2);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // invalid use (not first action)
    let state = from_action_sequence(&SETTINGS, &[Action::BasicTouch, Action::Reflect]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_preparatory_touch() {
    let state = from_action_sequence(&SETTINGS, &[Action::PreparatoryTouch]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 160);
            assert_eq!(state.durability, 40);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(200)
            );
            assert_eq!(state.effects.inner_quiet, 2);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_groundwork() {
    let state = from_action_sequence(&SETTINGS, &[Action::Groundwork]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 182);
            assert_eq!(state.durability, 40);
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(360)
            );
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // can't use Groundwork when there isn't enough durability
    let mut state: InProgress = InProgress::new(&SETTINGS);
    state.durability = 10;
    let state = state.use_action(Action::Groundwork, Condition::Normal, &SETTINGS);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_delicate_synthesis() {
    let state = from_action_sequence(&SETTINGS, &[Action::DelicateSynthesis]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 168);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(100)
            );
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(100)
            );
            assert_eq!(state.effects.inner_quiet, 1);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_intensive_synthesis() {
    let state: InProgress = InProgress::new(&SETTINGS);
    let state = state.use_action(Action::IntensiveSynthesis, Condition::Good, &SETTINGS);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 194);
            assert_eq!(state.durability, 50);
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(400)
            );
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // can't use when condition isn't good or excellent
    let state = from_action_sequence(&SETTINGS, &[Action::IntensiveSynthesis]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_prudent_synthesis() {
    let state = from_action_sequence(&SETTINGS, &[Action::PrudentSynthesis]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 182);
            assert_eq!(state.durability, 55);
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(180)
            );
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
}

#[test]
fn test_trained_finesse() {
    let mut state: InProgress = InProgress::new(&SETTINGS);
    state.effects.inner_quiet = 10;
    match state.use_action(Action::TrainedFinesse, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(state.cp, 168);
            assert_eq!(state.durability, 60);
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(200)
            );
            assert_eq!(state.effects.inner_quiet, 10);
            assert_eq!(state.combo, None);
        }
        _ => panic!(),
    }
    // can't use TrainedFinesse when InnerQuiet < 10
    let mut state: InProgress = InProgress::new(&SETTINGS);
    state.effects.inner_quiet = 9;
    let state = state.use_action(Action::TrainedFinesse, Condition::Normal, &SETTINGS);
    assert!(matches!(state, State::Invalid));
}
