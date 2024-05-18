use raphael::game::{units::Quality, Action, ActionMask, Condition, Settings, State};

use raphael::solvers::MacroSolver;

fn solve(settings: &Settings) -> Option<Vec<Action>> {
    MacroSolver::new(settings.clone()).solve(State::new(settings))
}

fn get_quality(settings: &Settings, actions: &[Action]) -> Quality {
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
            settings.max_quality.saturating_sub(missing_quality)
        }
        _ => 0,
    }
}

#[test]
fn test_random_0f93c79f() {
    let settings = Settings {
        max_cp: 370,
        max_durability: 60,
        max_progress: 2000,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 1682);
}

#[test]
fn test_random_1e281667() {
    let settings = Settings {
        max_cp: 553,
        max_durability: 70,
        max_progress: 2400,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3351);
}

#[test]
fn test_random_d0bf2aef() {
    let settings = Settings {
        max_cp: 612,
        max_durability: 60,
        max_progress: 2560,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3405);
}

#[test]
fn test_unsolvable() {
    let settings = Settings {
        max_cp: 100,
        max_durability: 60,
        max_progress: 4000,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings);
    assert_eq!(actions, None);
}

#[test]
fn test_max_quality() {
    let settings = Settings {
        max_cp: 400,
        max_durability: 60,
        max_progress: 2000,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 1000);
}

#[test]
fn test_zero_quality() {
    let settings = Settings {
        max_cp: 80,
        max_durability: 60,
        max_progress: 1920,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 0);
}

#[test]
fn test_random_e413e05d() {
    let settings = Settings {
        max_cp: 450,
        max_durability: 80,
        max_progress: 2800,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2017);
}

#[test]
fn test_random_bb38a037() {
    let settings = Settings {
        max_cp: 540,
        max_durability: 70,
        max_progress: 2700,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2751);
}

#[test]
fn test_random_a300ca2b() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 2500,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4683);
}

#[test]
fn test_random_0f9d7781() {
    let settings = Settings {
        max_cp: 701,
        max_durability: 60,
        max_progress: 3950,
        max_quality: 6950,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2839);
}

#[test]
fn test_random_e451d981() {
    let settings = Settings {
        max_cp: 606,
        max_durability: 80,
        max_progress: 1200,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 5210);
}

#[test]
fn test_random_6799bb1d() {
    let settings = Settings {
        max_cp: 501,
        max_durability: 70,
        max_progress: 1950,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3259);
}

#[test]
fn test_random_940b4755() {
    let settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: 2170,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4402);
}

#[test]
fn test_rinascita_3700_3280() {
    let settings = Settings {
        max_cp: 680,
        max_durability: 70,
        max_progress: 5060,
        max_quality: 12628,
        base_progress: 229,
        base_quality: 224,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 10513);
}

#[test]
fn test_pactmaker_3240_3130() {
    let settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: 4300,
        max_quality: 12800,
        base_progress: 200,
        base_quality: 215,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 8758);
}

#[test]
fn test_diadochos_4021_3660() {
    let settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: 6600,
        max_quality: 14040,
        base_progress: 249,
        base_quality: 247,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 9688);
}

#[test]
fn test_indagator_3858_4057() {
    let settings = Settings {
        max_cp: 687,
        max_durability: 70,
        max_progress: 5720,
        max_quality: 12900,
        base_progress: 239,
        base_quality: 271,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let actions = solve(&settings).unwrap();
    assert_eq!(get_quality(&settings, &actions), 12793);
}
