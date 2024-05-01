use raphael::game::{
    state::InProgress,
    units::{Progress, Quality},
    Action, Condition, Settings, State,
};

const SETTINGS: Settings = Settings {
    max_cp: 200,
    max_durability: 60,
    max_progress: Progress::new(2000),
    max_quality: Quality::new(40000),
};

#[test]
fn test_muscle_memory() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.muscle_memory = 3;
    match state.use_action(Action::CarefulSynthesis, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(Progress::from(360.00))
            );
            assert_eq!(state.effects.muscle_memory, 0);
        }
        _ => panic!(),
    }
    match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(state.effects.muscle_memory, 2);
        }
        _ => panic!(),
    }
}

#[test]
fn test_veneration() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.veneration = 3;
    match state.use_action(Action::CarefulSynthesis, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(Progress::from(270.00))
            );
            assert_eq!(state.effects.veneration, 2);
        }
        _ => panic!(),
    }
}

#[test]
fn test_muscle_memory_veneration() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.muscle_memory = 3;
    state.effects.veneration = 3;
    match state.use_action(Action::CarefulSynthesis, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(
                state.missing_progress,
                SETTINGS.max_progress.saturating_sub(Progress::from(450.00))
            );
            assert_eq!(state.effects.muscle_memory, 0);
            assert_eq!(state.effects.veneration, 2);
        }
        _ => panic!(),
    }
}

#[test]
fn test_waste_not() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.waste_not = 3;
    match state.use_action(Action::CarefulSynthesis, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(state.durability, 55);
            assert_eq!(state.effects.waste_not, 2);
        }
        _ => panic!(),
    }
    match state.use_action(Action::PrudentTouch, Condition::Normal, &SETTINGS) {
        State::Invalid => (),
        _ => panic!(),
    }
    match state.use_action(Action::PrudentSynthesis, Condition::Normal, &SETTINGS) {
        State::Invalid => (),
        _ => panic!(),
    }
}

#[test]
fn test_manipulation() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.manipulation = 3;
    state.durability = 30;
    match state.use_action(Action::BasicSynthesis, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(state.durability, 25);
            assert_eq!(state.effects.manipulation, 2);
        }
        _ => panic!(),
    }
    match state.use_action(Action::Observe, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(state.durability, 35);
            assert_eq!(state.effects.manipulation, 2);
        }
        _ => panic!(),
    }
    match state.use_action(Action::Manipulation, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(state.durability, 30);
            assert_eq!(state.effects.manipulation, 8);
        }
        _ => panic!(),
    }
    match state.use_action(Action::MasterMend, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(state.durability, 60);
            assert_eq!(state.effects.manipulation, 2);
        }
        _ => panic!(),
    }
}

#[test]
fn test_great_strides() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.great_strides = 3;
    match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(Quality::from(200.00))
            );
            assert_eq!(state.effects.great_strides, 0);
        }
        _ => panic!(),
    }
    match state.use_action(Action::BasicSynthesis, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(state.effects.great_strides, 2);
        }
        _ => panic!(),
    }
}

#[test]
fn test_innovation() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.innovation = 3;
    match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(Quality::from(150.00))
            );
            assert_eq!(state.effects.innovation, 2);
        }
        _ => panic!(),
    }
}

#[test]
fn test_great_strides_innovation() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.great_strides = 3;
    state.effects.innovation = 3;
    match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(Quality::from(250.00))
            );
            assert_eq!(state.effects.great_strides, 0);
            assert_eq!(state.effects.innovation, 2);
        }
        _ => panic!(),
    }
}

#[test]
fn test_inner_quiet() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.inner_quiet = 4;
    match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(Quality::from(140.00))
            );
            assert_eq!(state.effects.inner_quiet, 5);
        }
        _ => panic!(),
    }
}

#[test]
fn test_innovation_inner_quiet() {
    let mut state = InProgress::new(&SETTINGS);
    state.effects.innovation = 3;
    state.effects.inner_quiet = 4;
    match state.use_action(Action::BasicTouch, Condition::Normal, &SETTINGS) {
        State::InProgress(state) => {
            assert_eq!(
                state.missing_quality,
                SETTINGS.max_quality.saturating_sub(Quality::from(210.00))
            );
            assert_eq!(state.effects.innovation, 2);
            assert_eq!(state.effects.inner_quiet, 5);
        }
        _ => panic!(),
    }
}
