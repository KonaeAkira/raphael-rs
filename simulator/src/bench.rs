use crate::*;

#[bench]
fn from_macro(bencher: &mut test::Bencher) {
    let settings = Settings {
        max_cp: 1000,
        max_durability: 80,
        max_progress: 50000,
        max_quality: 50000,
        base_progress: 321,
        base_quality: 321,
        job_level: 100,
        allowed_actions: ActionMask::all(),
        adversarial: false,
    };

    let actions = [
        Action::Reflect,
        Action::Manipulation,
        Action::Observe,
        Action::AdvancedTouch,
        Action::Innovation,
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
        Action::PreparatoryTouch,
        Action::ImmaculateMend,
        Action::Innovation,
        Action::PrudentTouch,
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
        Action::ByregotsBlessing,
        Action::TrainedPerfection,
        Action::ImmaculateMend,
        Action::Veneration,
        Action::Groundwork,
    ];

    bencher.iter(|| {
        SimulationState::from_macro(
            std::hint::black_box(&settings),
            std::hint::black_box(&actions),
        )
    });
}

#[bench]
fn from_macro_adversarial(bencher: &mut test::Bencher) {
    let settings = Settings {
        max_cp: 1000,
        max_durability: 80,
        max_progress: 50000,
        max_quality: 50000,
        base_progress: 321,
        base_quality: 321,
        job_level: 100,
        allowed_actions: ActionMask::all(),
        adversarial: true,
    };

    let actions = [
        Action::Reflect,
        Action::Manipulation,
        Action::Observe,
        Action::AdvancedTouch,
        Action::Innovation,
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
        Action::PreparatoryTouch,
        Action::ImmaculateMend,
        Action::Innovation,
        Action::PrudentTouch,
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
        Action::ByregotsBlessing,
        Action::TrainedPerfection,
        Action::ImmaculateMend,
        Action::Veneration,
        Action::Groundwork,
    ];

    bencher.iter(|| {
        SimulationState::from_macro(
            std::hint::black_box(&settings),
            std::hint::black_box(&actions),
        )
    });
}
