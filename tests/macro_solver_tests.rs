use raphael_rs::game::{
    units::{Progress, Quality},
    Action, Condition, Settings, State,
};

use raphael_rs::solvers::MacroSolver;

fn solve(settings: &Settings) -> Option<Vec<Action>> {
    MacroSolver::new(settings.clone()).solve(State::new(settings))
}

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
fn test_01() {
    let settings = Settings {
        max_cp: 370,
        max_durability: 60,
        max_progress: Progress::from(2000),
        max_quality: Quality::from(40000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(1603));
}

#[test]
fn test_02() {
    let settings = Settings {
        max_cp: 553,
        max_durability: 70,
        max_progress: Progress::from(2400),
        max_quality: Quality::from(20000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(3316.25));
}

#[test]
fn test_03() {
    let settings = Settings {
        max_cp: 612,
        max_durability: 60,
        max_progress: Progress::from(2560),
        max_quality: Quality::from(40000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(3400));
}

#[test]
fn test_04() {
    let settings = Settings {
        max_cp: 400,
        max_durability: 60,
        max_progress: Progress::from(2000),
        max_quality: Quality::from(1000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(1000));
}

#[test]
fn test_05() {
    let settings = Settings {
        max_cp: 450,
        max_durability: 80,
        max_progress: Progress::from(2800),
        max_quality: Quality::from(40000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(1978.25));
}

#[test]
fn test_06() {
    let settings = Settings {
        max_cp: 540,
        max_durability: 70,
        max_progress: Progress::from(2700),
        max_quality: Quality::from(40000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(2752.5));
}

#[test]
fn test_07() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: Progress::from(2500),
        max_quality: Quality::from(40000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(4546.25));
}

#[test]
fn test_08() {
    let settings = Settings {
        max_cp: 701,
        max_durability: 60,
        max_progress: Progress::from(3950),
        max_quality: Quality::from(6950),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(2740));
}

#[test]
fn test_09() {
    let settings = Settings {
        max_cp: 606,
        max_durability: 80,
        max_progress: Progress::from(1200),
        max_quality: Quality::from(20000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(5173.75));
}

#[test]
fn test_10() {
    let settings = Settings {
        max_cp: 501,
        max_durability: 70,
        max_progress: Progress::from(1950),
        max_quality: Quality::from(20000),
    };
    let actions = solve(&settings).unwrap();
    let final_state = from_action_sequence(&settings, &actions)
        .as_completed()
        .unwrap();
    assert_eq!(final_state.quality, Quality::from(3220));
}
