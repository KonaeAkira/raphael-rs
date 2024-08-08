use simulator::{Action, ActionMask, Settings};
use solvers::test_utils::*;

#[test]
fn indagator_3858_4057() {
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
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (13046, 23, 62));
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
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (11627, 13, 35));
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
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (6455, 14, 40));
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
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (6038, 14, 38));
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
        allowed_actions: ActionMask::from_level(100)
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, true).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (8589, 17, 46));
}
