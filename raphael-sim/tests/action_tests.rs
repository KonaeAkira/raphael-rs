use raphael_sim::*;

const SETTINGS: Settings = Settings {
    max_cp: 250,
    max_durability: 60,
    max_progress: 2000,
    max_quality: 40000,
    base_progress: 100,
    base_quality: 100,
    job_level: 100,
    allowed_actions: ActionMask::all(),
    adversarial: false,
    backload_progress: false,
    stellar_steady_hand_charges: 0,
};

/// Returns the 4 primary stats of a state:
/// - Progress
/// - Quality
/// - Durability (used)
/// - CP (used)
fn primary_stats(state: &SimulationState, settings: &Settings) -> (u32, u32, u16, u16) {
    (
        state.progress,
        state.quality,
        SETTINGS.max_durability - state.durability,
        settings.max_cp - state.cp,
    )
}

#[test]
fn test_basic_synthesis() {
    // Low level, potency increase trait not unlocked
    let settings = Settings {
        job_level: 30,
        ..SETTINGS
    };
    let state = SimulationState::new(&settings)
        .use_action(Action::BasicSynthesis, Condition::Normal, &settings)
        .unwrap();
    assert_eq!(primary_stats(&state, &settings), (100, 0, 10, 0));
    // Potency-increase trait unlocked
    let settings = Settings {
        job_level: 31,
        ..SETTINGS
    };
    let state = SimulationState::new(&settings)
        .use_action(Action::BasicSynthesis, Condition::Normal, &settings)
        .unwrap();
    assert_eq!(primary_stats(&state, &settings), (120, 0, 10, 0));
}

#[test]
fn test_basic_touch() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::BasicTouch, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 100, 10, 18));
    assert_eq!(state.effects.inner_quiet(), 1);
    assert_eq!(state.effects.combo(), Combo::BasicTouch);
}

#[test]
fn test_master_mend() {
    // Durability-restore fully utilized
    let initial_state = SimulationState {
        durability: SETTINGS.max_durability - 40,
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::MasterMend, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 10, 88));
    // Durability-restore partially utilized
    let initial_state = SimulationState {
        durability: SETTINGS.max_durability - 10,
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::MasterMend, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 88));
}

#[test]
fn test_observe() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::Observe, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 7));
    assert_eq!(state.effects.combo(), Combo::StandardTouch);
}

#[test]
fn test_tricks_of_the_trade() {
    // Precondition not fulfilled
    let error = SimulationState::new(&SETTINGS)
        .use_action(Action::TricksOfTheTrade, Condition::Normal, &SETTINGS)
        .unwrap_err();
    assert_eq!(error, ActionError::SpecialConditionNotMet);
    // Can use when condition is Good or Excellent
    let initial_state = SimulationState {
        cp: SETTINGS.max_cp - 25, // test maximum restored CP
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::TricksOfTheTrade, Condition::Good, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 5));
    // Can use when Heart and Soul is active
    let initial_state = SimulationState {
        cp: SETTINGS.max_cp - 5, // test that restored CP is capped at max_cp
        effects: Effects::new().with_heart_and_soul_active(true),
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::TricksOfTheTrade, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 0));
    assert_eq!(state.effects.heart_and_soul_active(), false);
    // Heart and Soul effect isn't consumed when condition is Good or Excellent
    let initial_state = SimulationState {
        effects: Effects::new().with_heart_and_soul_active(true),
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::TricksOfTheTrade, Condition::Good, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 0));
    assert_eq!(state.effects.heart_and_soul_active(), true);
}

#[test]
fn test_waste_not() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::WasteNot, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 56));
    assert_eq!(state.effects.waste_not(), 4);
}

#[test]
fn test_veneration() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::Veneration, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 18));
    assert_eq!(state.effects.veneration(), 4);
}

#[test]
fn test_standard_touch() {
    // Combo requirement not fulfilled
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::StandardTouch, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 125, 10, 32));
    assert_eq!(state.effects.inner_quiet(), 1);
    assert_eq!(state.effects.combo(), Combo::None);
    // Combo requirement fulfilled
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_combo(Combo::BasicTouch);
    let state = initial_state
        .use_action(Action::StandardTouch, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 125, 10, 18));
    assert_eq!(state.effects.inner_quiet(), 1);
    assert_eq!(state.effects.combo(), Combo::StandardTouch);
}

#[test]
fn test_great_strides() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::GreatStrides, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 32));
    assert_eq!(state.effects.great_strides(), 3);
}

