use simulator::{Action, ActionMask, Settings, SimulationState};

const SETTINGS: Settings = Settings {
    max_cp: 250,
    max_durability: 60,
    max_progress: 2000,
    max_quality: 40000,
    base_progress: 100,
    base_quality: 100,
    initial_quality: 0,
    job_level: 100,
    allowed_actions: ActionMask::all(),
    adversarial: false,
};

#[test]
fn test_redundant_half_efficiency_groundwork() {
    // Groundwork's efficiency is halved when the remaining durability is lower than the durability cost
    // Careful Synthesis is a strictly better action in case Groundwork's efficiency is halved
    // This enables the simulator to simplify things by simply not allowing Groundwork when there isn't enough durability
    for level in 1..=100 {
        let settings = Settings {
            job_level: level,
            allowed_actions: ActionMask::from_level(level as _, true, false),
            ..SETTINGS
        };
        match (
            settings.allowed_actions.has(Action::CarefulSynthesis),
            settings.allowed_actions.has(Action::Groundwork),
        ) {
            (false, true) => panic!("Cannot replace half-efficiency Groundwork because CarefulSynthesis is not available at level {}", level),
            (true, true) => {
                let state_1 = SimulationState::from_macro(&settings, &[Action::CarefulSynthesis]).unwrap();
                let state_2 = SimulationState::from_macro(&settings, &[Action::Groundwork]).unwrap();
                let progress_1 = settings.max_progress - state_1.missing_progress;
                let progress_2 = settings.max_progress - state_2.missing_progress;
                assert!(progress_1 * 2 >= progress_2);
                assert!(state_1.durability >= state_2.durability);
                assert!(state_1.cp >= state_2.cp);
            }
            _ => ()
        }
    }
}

#[test]
fn test_standard_touch_combo() {
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::ComboStandardTouch]);
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::ComboStandardTouch]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_advanced_touch_combo() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::BasicTouch,
            Action::ComboStandardTouch,
            Action::ComboAdvancedTouch,
        ],
    );
    assert!(matches!(state, Ok(_)));
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::Observe, Action::ComboAdvancedTouch]);
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::ComboAdvancedTouch]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_reflect_opener() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::Reflect]);
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::Reflect]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_muscle_memory_opener() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::MuscleMemory]);
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::MuscleMemory]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_trained_eye_opener() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::TrainedEye]);
    assert!(matches!(state, Ok(_)));
    let state = state.unwrap();
    assert_eq!(state.get_missing_quality(), 0);
    assert_eq!(state.effects.inner_quiet(), 1);
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::BasicSynthesis, Action::TrainedEye]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_manipulation() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::BasicSynthesis,
            Action::Manipulation,
            Action::Manipulation,
        ],
    )
    .unwrap();
    assert_eq!(state.durability, SETTINGS.max_durability - 10);
}

#[test]
fn test_prudent_touch() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::WasteNot, Action::PrudentTouch]);
    assert!(matches!(
        state,
        Err("Action cannot be used during Waste Not")
    ));
}

#[test]
fn test_groundwork() {
    let settings = Settings {
        job_level: 100,
        max_durability: 10,
        ..SETTINGS
    };
    let state = SimulationState::from_macro(&settings, &[Action::Groundwork]);
    assert!(matches!(state, Err("Not enough durability")));
    let state =
        SimulationState::from_macro(&settings, &[Action::TrainedPerfection, Action::Groundwork]);
    match state {
        Ok(state) => {
            assert_eq!(settings.max_progress - state.missing_progress, 360);
            assert_eq!(state.durability, 10);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_prudent_synthesis() {
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::WasteNot, Action::PrudentSynthesis]);
    assert!(matches!(
        state,
        Err("Action cannot be used during Waste Not")
    ));
}

#[test]
fn test_trained_finesse() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::PreparatoryTouch,
            Action::PreparatoryTouch,
            Action::TrainedFinesse,
        ],
    );
    assert!(matches!(state, Err("Requires 10 Inner Quiet")));
}

