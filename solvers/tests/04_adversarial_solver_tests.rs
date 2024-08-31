use simulator::{Action, ActionMask, Settings};
use solvers::test_utils::*;

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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2046, 19, 54));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2983, 22, 60));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (3159, 23, 62));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (1908, 18, 49));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2559, 21, 57));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2530, 24, 65));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (3004, 18, 49));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (10985, 17, 47));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2759, 17, 47));
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
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (11682, 16, 45));
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
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (12138, 32, 91));
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
        adversarial: true,
    };
    let actions = solve(&settings, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (8232, 14, 38));
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
        adversarial: true,
    };
    let actions = solve(&settings, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (10686, 26, 71));
}
