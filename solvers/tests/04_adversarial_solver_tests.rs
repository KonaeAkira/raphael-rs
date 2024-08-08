use simulator::{Action, ActionMask, Condition, Settings, SimulationState};
use solvers::MacroSolver;

fn solve(
    settings: &Settings,
    backload_progress: bool,
    minimize_steps: bool,
) -> Option<Vec<Action>> {
    MacroSolver::new(settings.clone(), Box::new(|_| {}), Box::new(|_| {})).solve(
        SimulationState::new(settings),
        backload_progress,
        minimize_steps,
    )
}

fn get_quality(settings: &Settings, actions: &[Action]) -> u16 {
    let mut state = SimulationState::new(&settings);
    for action in actions {
        state = SimulationState::try_from(state)
            .unwrap()
            .use_action(action.clone(), Condition::Normal, &settings)
            .unwrap();
    }
    assert!(state.progress >= settings.max_progress);
    state.get_quality()
}

fn get_duration(actions: &[Action]) -> i16 {
    actions.into_iter().map(|action| action.time_cost()).sum()
}

fn is_progress_backloaded(actions: &[Action]) -> bool {
    let first_progress_action = actions
        .iter()
        .position(|action| action.progress_efficiency(1) != 0)
        .unwrap();
    // there musn't be any Quality-increasing actions after the first Progress-increasing action
    !actions
        .into_iter()
        .skip(first_progress_action)
        .any(|action| action.quality_efficiency(10) != 0)
}

const SETTINGS: Settings = Settings {
    max_cp: 370,
    max_durability: 60,
    max_progress: 2000,
    max_quality: 40000,
    base_progress: 100,
    base_quality: 100,
    job_level: 100,
    allowed_actions: ActionMask::all()
        .remove(Action::TrainedEye)
        .remove(Action::HeartAndSoul)
        .remove(Action::QuickInnovation),
    adversarial: true,
};

#[test]
fn random_0f93c79f() {
    let settings = Settings {
        max_cp: 370,
        max_durability: 60,
        max_progress: 2000,
        ..SETTINGS
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2046);
    assert_eq!(get_duration(&actions), 54);
    assert_eq!(actions.len(), 19);
}

#[test]
fn random_1e281667() {
    let settings = Settings {
        max_cp: 553,
        max_durability: 70,
        max_progress: 2400,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2983);
    assert_eq!(get_duration(&actions), 60);
    assert_eq!(actions.len(), 22);
}

#[test]
fn random_d0bf2aef() {
    let settings = Settings {
        max_cp: 612,
        max_durability: 60,
        max_progress: 2560,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3159);
    assert_eq!(get_duration(&actions), 62);
    assert_eq!(actions.len(), 23);
}

#[test]
fn random_e413e05d() {
    let settings = Settings {
        max_cp: 450,
        max_durability: 80,
        max_progress: 2800,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 1908);
    assert_eq!(get_duration(&actions), 49);
    assert_eq!(actions.len(), 18);
}

#[test]
fn random_bb38a037() {
    let settings = Settings {
        max_cp: 540,
        max_durability: 70,
        max_progress: 2700,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2559);
    assert_eq!(get_duration(&actions), 57);
    assert_eq!(actions.len(), 21);
}

#[test]
fn backload_random_bb38a037() {
    let settings = Settings {
        max_cp: 540,
        max_durability: 70,
        max_progress: 2700,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 2514);
    assert_eq!(get_duration(&actions), 57);
    assert_eq!(actions.len(), 21);
}

#[ignore]
#[test]
fn random_a300ca2b() {
    // Took 22 minutes.
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 2500,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3973);
    assert_eq!(get_duration(&actions), 76);
    assert_eq!(actions.len(), 28);
}

#[test]
fn random_0f9d7781() {
    let settings = Settings {
        max_cp: 701,
        max_durability: 60,
        max_progress: 3950,
        max_quality: 6950,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2530);
    assert_eq!(get_duration(&actions), 65);
    assert_eq!(actions.len(), 24);
}

#[ignore]
#[test]
fn random_e451d981() {
    // Took 11 minutes.
    let settings = Settings {
        max_cp: 606,
        max_durability: 80,
        max_progress: 1200,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4547);
    assert_eq!(get_duration(&actions), 74);
    assert_eq!(actions.len(), 27);
}

#[test]
fn random_6799bb1d() {
    let settings = Settings {
        max_cp: 501,
        max_durability: 70,
        max_progress: 1950,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3004);
    assert_eq!(get_duration(&actions), 49);
    assert_eq!(actions.len(), 18);
}

#[ignore]
#[test]
fn random_940b4755() {
    // Took 16 minutes.
    let settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: 2170,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3761);
    assert_eq!(get_duration(&actions), 68);
    assert_eq!(actions.len(), 25);
}

#[ignore]
#[test]
fn rinascita_3700_3280() {
    // Took 16 minutes.
    let settings = Settings {
        max_cp: 680,
        max_durability: 70,
        max_progress: 5060,
        max_quality: 12628,
        base_progress: 229,
        base_quality: 224,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 9254);
    assert_eq!(get_duration(&actions), 68);
    assert_eq!(actions.len(), 25);
}

#[ignore]
#[test]
fn pactmaker_3240_3130() {
    // Took 8 minutes.
    let settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: 4300,
        max_quality: 12800,
        base_progress: 200,
        base_quality: 215,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 7494);
    assert_eq!(get_duration(&actions), 62);
    assert_eq!(actions.len(), 23);
}

