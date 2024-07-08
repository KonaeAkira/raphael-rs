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

fn progress_quality_pair(settings: &Settings, state: SimulationState) -> (u16, u16) {
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
        initial_quality: 0,
        job_level: 10,
        allowed_actions: ActionMask::all(),
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
    assert_eq!(state.effects.inner_quiet(), 0);
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
        initial_quality: 0,
        job_level: 85,
        allowed_actions: ActionMask::all(),
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
        initial_quality: 0,
        job_level: 81,
        allowed_actions: ActionMask::all(),
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
    assert_eq!(state.effects.inner_quiet(), 5);
    assert_eq!(state.effects.innovation(), 1);
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
        initial_quality: 0,
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
        Action::ComboAdvancedTouch,
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
        initial_quality: 0,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };
    let states: Vec<(u16, u16)> = simulate(
        &settings,
        [
            (Action::Reflect, Condition::Normal), // 0, 795
            (Action::Manipulation, Condition::Normal),
            (Action::PreciseTouch, Condition::Excellent), // 0, 2703
            (Action::WasteNot, Condition::Poor),
            (Action::PreparatoryTouch, Condition::Normal), // 0, 3445
            (Action::MasterMend, Condition::Normal),
            (Action::PreparatoryTouch, Condition::Normal), // 0, 4293
            (Action::Innovation, Condition::Normal),
            (Action::BasicTouch, Condition::Normal), // 0, 5008
            (Action::ComboStandardTouch, Condition::Normal), // 0, 5952
            (Action::ComboAdvancedTouch, Condition::Normal), // 0, 7144
            (Action::PrudentTouch, Condition::Normal), // 0, 7939
            (Action::GreatStrides, Condition::Normal),
            (Action::Innovation, Condition::Normal),
            (Action::Observe, Condition::Normal),
            (Action::ComboAdvancedTouch, Condition::Normal), // 0, 9926
            (Action::Veneration, Condition::Normal),
            (Action::Groundwork, Condition::Normal), // 1333, 9926
            (Action::DelicateSynthesis, Condition::Normal), // 1703, 10456
        ]
        .into_iter(),
    )
    .unwrap()
    .into_iter()
    .map(|state| progress_quality_pair(&settings, state))
    .collect();
    let expected = [
        (0, 795),
        (0, 795),
        (0, 2703),
        (0, 2703),
        (0, 3445),
        (0, 3445),
        (0, 4293),
        (0, 4293),
        (0, 5008),
        (0, 5952),
        (0, 7144),
        (0, 7939),
        (0, 7939),
        (0, 7939),
        (0, 7939),
        (0, 9926),
        (0, 9926),
        (1333, 9926),
        (1703, 10456),
    ];
    assert_eq!(states, expected);
}

#[test]
fn test_ingame_d11d9c68() {
    // Black Star Mask of Casting
    // Lv. 94, 3957 Craftsmanship, 3896 Control
    // Verified in-game (patch 7.0)
    let settings = Settings {
        max_cp: 601,
        max_durability: 80,
        max_progress: 6300,
        max_quality: 11400,
        base_progress: 238,
        base_quality: 300,
        initial_quality: 0,
        job_level: 94,
        allowed_actions: ActionMask::all(),
    };
    let actions = [
        Action::Reflect,
        Action::Innovation,
        Action::PreparatoryTouch,
        Action::PrudentTouch,
        Action::GreatStrides,
        Action::PreparatoryTouch,
        Action::GreatStrides,
        Action::Innovation,
        Action::PreparatoryTouch,
        Action::MasterMend,
        Action::GreatStrides,
        Action::ByregotsBlessing,
    ];
    let states: Vec<(u16, u16)> = simulate(
        &settings,
        actions
            .into_iter()
            .zip(std::iter::repeat(Condition::Normal)),
    )
    .unwrap()
    .into_iter()
    .map(|state| progress_quality_pair(&settings, state))
    .collect();
    let expected = [
        (0, 900),
        (0, 900),
        (0, 1980),
        (0, 2610),
        (0, 2610),
        (0, 4860),
        (0, 4860),
        (0, 4860),
        (0, 7410),
        (0, 7410),
        (0, 7410),
        (0, 11400),
    ];
    assert_eq!(states, expected);
}
