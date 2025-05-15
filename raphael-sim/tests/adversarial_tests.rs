use raphael_sim::{Action, ActionMask, Condition, Settings, SimulationState};
const SETTINGS: Settings = Settings {
    max_cp: 1000,
    max_durability: 80,
    max_progress: 2000,
    max_quality: 40000,
    base_progress: 100,
    base_quality: 100,
    job_level: 100,
    allowed_actions: ActionMask::all(),
    adversarial: true,
    backload_progress: false,
};

/// Calculate the minimum achievable Quality across all possible Condition rolls
fn guaranteed_quality(mut settings: Settings, actions: &[Action]) -> Result<u32, &'static str> {
    let is_valid_mask = |mut mask: i32| {
        // a 1-bit denotes an Excellent proc
        if (mask & 1) != 0 {
            // first step cannot be Excellent
            return false;
        }
        while mask != 0 {
            if (mask & 0b111).count_ones() > 1 {
                // Excellent procs must be at least 3 steps apart
                // due to the forced Excellent > Poor > Normal condition chain
                return false;
            }
            mask >>= 1;
        }
        true
    };

    settings.adversarial = false;
    let mut min_quality = u32::MAX;

    for mask in 0..(1 << actions.len()) {
        if !is_valid_mask(mask) {
            continue;
        }
        let mut state = SimulationState::new(&settings);
        for (index, action) in actions.iter().enumerate() {
            let condition = if ((mask >> index) & 1) == 1 {
                Condition::Excellent
            } else if index == 0 || ((mask >> (index - 1)) & 1) == 0 {
                Condition::Normal
            } else {
                Condition::Poor
            };
            state = state.use_action(*action, condition, &settings)?;
        }
        min_quality = std::cmp::min(min_quality, state.quality);
    }
    Ok(min_quality)
}

#[test]
fn test_simple() {
    let actions = [
        Action::Observe,
        Action::Observe,
        Action::PreparatoryTouch,
        Action::BasicSynthesis,
    ];
    let state = SimulationState::from_macro(&SETTINGS, &actions);
    if let Ok(state) = state {
        assert_eq!(guaranteed_quality(SETTINGS, &actions).unwrap(), 100);
        assert_eq!(state.quality, 100);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_short_quality_opener() {
    let actions = [Action::Reflect];
    let state = SimulationState::from_macro(&SETTINGS, &actions);
    if let Ok(state) = state {
        assert_eq!(guaranteed_quality(SETTINGS, &actions).unwrap(), 300);
        assert_eq!(state.quality, 300);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_long_quality_opener() {
    let actions = [
        Action::Reflect,
        Action::PreparatoryTouch,
        Action::PreparatoryTouch,
        Action::PreparatoryTouch,
    ];
    let state = SimulationState::from_macro(&SETTINGS, &actions);
    if let Ok(state) = state {
        assert_eq!(guaranteed_quality(SETTINGS, &actions).unwrap(), 1140);
        assert_eq!(state.quality, 1140);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_alternating_quality_actions() {
    let actions = [
        Action::MuscleMemory,
        Action::GreatStrides,
        Action::BasicTouch,
        Action::GreatStrides,
        Action::BasicTouch,
        Action::GreatStrides,
        Action::BasicTouch,
    ];
    let state = SimulationState::from_macro(&SETTINGS, &actions);
    if let Ok(state) = state {
        assert_eq!(guaranteed_quality(SETTINGS, &actions).unwrap(), 440);
        assert_eq!(state.quality, 440);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_double_status_drops() {
    let actions = [
        Action::MuscleMemory,
        Action::GreatStrides,
        Action::BasicTouch,
        Action::Innovation,
        Action::GreatStrides,
        Action::BasicTouch,
        Action::GreatStrides,
        Action::BasicTouch,
    ];
    let state = SimulationState::from_macro(&SETTINGS, &actions);
    if let Ok(state) = state {
        assert_eq!(guaranteed_quality(SETTINGS, &actions).unwrap(), 525);
        assert_eq!(state.quality, 525);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_two_action_drops() {
    let actions = [
        Action::MuscleMemory,
        Action::GreatStrides,
        Action::BasicTouch,
        Action::StandardTouch,
        Action::GreatStrides,
        Action::BasicTouch,
        Action::GreatStrides,
        Action::BasicTouch,
    ];
    let state = SimulationState::from_macro(&SETTINGS, &actions);
    if let Ok(state) = state {
        assert_eq!(guaranteed_quality(SETTINGS, &actions).unwrap(), 607);
        assert_eq!(state.quality, 607);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_dp() {
    let actions = [
        Action::MuscleMemory,
        Action::GreatStrides,
        Action::PreparatoryTouch,
        Action::Innovation,
        Action::BasicTouch,
        Action::Observe,
        Action::AdvancedTouch,
        Action::GreatStrides,
        Action::PreparatoryTouch,
    ];
    let state = SimulationState::from_macro(&SETTINGS, &actions);
    if let Ok(state) = state {
        assert_eq!(guaranteed_quality(SETTINGS, &actions).unwrap(), 952);
        assert_eq!(state.quality, 952);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_long_sequence() {
    let actions = [
        Action::Reflect,
        Action::Manipulation,
        Action::Innovation,
        Action::WasteNot2,
        Action::BasicTouch,
        Action::StandardTouch,
        Action::PreparatoryTouch,
        Action::Veneration,
        Action::DelicateSynthesis,
        Action::Groundwork,
        Action::Groundwork,
        Action::Groundwork,
        Action::Innovation,
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
        Action::ByregotsBlessing,
        Action::CarefulSynthesis,
    ];
    let state = SimulationState::from_macro(&SETTINGS, &actions);
    if let Ok(state) = state {
        assert_eq!(guaranteed_quality(SETTINGS, &actions).unwrap(), 2924);
        assert_eq!(state.quality, 2924);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
/// Test random all possible sequences of Observe and PrudentTouch of a certain length
fn test_exhaustive() {
    const STEPS: usize = 10;
    for mask in 0..(1 << STEPS) {
        let actions: Vec<Action> = (0..STEPS)
            .map(|index| match (mask >> index) & 1 {
                0 => Action::Observe,
                _ => Action::PrudentTouch,
            })
            .collect();
        let state = SimulationState::from_macro(&SETTINGS, &actions);
        if let Ok(state) = state {
            dbg!(&actions);
            assert_eq!(
                state.quality,
                guaranteed_quality(SETTINGS, &actions).unwrap()
            );
        } else {
            panic!("Unexpected err: {}", state.err().unwrap());
        }
    }
}

#[test]
/// Test random quality action sequences
fn test_fuzz() {
    const STEPS: usize = 10;
    const ACTIONS: [Action; 9] = [
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
        Action::ByregotsBlessing,
        Action::Observe,
        Action::Innovation,
        Action::GreatStrides,
        Action::ImmaculateMend,
        Action::TricksOfTheTrade,
    ];
    for _ in 0..100000 {
        let actions: Vec<Action> =
            std::iter::repeat_with(|| ACTIONS[rand::random_range(0..ACTIONS.len())])
                .take(STEPS)
                .collect();
        if let Ok(state) = SimulationState::from_macro(&SETTINGS, &actions) {
            dbg!(&actions);
            assert_eq!(
                state.quality,
                guaranteed_quality(SETTINGS, &actions).unwrap()
            );
        }
    }
}