#[test]
fn backload_pactmaker_3240_3130() {
    let settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: 4300,
        max_quality: 12800,
        base_progress: 200,
        base_quality: 215,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 7575);
    assert_eq!(get_duration(&actions), 75);
    assert_eq!(actions.len(), 27);
}

#[ignore]
#[test]
fn diadochos_4021_3660() {
    // Took 4 minutes.
    let settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: 6600,
        max_quality: 14040,
        base_progress: 249,
        base_quality: 247,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 8489);
    assert_eq!(get_duration(&actions), 65);
    assert_eq!(actions.len(), 24);
}

#[ignore]
#[test]
fn indagator_3858_4057() {
    // Took 26 minutes.
    let settings = Settings {
        max_cp: 687,
        max_durability: 70,
        max_progress: 5720,
        max_quality: 12900,
        base_progress: 239,
        base_quality: 271,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 10675);
    assert_eq!(get_duration(&actions), 65);
    assert_eq!(actions.len(), 24);
}

#[test]
fn random_2ea6c001() {
    let settings = Settings {
        max_cp: 720,
        max_durability: 80,
        max_progress: 5700,
        max_quality: 10600,
        base_progress: 241,
        base_quality: 322,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 10768);
    assert_eq!(get_duration(&actions), 49);
    assert_eq!(actions.len(), 17);
}

#[ignore]
#[test]
fn random_48ae7c9f() {
    // Took 69 minutes.
    let settings = Settings {
        max_cp: 699,
        max_durability: 80,
        max_progress: 5700,
        max_quality: 20000,
        base_progress: 295,
        base_quality: 310,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 17236);
    assert_eq!(get_duration(&actions), 89);
    assert_eq!(actions.len(), 31);
}

#[ignore]
#[test]
fn backload_random_48ae7c9f() {
    // Took 6 minutes.
    let settings = Settings {
        max_cp: 699,
        max_durability: 80,
        max_progress: 5700,
        max_quality: 20000,
        base_progress: 295,
        base_quality: 310,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 16969);
    assert_eq!(get_duration(&actions), 85);
    assert_eq!(actions.len(), 30);
}

#[ignore]
#[test]
fn max_quality_indagator_3858_4057() {
    // Took 26 minutes.
    let settings = Settings {
        max_cp: 714,
        max_durability: 70,
        max_progress: 5720,
        max_quality: 12900,
        base_progress: 239,
        base_quality: 271,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 11377);
    assert_eq!(get_duration(&actions), 73);
    assert_eq!(actions.len(), 27);
}

#[test]
fn random_4ecd54c4() {
    let settings = Settings {
        max_cp: 456,
        max_durability: 80,
        max_progress: 2024,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2759);
    assert_eq!(get_duration(&actions), 47);
    assert_eq!(actions.len(), 17);
}

#[test]
fn backload_random_4ecd54c4() {
    let settings = Settings {
        max_cp: 456,
        max_durability: 80,
        max_progress: 2024,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 100,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 2717);
    assert_eq!(get_duration(&actions), 52);
    assert_eq!(actions.len(), 19);
}

#[test]
fn trained_eye() {
    // Grade 8 Vitality Alkahest
    // 4005 Craftsmanship, 3961 Control, Level 100
    let settings = Settings {
        max_cp: 604,
        max_durability: 35,
        max_progress: 4488,
        max_quality: 9090,
        base_progress: 310,
        base_quality: 379,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 9090);
    assert_eq!(get_duration(&actions), 16);
    assert_eq!(actions.len(), 6);
}

#[test]
fn rare_tacos() {
    // Rarefied Tacos de Carne Asada
    // 4694 Craftsmanship, 4327 Control, Level 100, HQ Jhinga Biryani, 2/2 HQ mats
    let settings = Settings {
        max_cp: 663,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 11400 - 6000,
        base_progress: 250,
        base_quality: 246,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions) + 6000, 11442);
    assert_eq!(get_duration(&actions), 43);
    // solver should prefer rotation with fewer steps when duration is the same (#39)
    assert_eq!(actions.len(), 16);
}

#[test]
fn mountain_chromite_ingot_no_manipulation() {
    // Mountain Chromite Ingot
    // 3076 Craftsmanship, 3106 Control, Level 90, HQ Tsai Tou Vonou
    let settings = Settings {
        max_cp: 616,
        max_durability: 40,
        max_progress: 2000,
        max_quality: 8200,
        base_progress: 217,
        base_quality: 293,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::Manipulation)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 8232);
    assert_eq!(get_duration(&actions), 38);
    assert_eq!(actions.len(), 14);
}

// This test takes a long time to run right now.
// This test does work, but it takes several hours to run.
#[ignore]
#[test]
fn rare_tacos_2() {
    // lv100 Rarefied Tacos de Carne Asada
    // 4785 CMS, 4758 Ctrl, 646 CP
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 256,
        base_quality: 265,
        ..SETTINGS
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2046);
    assert_eq!(get_duration(&actions), 54);
    assert_eq!(actions.len(), 19);
}

#[test]
fn stuffed_peppers() {
    // lv99 Rarefied Stuffed Peppers
    // 4785 CMS, 4758 Ctrl, 646 CP
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 11400,
        base_progress: 289,
        base_quality: 360,
        ..SETTINGS
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 12024);
    assert_eq!(get_duration(&actions), 47);
    assert_eq!(actions.len(), 17);
}
