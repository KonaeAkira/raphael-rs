use simulator::{state::InProgress, Action, ActionMask, Condition, Settings, SimulationState};

fn simulate(
    settings: &Settings,
    steps: impl Iterator<Item = (Action, Condition)>,
) -> Result<Vec<SimulationState>, &'static str> {
    let mut state = SimulationState::new(&settings);
    let mut result = Vec::new();
    for (action, condition) in steps {
        let in_progress: InProgress = state.try_into()?;
        state = in_progress.use_action(action, condition, &settings)?;
        result.push(state);
    }
    Ok(result)
}

fn progress_quality_pair(settings: &Settings, state: SimulationState) -> (u32, u32) {
    (
        settings.max_progress - state.missing_progress,
        settings.max_quality - state.missing_quality,
    )
}

#[test]
fn test_random_926ae85b() {
    // Copper Gorget
    // 10 Craftsmanship, 10 Control
    let settings = Settings {
        max_cp: 50,
        max_durability: 60,
        max_progress: 33,
        max_quality: 150,
        base_progress: 4,
        base_quality: 38,
        job_level: 10,
        allowed_actions: ActionMask::none(),
    };
    let actions = [
        Action::BasicSynthesis,
        Action::BasicTouch,
        Action::BasicTouch,
    ];
    let simulation = simulate(
        &settings,
        actions
            .into_iter()
            .zip(std::iter::repeat(Condition::Normal)),
    );
    let state = simulation.unwrap().last().copied().unwrap();
    assert_eq!(state.cp, 14);
    assert_eq!(state.durability, 30);
    assert_eq!(settings.max_progress - state.missing_progress, 4);
    assert_eq!(settings.max_quality - state.missing_quality, 76);
    assert_eq!(state.effects.inner_quiet, 0);
}

#[test]
fn test_random_3c721e47() {
    // Ironwood Spear
    // 3000 Craftsmanship, 3000 Control
    let settings = Settings {
        max_cp: 500,
        max_durability: 80,
        max_progress: 3100,
        max_quality: 6800,
        base_progress: 240,
        base_quality: 307,
        job_level: 85,
        allowed_actions: ActionMask::none(),
    };
    let actions = [
        Action::MuscleMemory,
        Action::Veneration,
        Action::WasteNot,
        Action::Groundwork,
        Action::Manipulation,
        Action::Innovation,
        Action::PreparatoryTouch,
        Action::PrudentTouch,
    ];
    let simulation = simulate(
        &settings,
        actions
            .into_iter()
            .zip(std::iter::repeat(Condition::Normal)),
    );
    let state = simulation.unwrap().last().copied().unwrap();
    assert_eq!(state.cp, 223);
    assert_eq!(state.durability, 60);
    assert_eq!(settings.max_progress - state.missing_progress, 2520);
    assert_eq!(settings.max_quality - state.missing_quality, 1473);
}

#[test]
fn test_random_3ba90d3a() {
    // Grade 4 Skybuilders' Stone
    // 1826 Craftsmanship, 1532 Control
    let settings = Settings {
        max_cp: 427,
        max_durability: 60,
        max_progress: 1080,
        max_quality: 9900,
        base_progress: 204,
        base_quality: 253,
        job_level: 81,
        allowed_actions: ActionMask::none(),
    };
    let actions = [
        Action::Veneration,
        Action::CarefulSynthesis,
        Action::CarefulSynthesis,
        Action::PreparatoryTouch,
        Action::MasterMend,
        Action::Innovation,
        Action::PrudentTouch,
        Action::BasicTouch,
        Action::ComboStandardTouch,
    ];
    let simulation = simulate(
        &settings,
        actions
            .into_iter()
            .zip(std::iter::repeat(Condition::Normal)),
    );
    let state = simulation.unwrap().last().copied().unwrap();
    assert_eq!(state.cp, 188);
    assert_eq!(state.durability, 25);
    assert_eq!(settings.max_progress - state.missing_progress, 918);
    assert_eq!(settings.max_quality - state.missing_quality, 2118);
    assert_eq!(state.effects.inner_quiet, 5);
    assert_eq!(state.effects.innovation, 1);
}