#[test]
fn test_innovation() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::Innovation, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 18));
    assert_eq!(state.effects.innovation(), 4);
}

#[test]
fn test_waste_not_2() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::WasteNot2, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 98));
    assert_eq!(state.effects.waste_not(), 8);
}

#[test]
fn test_byregots_blessing() {
    // Cannot use without inner quiet
    let error = SimulationState::new(&SETTINGS)
        .use_action(Action::ByregotsBlessing, Condition::Normal, &SETTINGS)
        .unwrap_err();
    assert_eq!(error, ActionError::SpecialConditionNotMet);
    // Quality efficiency scales with inner quiet
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_inner_quiet(5);
    let state = initial_state
        .use_action(Action::ByregotsBlessing, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 300, 10, 24));
    assert_eq!(state.effects.inner_quiet(), 0);
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_inner_quiet(10);
    let state = initial_state
        .use_action(Action::ByregotsBlessing, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 600, 10, 24));
    assert_eq!(state.effects.inner_quiet(), 0);
}

#[test]
fn test_precise_touch() {
    // Precondition not fulfilled
    let error = SimulationState::new(&SETTINGS)
        .use_action(Action::PreciseTouch, Condition::Normal, &SETTINGS)
        .unwrap_err();
    assert_eq!(error, ActionError::SpecialConditionNotMet);
    // Can use when condition is Good or Excellent
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::PreciseTouch, Condition::Good, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 225, 10, 18));
    assert_eq!(state.effects.inner_quiet(), 2);
    // Can use when Heart and Soul is active
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_heart_and_soul_active(true);
    let state = initial_state
        .use_action(Action::PreciseTouch, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 150, 10, 18));
    assert_eq!(state.effects.inner_quiet(), 2);
    assert_eq!(state.effects.heart_and_soul_active(), false);
    // Heart and Soul effect isn't consumed when condition is Good or Excellent
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_heart_and_soul_active(true);
    let state = initial_state
        .use_action(Action::PreciseTouch, Condition::Good, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 225, 10, 18));
    assert_eq!(state.effects.inner_quiet(), 2);
    assert_eq!(state.effects.heart_and_soul_active(), true);
}

#[test]
fn test_muscle_memory() {
    // Precondition unfulfilled
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_combo(Combo::None);
    let error = initial_state
        .use_action(Action::MuscleMemory, Condition::Normal, &SETTINGS)
        .unwrap_err();
    assert_eq!(error, ActionError::ComboRequirementNotMet);
    // Precondition fulfilled
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::MuscleMemory, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (300, 0, 10, 6));
    assert_eq!(state.effects.muscle_memory(), 5);
}

#[test]
fn test_careful_synthesis() {
    // Low level, potency-increase trait not unlocked
    let settings = Settings {
        job_level: 81,
        ..SETTINGS
    };
    let state = SimulationState::new(&settings)
        .use_action(Action::CarefulSynthesis, Condition::Normal, &settings)
        .unwrap();
    assert_eq!(primary_stats(&state, &settings), (150, 0, 10, 7));
    // Potency-increase trait unlocked
    let settings = Settings {
        job_level: 82,
        ..SETTINGS
    };
    let state = SimulationState::new(&settings)
        .use_action(Action::CarefulSynthesis, Condition::Normal, &settings)
        .unwrap();
    assert_eq!(primary_stats(&state, &settings), (180, 0, 10, 7));
}

#[test]
fn test_manipulation() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::Manipulation, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 0, 96));
    assert_eq!(state.effects.manipulation(), 8);
    // Using Manipulation while Manipulation is already active doesn't restore durability
    let initial_state = SimulationState {
        durability: SETTINGS.max_durability - 5,
        effects: Effects::new().with_manipulation(2),
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::Manipulation, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 0, 5, 96));
    assert_eq!(state.effects.manipulation(), 8);
}

#[test]
fn test_prudent_touch() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::PrudentTouch, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 100, 5, 25));
    assert_eq!(state.effects.inner_quiet(), 1);
    // Cannot use while Waste Not is active
    let initial_state = SimulationState {
        effects: Effects::new().with_waste_not(2),
        ..SimulationState::new(&SETTINGS)
    };
    let error = initial_state
        .use_action(Action::PrudentTouch, Condition::Normal, &SETTINGS)
        .unwrap_err();
    assert_eq!(error, ActionError::SpecialConditionNotMet);
}

