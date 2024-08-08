use simulator::{Action, ActionMask, Settings};
use solvers::test_utils::*;

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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (2842, 23, 62));
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
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (8801, 24, 65));
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
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (19445, 35, 98));
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (3002, 19, 51));
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (9090, 6, 16));
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (5562, 15, 41));
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
        adversarial: false,
    };
    let actions = solve(&settings, true, false).unwrap();
    assert!(is_progress_backloaded(&actions));
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (8437, 13, 36));
}

#[test]
fn random_48ae7c9f_quick_innovation() {
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
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (19677, 33, 93));
}
