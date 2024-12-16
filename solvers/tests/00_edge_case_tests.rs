use simulator::*;
use solvers::test_utils::*;

#[test]
fn unsolvable() {
    let settings = Settings {
        max_cp: 100,
        max_durability: 60,
        max_progress: 4000,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false);
    assert_eq!(actions, None);
}

#[test]
fn zero_quality() {
    let settings = Settings {
        max_cp: 80,
        max_durability: 60,
        max_progress: 1920,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (0, 5, 14));
}

#[test]
fn max_quality() {
    let settings = Settings {
        max_cp: 400,
        max_durability: 60,
        max_progress: 2000,
        max_quality: 1000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (1100, 11, 28));
}

#[test]
fn large_progress_quality_increase() {
    let settings = Settings {
        max_cp: 300,
        max_durability: 40,
        max_progress: 100,
        max_quality: 100,
        base_progress: 5000,
        base_quality: 5000,
        job_level: 100,
        allowed_actions: ActionMask::all(),
        adversarial: false,
    };
    let actions = solve(&settings, false, false).unwrap();
    let score = get_score_triple(&settings, &actions);
    assert_eq!(score, (5000, 1, 3));
}
