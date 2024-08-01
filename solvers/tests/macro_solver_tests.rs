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
        state = state
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 1802);
    assert_eq!(get_duration(&actions), 44);
    assert_eq!(actions.len(), 16);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3366);
    assert_eq!(get_duration(&actions), 53);
    assert_eq!(actions.len(), 20);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3434);
    assert_eq!(get_duration(&actions), 67);
    assert_eq!(actions.len(), 25);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 1100);
    assert_eq!(get_duration(&actions), 29);
    assert_eq!(actions.len(), 11);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 0);
    assert_eq!(get_duration(&actions), 14);
    assert_eq!(actions.len(), 5);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2018);
    assert_eq!(get_duration(&actions), 52);
    assert_eq!(actions.len(), 19);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2942);
    assert_eq!(get_duration(&actions), 56);
    assert_eq!(actions.len(), 21);
}

#[test]
fn test_backload_random_bb38a037() {
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 2842);
    assert_eq!(get_duration(&actions), 62);
    assert_eq!(actions.len(), 23);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4683);
    assert_eq!(get_duration(&actions), 69);
    assert_eq!(actions.len(), 26);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 2939);
    assert_eq!(get_duration(&actions), 64);
    assert_eq!(actions.len(), 24);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 5364);
    assert_eq!(get_duration(&actions), 74);
    assert_eq!(actions.len(), 27);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3321);
    assert_eq!(get_duration(&actions), 51);
    assert_eq!(actions.len(), 19);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 4483);
    assert_eq!(get_duration(&actions), 67);
    assert_eq!(actions.len(), 25);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 10623);
    assert_eq!(get_duration(&actions), 70);
    assert_eq!(actions.len(), 26);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 8912);
    assert_eq!(get_duration(&actions), 55);
    assert_eq!(actions.len(), 21);
}

#[test]
fn test_pactmaker_3240_3130_heart_and_soul() {
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
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 9608);
    assert_eq!(get_duration(&actions), 65);
    assert_eq!(actions.len(), 24);
}

#[test]
fn test_backload_pactmaker_3240_3130() {
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 8801);
    assert_eq!(get_duration(&actions), 65);
    assert_eq!(actions.len(), 24);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 9688);
    assert_eq!(get_duration(&actions), 68);
    assert_eq!(actions.len(), 25);
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 12793);
    assert_eq!(get_duration(&actions), 72);
    assert_eq!(actions.len(), 27);
}

#[test]
fn test_random_2ea6c001() {
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
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 10752);
    assert_eq!(get_duration(&actions), 44);
    assert_eq!(actions.len(), 16);
}

#[test]
fn test_random_48ae7c9f() {
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
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 19621);
    assert_eq!(get_duration(&actions), 84);
    assert_eq!(actions.len(), 31);
}

#[test]
fn test_backload_random_48ae7c9f() {
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 19445);
    assert_eq!(get_duration(&actions), 98);
    assert_eq!(actions.len(), 35);
}

#[test]
fn test_backload_random_48ae7c9f_quick_innovation() {
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
            .remove(Action::HeartAndSoul),
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 19677);
    assert_eq!(get_duration(&actions), 93);
    assert_eq!(actions.len(), 33);
}

#[test]
fn test_max_quality_indagator_3858_4057() {
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
        adversarial: false,
    };
    let actions = solve(&settings, false, true).unwrap();
    assert_eq!(get_quality(&settings, &actions), 13046);
    assert_eq!(get_duration(&actions), 62);
    assert_eq!(actions.len(), 23);
}

#[test]
fn test_random_4ecd54c4() {
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
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    assert_eq!(get_quality(&settings, &actions), 3080);
    assert_eq!(get_duration(&actions), 51);
    assert_eq!(actions.len(), 19);
}

#[test]
fn test_backload_random_4ecd54c4() {
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 3002);
    assert_eq!(get_duration(&actions), 51);
    assert_eq!(actions.len(), 19);
}

#[test]
fn test_trained_eye() {
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 9090);
    assert_eq!(get_duration(&actions), 16);
    assert_eq!(actions.len(), 6);
}

#[test]
fn test_rare_tacos() {
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions) + 6000, 11562);
    assert_eq!(get_duration(&actions), 41);
    // solver should prefer rotation with fewer steps when duration is the same (#39)
    assert_eq!(actions.len(), 15);
}

#[test]
fn test_mountain_chromite_ingot_no_manipulation() {
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    assert_eq!(get_quality(&settings, &actions), 8437);
    assert_eq!(get_duration(&actions), 36);
    assert_eq!(actions.len(), 13);
}

#[test]
fn test_rare_tacos_2() {
    // lv100 Rarefied Tacos de Carne Asada
    // 4785 CMS, 4758 Ctrl, 646 CP
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 256,
        base_quality: 265,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    dbg!(actions.clone());
    assert_eq!(get_quality(&settings, &actions), 12082);
    assert_eq!(get_duration(&actions), 58);
    assert_eq!(actions.len(), 22);
}

#[test]
fn test_stuffed_peppers_2() {
    // lv99 Rarefied Stuffed Peppers
    // 4785 CMS, 4758 Ctrl, 646 CP
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 40000,
        base_progress: 289,
        base_quality: 360,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    dbg!(actions.clone());
    assert_eq!(get_quality(&settings, &actions), 20177);
    assert_eq!(get_duration(&actions), 85);
    assert_eq!(actions.len(), 31);
}

#[test]
fn test_stuffed_peppers_2_heart_and_soul() {
    // lv99 Rarefied Stuffed Peppers
    // 4785 CMS, 4758 Ctrl, 646 CP
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 40000,
        base_progress: 289,
        base_quality: 360,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    dbg!(actions.clone());
    assert_eq!(get_quality(&settings, &actions), 21536);
    assert_eq!(get_duration(&actions), 83);
    assert_eq!(actions.len(), 30);
}

#[test]
fn test_stuffed_peppers_2_quick_innovation() {
    // lv99 Rarefied Stuffed Peppers
    // 4785 CMS, 4758 Ctrl, 646 CP
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 40000,
        base_progress: 289,
        base_quality: 360,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    dbg!(actions.clone());
    assert_eq!(get_quality(&settings, &actions), 20502);
    assert_eq!(get_duration(&actions), 77);
    assert_eq!(actions.len(), 28);
}

#[test]
fn test_rakaznar_lapidary_hammer() {
    // Ra'Kaznar Lapidary Hammer
    // 4462 Craftsmanship, 4391 Control
    let settings = Settings {
        max_cp: 569,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 6500, // full HQ mats, 12500 custom target
        base_progress: 237,
        base_quality: 245,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    dbg!(actions.clone());
    assert_eq!(get_quality(&settings, &actions), 7056);
    assert_eq!(get_duration(&actions), 45);
    assert_eq!(actions.len(), 16);
}

#[test]
fn test_black_star() {
    // Black Star
    // 4068 Craftsmanship, 3997 Control
    let settings = Settings {
        max_cp: 596,
        max_durability: 40,
        max_progress: 3000,
        max_quality: 5500, // full HQ mats
        base_progress: 250,
        base_quality: 312,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    dbg!(actions.clone());
    assert_eq!(get_quality(&settings, &actions), 6114);
    assert_eq!(get_duration(&actions), 31);
    assert_eq!(actions.len(), 12);
}