#[test]
fn test_refined_touch() {
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::ComboRefinedTouch]);
    match state {
        Ok(state) => {
            assert_eq!(state.effects.inner_quiet(), 3);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
    assert!(matches!(state, Ok(_)));
    let state = SimulationState::from_macro(&SETTINGS, &[Action::ComboRefinedTouch]);
    assert!(matches!(state, Err("Combo requirement not fulfilled")));
}

#[test]
fn test_immaculate_mend() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::BasicTouch,
            Action::Groundwork,
            Action::Groundwork,
            Action::ImmaculateMend,
        ],
    );
    match state {
        Ok(state) => {
            assert_eq!(state.durability, SETTINGS.max_durability);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_trained_perfection() {
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[
            Action::TrainedPerfection,
            Action::Observe, // 0-durability actions don't proc Trained Perfection
            Action::Groundwork,
        ],
    );
    match state {
        Ok(state) => {
            assert_eq!(state.durability, SETTINGS.max_durability);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    };
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[Action::TrainedPerfection, Action::TrainedPerfection],
    );
    assert!(matches!(
        state,
        Err("Action can only be used once per synthesis")
    ));
}

#[test]
fn test_delicate_synthesis() {
    let settings = Settings {
        job_level: 93,
        ..SETTINGS
    };
    let state = SimulationState::from_macro(&settings, &[Action::DelicateSynthesis]);
    match state {
        Ok(state) => {
            assert_eq!(settings.max_progress - state.missing_progress, 100);
            assert_eq!(settings.max_quality - state.get_missing_quality(), 100);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
    let settings = Settings {
        job_level: 94,
        ..SETTINGS
    };
    let state = SimulationState::from_macro(&settings, &[Action::DelicateSynthesis]);
    match state {
        Ok(state) => {
            assert_eq!(settings.max_progress - state.missing_progress, 150);
            assert_eq!(settings.max_quality - state.get_missing_quality(), 100);
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}

#[test]
fn test_adversarial_calculation() {
    let settings = Settings {
        adversarial: true,
        ..SETTINGS
    };
    let state =
        SimulationState::from_macro(&settings, &[Action::Observe, Action::Observe, Action::PreparatoryTouch, Action::BasicSynthesis]);
    if let Ok(state) = state {
        println!("{}", state.get_missing_quality());
        assert_eq!(settings.max_quality - state.get_missing_quality(), 100);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
    
}

#[test]
fn test_flipping() {
    let settings = Settings {
        adversarial: true,
        ..SETTINGS
    };
    let state =
        SimulationState::from_macro(&settings, &[
            Action::MuscleMemory, 
            Action::GreatStrides, 
            Action::BasicTouch, 
            Action::GreatStrides, 
            Action::BasicTouch, 
            Action::GreatStrides, 
            Action::BasicTouch
        ]);
    if let Ok(state) = state {
        println!("{}", state.get_missing_quality());
        assert_eq!(settings.max_quality - state.get_missing_quality(), 100 + 220 + 120);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_double_status_drops_unreliable() {
    let settings = Settings {
        adversarial: true,
        ..SETTINGS
    };
    let state =
        SimulationState::from_macro(&settings, &[
            Action::MuscleMemory, 
            Action::GreatStrides, 
            Action::BasicTouch, 
            Action::Innovation,
            Action::GreatStrides, 
            Action::BasicTouch, 
            Action::GreatStrides, 
            Action::BasicTouch
        ]);
    if let Ok(state) = state {
        println!("{}", state.get_missing_quality());
        assert_eq!(settings.max_quality - state.get_missing_quality(), 100 + 275 + 150);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_two_actions_drop_unreliable() {
    let settings = Settings {
        adversarial: true,
        ..SETTINGS
    };
    let state =
        SimulationState::from_macro(&settings, &[
            Action::MuscleMemory, 
            Action::GreatStrides, 
            Action::BasicTouch, 
            Action::StandardTouch,
            Action::GreatStrides, 
            Action::BasicTouch, 
            Action::GreatStrides, 
            Action::BasicTouch
        ]);
    if let Ok(state) = state {
        println!("{}", state.get_missing_quality());
        assert_eq!(settings.max_quality - state.get_missing_quality(), 100 + 137 + 240 + 130);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}

#[test]
fn test_unreliable_dp() {
    let settings = Settings {
        adversarial: true,
        max_durability: 80,
        max_cp: 1000,
        ..SETTINGS
    };
    let state =
        SimulationState::from_macro(&settings, &[
            Action::MuscleMemory, 
            Action::GreatStrides, 
            Action::PreparatoryTouch,
            Action::Innovation,
            Action::BasicTouch,
            Action::Observe,
            Action::AdvancedTouch,
            Action::GreatStrides,
            Action::PreparatoryTouch
        ]);
    if let Ok(state) = state {
        println!("{}", state.get_missing_quality());
        assert_eq!(settings.max_quality - state.get_missing_quality(), 200 + 180 + 292 + 280);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}