#[test]
fn test_random_bce2650c() {
    // Diadochos Wristband of Healing
    // 4020 Craftsmanship, 4042 Control
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 6600,
        max_quality: 14040,
        base_progress: 248,
        base_quality: 270,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, false),
    };
    let actions = [
        Action::MuscleMemory,
        Action::WasteNot,
        Action::Veneration,
        Action::Groundwork,
        Action::Groundwork,
        Action::Groundwork,
        Action::PrudentSynthesis,
        Action::MasterMend,
        Action::PrudentTouch,
        Action::Innovation,
        Action::PrudentTouch,
        Action::PrudentTouch,
        Action::PrudentTouch,
        Action::PrudentTouch,
        Action::MasterMend,
        Action::Innovation,
        Action::PrudentTouch,
        Action::BasicTouch,
        Action::ComboStandardTouch,
        Action::ComboAdvancedTouch,
        Action::GreatStrides,
        Action::Innovation,
        Action::Observe,
        Action::FocusedTouch,
        Action::GreatStrides,
        Action::ByregotsBlessing,
    ];
    let simulation = simulate(
        &settings,
        actions
            .into_iter()
            .zip(std::iter::repeat(Condition::Normal)),
    );
    let state = simulation.unwrap().last().copied().unwrap();
    assert_eq!(state.cp, 1);
    assert_eq!(state.durability, 5);
    assert_eq!(settings.max_progress - state.missing_progress, 6323);
    assert_eq!(settings.max_quality - state.missing_quality, 11475);
}

#[test]
fn test_ingame_be9fc5c2() {
    // Classical Index
    // 4000 Craftsmanship, 3962 Control
    let settings = Settings {
        max_cp: 594,
        max_durability: 70,
        max_progress: 3900,
        max_quality: 10920,
        base_progress: 247,
        base_quality: 265,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let states: Vec<(u32, u32)> = simulate(
        &settings,
        [
            (Action::Reflect, Condition::Normal), // 0, 265
            (Action::Manipulation, Condition::Normal),
            (Action::PreciseTouch, Condition::Excellent), // 0, 2173
            (Action::WasteNot, Condition::Poor),
            (Action::PreparatoryTouch, Condition::Normal), // 0, 2915
            (Action::MasterMend, Condition::Normal),
            (Action::PreparatoryTouch, Condition::Normal), // 0, 3763
            (Action::Innovation, Condition::Normal),
            (Action::BasicTouch, Condition::Normal), // 0, 4478
            (Action::ComboStandardTouch, Condition::Normal), // 0, 5422
            (Action::ComboAdvancedTouch, Condition::Normal), // 0, 6614
            (Action::PrudentTouch, Condition::Normal), // 0, 7409
            (Action::GreatStrides, Condition::Normal),
            (Action::Innovation, Condition::Normal),
            (Action::Observe, Condition::Normal),
            (Action::FocusedTouch, Condition::Normal), // 0, 9396
            (Action::Veneration, Condition::Normal),
            (Action::Groundwork, Condition::Normal), // 1333, 9396
            (Action::DelicateSynthesis, Condition::Normal), // 1703, 9926
        ]
        .into_iter(),
    )
    .unwrap()
    .into_iter()
    .map(|state| progress_quality_pair(&settings, state))
    .collect();
    let expected = [
        (0, 265),
        (0, 265),
        (0, 2173),
        (0, 2173),
        (0, 2915),
        (0, 2915),
        (0, 3763),
        (0, 3763),
        (0, 4478),
        (0, 5422),
        (0, 6614),
        (0, 7409),
        (0, 7409),
        (0, 7409),
        (0, 7409),
        (0, 9396),
        (0, 9396),
        (1333, 9396),
        (1703, 9926),
    ];
    assert_eq!(states, expected);
}
