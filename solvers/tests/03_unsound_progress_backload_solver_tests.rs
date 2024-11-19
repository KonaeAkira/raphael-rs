use simulator::*;
use solvers::test_utils::*;

#[test]
fn random_0f93c79f() {
    let settings = Settings {
        max_cp: 370,
        max_durability: 60,
        max_progress: 2000,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (1802, 16, 44));
    assert!(is_progress_backloaded(&actions, &settings));
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (3306, 20, 53));
    assert!(is_progress_backloaded(&actions, &settings));
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (3402, 25, 67));
    assert!(is_progress_backloaded(&actions, &settings));
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2013, 20, 55));
    assert!(is_progress_backloaded(&actions, &settings));
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2842, 23, 62));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn random_a300ca2b() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 2500,
        max_quality: 40000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (4633, 28, 75));
    assert!(is_progress_backloaded(&actions, &settings));
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2939, 24, 64));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn random_e451d981() {
    let settings = Settings {
        max_cp: 606,
        max_durability: 80,
        max_progress: 1200,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (5364, 27, 74));
    assert!(is_progress_backloaded(&actions, &settings));
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (3321, 19, 51));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn random_940b4755() {
    let settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: 2170,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (4449, 26, 69));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn rinascita_3700_3280() {
    let settings = Settings {
        max_cp: 680,
        max_durability: 70,
        max_progress: 5060,
        max_quality: 12628,
        base_progress: 229,
        base_quality: 224,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (10492, 25, 66));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn pactmaker_3240_3130() {
    let settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: 4300,
        max_quality: 12800,
        base_progress: 200,
        base_quality: 215,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (8801, 24, 65));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn pactmaker_3240_3130_heart_and_soul() {
    let settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: 4300,
        max_quality: 12800,
        base_progress: 200,
        base_quality: 215,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (9608, 24, 65));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn diadochos_4021_3660() {
    let settings = Settings {
        max_cp: 640,
        max_durability: 70,
        max_progress: 6600,
        max_quality: 14040,
        base_progress: 249,
        base_quality: 247,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (9580, 23, 61));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn indagator_3858_4057() {
    let settings = Settings {
        max_cp: 687,
        max_durability: 70,
        max_progress: 5720,
        max_quality: 12900,
        base_progress: 239,
        base_quality: 271,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (12313, 27, 72));
    assert!(is_progress_backloaded(&actions, &settings));
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (10752, 16, 44));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn random_48ae7c9f() {
    let settings = Settings {
        max_cp: 699,
        max_durability: 80,
        max_progress: 5700,
        max_quality: 20000,
        base_progress: 295,
        base_quality: 310,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (19445, 35, 98));
    assert!(is_progress_backloaded(&actions, &settings));
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
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (3002, 19, 51));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn rarefied_tacos_de_carne_asada_4785_4758() {
    let settings = Settings {
        max_cp: 646,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 256,
        base_quality: 265,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (12082, 22, 58));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn stuffed_peppers_2() {
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (19705, 29, 79));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn stuffed_peppers_2_heart_and_soul() {
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (21235, 32, 88));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn stuffed_peppers_2_quick_innovation() {
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (19984, 30, 83));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn rakaznar_lapidary_hammer_4462_4391() {
    let settings = Settings {
        max_cp: 569,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 6500, // full HQ mats, 12500 custom target
        base_progress: 237,
        base_quality: 245,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (7056, 16, 45));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn black_star_4048_3997() {
    let settings = Settings {
        max_cp: 596,
        max_durability: 40,
        max_progress: 3000,
        max_quality: 5500, // full HQ mats
        base_progress: 250,
        base_quality: 312,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (6114, 12, 31));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn claro_walnut_lumber_4900_4800() {
    let settings = Settings {
        max_cp: 620,
        max_durability: 40,
        max_progress: 3000,
        max_quality: 11000,
        base_progress: 300,
        base_quality: 368,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (11517, 14, 35));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn rakaznar_lapidary_hammer_4900_4800() {
    let settings = Settings {
        max_cp: 620,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 6000, // full hq-mats
        base_progress: 261,
        base_quality: 266,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (6015, 15, 41));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn rarefied_tacos_de_carne_asada_4966_4817() {
    let settings = Settings {
        max_cp: 626,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 5400, // full hq-mats, 95% target
        base_progress: 264,
        base_quality: 267,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (5717, 14, 38));
    assert!(is_progress_backloaded(&actions, &settings));
}

#[test]
fn archeo_kingdom_broadsword_4966_4914() {
    let settings = Settings {
        max_cp: 745,
        max_durability: 70,
        max_progress: 7500,
        max_quality: 8250, // full hq-mats
        base_progress: 264,
        base_quality: 271,
        job_level: 100,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (9049, 18, 49));
    assert!(is_progress_backloaded(&actions, &settings));
}