#[test]
fn test_advanced_touch() {
    // Combo requirement unfulfilled
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::AdvancedTouch, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 150, 10, 46));
    assert_eq!(state.effects.inner_quiet(), 1);
    // Combo requirement fulfilled
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_combo(Combo::StandardTouch);
    let state = initial_state
        .use_action(Action::AdvancedTouch, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 150, 10, 18));
    assert_eq!(state.effects.inner_quiet(), 1);
}

#[test]
fn test_reflect() {
    // Precondition unfulfilled
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_combo(Combo::None);
    let error = initial_state
        .use_action(Action::Reflect, Condition::Normal, &SETTINGS)
        .unwrap_err();
    assert_eq!(error, ActionError::ComboRequirementNotMet);
    // Precondition fulfilled
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::Reflect, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 300, 10, 6));
    assert_eq!(state.effects.inner_quiet(), 2);
}

#[test]
fn test_preparatory_touch() {
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::PreparatoryTouch, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (0, 200, 20, 40));
    assert_eq!(state.effects.inner_quiet(), 2);
}

#[test]
fn test_groundwork() {
    // Low level, potency-increasing trait not unlocked
    let settings = Settings {
        job_level: 85,
        ..SETTINGS
    };
    let state = SimulationState::new(&settings)
        .use_action(Action::Groundwork, Condition::Normal, &settings)
        .unwrap();
    assert_eq!(primary_stats(&state, &settings), (300, 0, 20, 18));
    // Potency-increasing trait unlocked
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::Groundwork, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (360, 0, 20, 18));
    // Potency is halved when durability isn't enough
    let initial_state = SimulationState {
        durability: 10,
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::Groundwork, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(
        primary_stats(&state, &SETTINGS),
        (180, 0, SETTINGS.max_durability, 18)
    );
    // Potency isn't halved when Waste Not causes durability cost to fit into remaining durability
    let initial_state = SimulationState {
        durability: 10,
        effects: Effects::new().with_waste_not(1),
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::Groundwork, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(
        primary_stats(&state, &SETTINGS),
        (360, 0, SETTINGS.max_durability, 18)
    );
    // Potency isn't halved when Trained Perfection is active
    let initial_state = SimulationState {
        durability: 10,
        effects: Effects::new().with_trained_perfection_active(true),
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::Groundwork, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(
        primary_stats(&state, &SETTINGS),
        (360, 0, SETTINGS.max_durability - 10, 18)
    );
}

#[test]
fn test_delicate_synthesis() {
    // Low level, potency-increasing trait not unlocked
    let settings = Settings {
        job_level: 93,
        ..SETTINGS
    };
    let state = SimulationState::new(&settings)
        .use_action(Action::DelicateSynthesis, Condition::Normal, &settings)
        .unwrap();
    assert_eq!(primary_stats(&state, &settings), (100, 100, 10, 32));
    // Potency-increasing trait unlocked
    let settings = Settings {
        job_level: 94,
        ..SETTINGS
    };
    let state = SimulationState::new(&settings)
        .use_action(Action::DelicateSynthesis, Condition::Normal, &settings)
        .unwrap();
    assert_eq!(primary_stats(&state, &settings), (150, 100, 10, 32));
}

#[test]
fn test_intensive_synthesis() {
    // Precondition not fulfilled
    let error = SimulationState::new(&SETTINGS)
        .use_action(Action::IntensiveSynthesis, Condition::Normal, &SETTINGS)
        .unwrap_err();
    assert_eq!(error, ActionError::SpecialConditionNotMet);
    // Can use when condition is Good or Excellent
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::IntensiveSynthesis, Condition::Good, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (400, 0, 10, 6));
    // Can use when Heart and Soul is active
    let initial_state = SimulationState {
        effects: Effects::new().with_heart_and_soul_active(true),
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::IntensiveSynthesis, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (400, 0, 10, 6));
    assert_eq!(state.effects.heart_and_soul_active(), false);
    // Heart and Soul effect isn't consumed when condition is Good or Excellent
    let initial_state = SimulationState {
        effects: Effects::new().with_heart_and_soul_active(true),
        ..SimulationState::new(&SETTINGS)
    };
    let state = initial_state
        .use_action(Action::IntensiveSynthesis, Condition::Good, &SETTINGS)
        .unwrap();
    assert_eq!(primary_stats(&state, &SETTINGS), (400, 0, 10, 6));
    assert_eq!(state.effects.heart_and_soul_active(), true);
}

#[test]
fn test_trained_eye() {
    // Precondition unfulfilled
    let mut initial_state = SimulationState::new(&SETTINGS);
    initial_state.effects.set_combo(Combo::None);
    let error = initial_state
        .use_action(Action::TrainedEye, Condition::Normal, &SETTINGS)
        .unwrap_err();
    assert_eq!(error, ActionError::ComboRequirementNotMet);
    // Precondition fulfilled
    let state = SimulationState::new(&SETTINGS)
        .use_action(Action::TrainedEye, Condition::Normal, &SETTINGS)
        .unwrap();
    assert_eq!(
        primary_stats(&state, &SETTINGS),
        (0, u32::from(SETTINGS.max_quality), 10, 250)
    );
    assert_eq!(state.effects.inner_quiet(), 1);
}

#[test]
fn test_prudent_synthesis() {
    let state =
        SimulationState::from_macro(&SETTINGS, &[Action::WasteNot, Action::PrudentSynthesis]);
    assert_eq!(state, Err(ActionError::SpecialConditionNotMet));
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
    assert_eq!(state, Err(ActionError::SpecialConditionNotMet));
}

#[test]
fn test_refined_touch() {
    let state = SimulationState::from_macro(&SETTINGS, &[Action::BasicTouch, Action::RefinedTouch])
        .unwrap();
    assert_eq!(state.effects.inner_quiet(), 3);
    let state = SimulationState::from_macro(&SETTINGS, &[Action::RefinedTouch]);
    assert_eq!(state, Err(ActionError::ComboRequirementNotMet));
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
    )
    .unwrap();
    assert_eq!(state.durability, SETTINGS.max_durability);
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
    )
    .unwrap();
    assert_eq!(state.durability, SETTINGS.max_durability);
    let state = SimulationState::from_macro(
        &SETTINGS,
        &[Action::TrainedPerfection, Action::TrainedPerfection],
    );
    assert_eq!(state, Err(ActionError::NoRemainingUses));
}

