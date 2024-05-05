use raphael::game::{
    units::{Progress, Quality},
    Action, Condition, Settings, State,
};

use raphael::solvers::MacroSolver;

fn solve(settings: &Settings) -> Option<Vec<Action>> {
    let result = MacroSolver::new(settings.clone()).solve(State::new(settings));
    dbg!(&result);
    result
}

fn get_quality(settings: &Settings, actions: &[Action]) -> f32 {
    let mut state: State = State::new(&settings);
    for action in actions {
        state = state.as_in_progress().unwrap().use_action(
            action.clone(),
            Condition::Normal,
            &settings,
        );
    }
    match state {
        State::Completed { missing_quality } => {
            settings.max_quality.saturating_sub(missing_quality).into()
        }
        _ => 0.0,
    }
}

#[test]
fn test_01() {
    let settings = Settings {
        max_cp: 370,
        max_durability: 60,
        max_progress: Progress::from(2000.00),
        max_quality: Quality::from(40000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 1653.00);
}

#[test]
fn test_02() {
    let settings = Settings {
        max_cp: 553,
        max_durability: 70,
        max_progress: Progress::from(2400.00),
        max_quality: Quality::from(20000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3352.50);
}

#[test]
fn test_03() {
    let settings = Settings {
        max_cp: 612,
        max_durability: 60,
        max_progress: Progress::from(2560.00),
        max_quality: Quality::from(40000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3407.50);
}

#[test]
fn test_04() {
    let settings = Settings {
        max_cp: 400,
        max_durability: 60,
        max_progress: Progress::from(2000.00),
        max_quality: Quality::from(1000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 1000.00);
}

#[test]
fn test_05() {
    let settings = Settings {
        max_cp: 450,
        max_durability: 80,
        max_progress: Progress::from(2800.00),
        max_quality: Quality::from(40000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2018.25);
}

#[test]
fn test_06() {
    let settings = Settings {
        max_cp: 540,
        max_durability: 70,
        max_progress: Progress::from(2700.00),
        max_quality: Quality::from(40000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2752.50);
}

#[test]
fn test_07() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: Progress::from(2500.00),
        max_quality: Quality::from(40000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4685.00);
}

#[test]
fn test_08() {
    let settings = Settings {
        max_cp: 701,
        max_durability: 60,
        max_progress: Progress::from(3950.00),
        max_quality: Quality::from(6950.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2840.00);
}

#[test]
fn test_09() {
    let settings = Settings {
        max_cp: 606,
        max_durability: 80,
        max_progress: Progress::from(1200.00),
        max_quality: Quality::from(20000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 5212.50);
}

#[test]
fn test_10() {
    let settings = Settings {
        max_cp: 501,
        max_durability: 70,
        max_progress: Progress::from(1950.00),
        max_quality: Quality::from(20000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3221.25);
}

#[test]
fn test_11() {
    let settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: Progress::from(2170.00),
        max_quality: Quality::from(20000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4405.00);
}

#[test]
fn test_rinascita_min_stats() {
    let settings = Settings {
        max_cp: 680,
        max_durability: 70,
        max_progress: Progress::from(2210.00),
        max_quality: Quality::from(20000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4695.00);
}

#[test]
fn test_pactmaker_min_stats() {
    let settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: Progress::from(2150.00),
        max_quality: Quality::from(20000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4052.50);
}

#[test]
fn test_diadochos_min_stats() {
    let settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: Progress::from(2705.00),
        max_quality: Quality::from(20000.00),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3818.75);
}
