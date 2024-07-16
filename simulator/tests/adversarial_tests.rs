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
        dbg!(state.get_missing_quality());
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
        dbg!(state.get_missing_quality());
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
        dbg!(state.get_missing_quality());
        assert_eq!(settings.max_quality - state.get_missing_quality(), 200 + 180 + 292 + 280);
    } else {
        panic!("Unexpected err: {}", state.err().unwrap());
    }
}