#[test]
fn test_heart_and_soul() {
    let settings = Settings {
        adversarial: true,
        ..SETTINGS
    };
    let state = SimulationState::from_macro(
        &settings,
        &[
            Action::Manipulation,
            Action::BasicTouch,
            Action::HeartAndSoul,
        ],
    )
    .unwrap();
    assert_eq!(state.effects.combo(), Combo::None); // combo is removed
    assert_eq!(
        state.effects.special_quality_state(),
        SpecialQualityState::AdversarialGuard
    ); // condition is not re-rolled
    assert_eq!(state.effects.manipulation(), 7); // effects are not ticked
    assert_eq!(state.effects.heart_and_soul_available(), false);
    assert_eq!(state.effects.heart_and_soul_active(), true);
    let state = SimulationState::from_macro(
        &settings,
        &[
            Action::HeartAndSoul,
            Action::PrudentSynthesis,
            Action::MasterMend,
        ],
    )
    .unwrap();
    assert_eq!(state.effects.heart_and_soul_active(), true); // effect stays active until used
    let state = SimulationState::from_macro(
        &settings,
        &[Action::HeartAndSoul, Action::IntensiveSynthesis],
    )
    .unwrap();
    assert_eq!(state.effects.heart_and_soul_active(), false); // effect is used up
    let state =
        SimulationState::from_macro(&settings, &[Action::HeartAndSoul, Action::PreciseTouch])
            .unwrap();
    assert_eq!(state.effects.heart_and_soul_active(), false); // effect is used up
    let state = SimulationState::from_macro(
        &settings,
        &[
            Action::HeartAndSoul,
            Action::BasicTouch,
            Action::HeartAndSoul,
        ],
    );
    assert_eq!(state, Err(ActionError::NoRemainingUses));
}

#[test]
fn test_quick_innovation() {
    let setings = Settings {
        adversarial: true,
        ..SETTINGS
    };
    let state = SimulationState::from_macro(
        &setings,
        &[
            Action::Manipulation,
            Action::BasicTouch,
            Action::QuickInnovation,
        ],
    )
    .unwrap();
    assert_eq!(state.effects.combo(), Combo::None); // combo is removed
    assert_eq!(
        state.effects.special_quality_state(),
        SpecialQualityState::AdversarialGuard
    ); // condition is not re-rolled
    assert_eq!(state.effects.manipulation(), 7); // effects are not ticked
    assert_eq!(state.effects.innovation(), 1);
    let state = SimulationState::from_macro(
        &setings,
        &[
            Action::QuickInnovation,
            Action::BasicTouch,
            Action::QuickInnovation,
        ],
    );
    assert_eq!(state, Err(ActionError::NoRemainingUses));
    let state =
        SimulationState::from_macro(&setings, &[Action::Innovation, Action::QuickInnovation]);
    assert_eq!(state, Err(ActionError::SpecialConditionNotMet));
}

