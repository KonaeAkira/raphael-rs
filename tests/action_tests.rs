use raphael_rs::{
    config::Settings,
    game::{
        actions::{Action, PROG_DENOM, QUAL_DENOM},
        conditions::Condition,
        state::{InProgress, State},
    },
};

const SETTINGS: Settings = Settings {
    max_cp: 200,
    max_durability: 60,
    max_progress: (20.00 * PROG_DENOM) as i32,
    max_quality: (400.00 * QUAL_DENOM) as i32,
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
            assert_eq!(state.progress, (1.20 * PROG_DENOM) as i32);
            assert_eq!(state.last_action, Some(Action::BasicSynthesis));
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
            assert_eq!(state.quality, (1.00 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 1);
            assert_eq!(state.last_action, Some(Action::BasicTouch));
        }
        _ => panic!(),
    }
}

#[test]
fn test_standard_touch() {
    let state = from_action_sequence(&SETTINGS, &[Action::BasicTouch, Action::StandardTouch]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 164);
            assert_eq!(state.durability, 40);
            assert_eq!(state.quality, (2.375 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 2);
            assert_eq!(state.last_action, Some(Action::StandardTouch));
        }
        _ => panic!(),
    }
    // can't use without first using basic touch
    let state = from_action_sequence(&SETTINGS, &[Action::StandardTouch]);
    assert!(matches!(state, State::Invalid));
}

#[test]
fn test_advanced_touch() {
    let state = from_action_sequence(
        &SETTINGS,
        &[
            Action::BasicTouch,
            Action::StandardTouch,
            Action::AdvancedTouch,
        ],
    );
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 146);
            assert_eq!(state.durability, 30);
            assert_eq!(state.quality, (4.175 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 3);
            assert_eq!(state.last_action, Some(Action::AdvancedTouch));
        }
        _ => panic!(),
    }
    // can't use without first using basic touch
    let state = from_action_sequence(&SETTINGS, &[Action::AdvancedTouch]);
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
            assert_eq!(state.last_action, Some(Action::MasterMend));
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
            assert_eq!(state.last_action, Some(Action::Observe));
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
        },
        _ => panic!()
    }
    // CP after action can't exceed max cp
    let mut state: InProgress = InProgress::new(&SETTINGS);
    state.cp -= 10;
    let state = state.use_action(Action::TricksOfTheTrade, Condition::Excellent, &SETTINGS);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 200);
        },
        _ => panic!()
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
            assert_eq!(state.last_action, Some(Action::WasteNot));
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
            assert_eq!(state.last_action, Some(Action::WasteNot2));
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
            assert_eq!(state.last_action, Some(Action::Veneration));
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
            assert_eq!(state.last_action, Some(Action::Innovation));
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
            assert_eq!(state.last_action, Some(Action::GreatStrides));
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
            assert_eq!(state.quality, (2.32 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 0);
            assert_eq!(state.last_action, Some(Action::ByregotsBlessing));
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
            assert_eq!(state.quality, (2.25 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 2);
            assert_eq!(state.last_action, Some(Action::PreciseTouch));
        },
        _ => panic!()
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
            assert_eq!(state.progress, (3.00 * PROG_DENOM) as i32);
            assert_eq!(state.effects.muscle_memory, 5);
            assert_eq!(state.last_action, Some(Action::MuscleMemory));
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
            assert_eq!(state.progress, (1.80 * PROG_DENOM) as i32);
            assert_eq!(state.last_action, Some(Action::CarefulSynthesis));
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
            assert_eq!(state.last_action, Some(Action::Manipulation));
        }
        _ => panic!()
    }
}

#[test]
fn test_prudent_touch() {
    let state = from_action_sequence(&SETTINGS, &[Action::PrudentTouch]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 175);
            assert_eq!(state.durability, 55);
            assert_eq!(state.quality, (1.00 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 1);
            assert_eq!(state.last_action, Some(Action::PrudentTouch));
        }
        _ => panic!()
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
            assert_eq!(state.progress, (2.00 * PROG_DENOM) as i32);
            assert_eq!(state.last_action, Some(Action::FocusedSynthesis));
        }
        _ => panic!()
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
            assert_eq!(state.quality, (1.50 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 1);
            assert_eq!(state.last_action, Some(Action::FocusedTouch));
        }
        _ => panic!()
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
            assert_eq!(state.quality, (1.00 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 2);
            assert_eq!(state.last_action, Some(Action::Reflect));
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
            assert_eq!(state.quality, (2.00 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 2);
            assert_eq!(state.last_action, Some(Action::PreparatoryTouch));
        }
        _ => panic!()
    }
}

#[test]
fn test_groundwork() {
    let state = from_action_sequence(&SETTINGS, &[Action::Groundwork]);
    match state {
        State::InProgress(state) => {
            assert_eq!(state.cp, 182);
            assert_eq!(state.durability, 40);
            assert_eq!(state.progress, (3.60 * PROG_DENOM) as i32);
            assert_eq!(state.last_action, Some(Action::Groundwork));
        }
        _ => panic!()
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
            assert_eq!(state.progress, (1.00 * PROG_DENOM) as i32);
            assert_eq!(state.quality, (1.00 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 1);
            assert_eq!(state.last_action, Some(Action::DelicateSynthesis));
        }
        _ => panic!()
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
            assert_eq!(state.progress, (4.00 * PROG_DENOM) as i32);
            assert_eq!(state.last_action, Some(Action::IntensiveSynthesis));
        },
        _ => panic!()
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
            assert_eq!(state.progress, (1.80 * PROG_DENOM) as i32);
            assert_eq!(state.last_action, Some(Action::PrudentSynthesis));
        }
        _ => panic!()
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
            assert_eq!(state.quality, (2.00 * QUAL_DENOM) as i32);
            assert_eq!(state.effects.inner_quiet, 10);
            assert_eq!(state.last_action, Some(Action::TrainedFinesse));
        },
        _ => panic!()
    }
    // can't use TrainedFinesse when InnerQuiet < 10
    let mut state: InProgress = InProgress::new(&SETTINGS);
    state.effects.inner_quiet = 9;
    let state = state.use_action(Action::TrainedFinesse, Condition::Normal, &SETTINGS);
    assert!(matches!(state, State::Invalid));
}