#[test]
fn test_stellar_steady_hand() {
    let settings = Settings {
        stellar_steady_hand_charges: 2,
        ..SETTINGS
    };
    // Test that using Stellar Steady Hand gives the correct effect duration (3)
    // and decreases the number of remaining charges.
    let state = SimulationState::from_macro(&settings, &[Action::StellarSteadyHand]).unwrap();
    assert_eq!(state.effects.stellar_steady_hand(), 3);
    assert_eq!(state.effects.stellar_steady_hand_charges(), 1);
    // Test that Stellar Steady Hand resets the effect duration if the effect is active.
    let state = state
        .use_action(Action::StellarSteadyHand, Condition::Normal, &settings)
        .unwrap();
    assert_eq!(state.effects.stellar_steady_hand(), 3);
    assert_eq!(state.effects.stellar_steady_hand_charges(), 0);
    // Test that Stellar Steady Hand cannot be used if there are no remaining charges.
    let state = state.use_action(Action::StellarSteadyHand, Condition::Normal, &settings);
    assert_eq!(Err(ActionError::NoRemainingUses), state);
}

#[test]
fn test_rapid_synthesis() {
    let settings = Settings {
        stellar_steady_hand_charges: 1,
        ..SETTINGS
    };
    // Test that Rapid Synthesis can be used while Stellar Steady Hand is active.
    // Rapid Synthesis has a lower progress efficiency below level 63, but there is currently
    // no scenario to use the action below level 63 so the lower efficiency is not tested.
    let state = SimulationState::from_macro(
        &settings,
        &[Action::StellarSteadyHand, Action::RapidSynthesis],
    )
    .unwrap();
    assert_eq!(state.progress, 500);
    assert_eq!(state.durability, settings.max_durability - 10);
    assert_eq!(state.cp, settings.max_cp);
    // Test that Rapid Synthesis cannot be used if Stellar Steady Hand is not active.
    let state = SimulationState::from_macro(&settings, &[Action::RapidSynthesis]);
    assert_eq!(Err(ActionError::UnreliableAction), state);
}

#[test]
fn test_hasty_touch() {
    let settings = Settings {
        stellar_steady_hand_charges: 1,
        ..SETTINGS
    };
    // Test that Hasty Touch can be used while Stellar Steady Hand is active.
    let state =
        SimulationState::from_macro(&settings, &[Action::StellarSteadyHand, Action::HastyTouch])
            .unwrap();
    assert_eq!(state.quality, 100);
    assert_eq!(state.durability, settings.max_durability - 10);
    assert_eq!(state.cp, settings.max_cp);
    assert_eq!(state.effects.expedience(), true);
    // Test that Hasty Touch cannot be used if Stellar Steady Hand is not active.
    let state = SimulationState::from_macro(&settings, &[Action::HastyTouch]);
    assert_eq!(Err(ActionError::UnreliableAction), state);
}

#[test]
fn test_daring_touch() {
    let settings = Settings {
        stellar_steady_hand_charges: 1,
        ..SETTINGS
    };
    // Test that Daring Touch can be used while Stellar Steady Hand and Expedience are active.
    let state = SimulationState::from_macro(
        &settings,
        &[
            Action::StellarSteadyHand,
            Action::HastyTouch,
            Action::DaringTouch,
        ],
    )
    .unwrap();
    assert_eq!(state.quality, 265);
    assert_eq!(state.durability, settings.max_durability - 20);
    assert_eq!(state.cp, settings.max_cp);
    assert_eq!(state.effects.expedience(), false);
    // Test that Hasty Touch cannot be used if Stellar Steady Hand is not active.
    let state = SimulationState::from_macro(
        &settings,
        &[
            Action::StellarSteadyHand,
            Action::Observe,
            Action::Observe,
            Action::HastyTouch,
            Action::DaringTouch,
        ],
    );
    assert_eq!(Err(ActionError::UnreliableAction), state);
    // Test that Daring touch cannot be used if Expedience is not active.
    let state =
        SimulationState::from_macro(&settings, &[Action::StellarSteadyHand, Action::DaringTouch]);
    assert_eq!(Err(ActionError::SpecialConditionNotMet), state);
}
