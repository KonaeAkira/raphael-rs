use crate::{ActionMask, Condition, Effects, Settings, SimulationState};

const DEFAULT_EFFECT_RESET_MASK: Effects = {
    assert!(Combo::None.into_bits() == 0);
    Effects::from_bits(u64::MAX).with_combo(Combo::None)
};

pub trait ActionImpl {
    const LEVEL_REQUIREMENT: u8;
    /// All bits of this mask must be present in the settings' action mask for the action to be enabled.
    const ACTION_MASK: ActionMask;
    /// Does this action increase the step count when used?
    const INCREASES_STEP_COUNT: bool = true;

    const EFFECT_RESET_MASK: Effects;
    const EFFECT_SET_MASK: Effects;

    fn precondition(
        _state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        Ok(())
    }

    #[inline]
    fn progress_increase(state: &SimulationState, settings: &Settings) -> u32 {
        let action_mod = Self::progress_modifier(state, settings);
        let effect_mod = state.effects.progress_modifier();
        u32::from(settings.base_progress) * action_mod * effect_mod / 1000
    }

    #[inline]
    fn quality_increase(state: &SimulationState, settings: &Settings, condition: Condition) -> u32 {
        let action_mod = Self::quality_modifier(state, settings);
        let effect_mod = state.effects.quality_modifier();
        let condition_mod = match condition {
            Condition::Normal => 2,
            Condition::Good => 3,
            Condition::Excellent => 8,
            Condition::Poor => 1,
        };
        u32::from(settings.base_quality) * action_mod * effect_mod * condition_mod / 20000
    }

    fn durability_cost(state: &SimulationState, settings: &Settings, _condition: Condition) -> u16 {
        if state.effects.trained_perfection_active() {
            return 0;
        }
        match state.effects.waste_not() {
            0 => Self::base_durability_cost(state, settings),
            _ => Self::base_durability_cost(state, settings).div_ceil(2),
        }
    }

    fn cp_cost(state: &SimulationState, settings: &Settings, _condition: Condition) -> u16 {
        Self::base_cp_cost(state, settings)
    }

    fn progress_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        0
    }
    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        0
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        0
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        0
    }

    fn transform(_state: &mut SimulationState, _settings: &Settings, _condition: Condition) {}
}

pub struct BasicSynthesis {}
impl ActionImpl for BasicSynthesis {
    const LEVEL_REQUIREMENT: u8 = 1;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::BasicSynthesis);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_muscle_memory(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn progress_modifier(_state: &SimulationState, settings: &Settings) -> u32 {
        if settings.job_level < 31 { 100 } else { 120 }
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }
}

pub struct BasicTouch {}
impl BasicTouch {
    pub const CP_COST: u16 = 18;
}
impl ActionImpl for BasicTouch {
    const LEVEL_REQUIREMENT: u8 = 5;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::BasicTouch);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new().with_combo(Combo::BasicTouch);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        100
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
}

pub struct MasterMend {}
impl MasterMend {
    pub const CP_COST: u16 = 88;
}
impl ActionImpl for MasterMend {
    const LEVEL_REQUIREMENT: u8 = 7;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::MasterMend);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK;
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }

    fn transform(state: &mut SimulationState, settings: &Settings, _condition: Condition) {
        state.durability = std::cmp::min(settings.max_durability, state.durability + 30);
    }
}

pub struct Observe {}
impl Observe {
    pub const CP_COST: u16 = 7;
}
impl ActionImpl for Observe {
    const LEVEL_REQUIREMENT: u8 = 13;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Observe);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK;
    const EFFECT_SET_MASK: Effects = Effects::new().with_combo(Combo::StandardTouch);

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
}

pub struct TricksOfTheTrade {}
impl ActionImpl for TricksOfTheTrade {
    const LEVEL_REQUIREMENT: u8 = 13;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::TricksOfTheTrade);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK;
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.heart_and_soul_active()
            && condition != Condition::Good
            && condition != Condition::Excellent
        {
            return Err(ActionError::SpecialConditionNotMet);
        }
        Ok(())
    }

    fn transform(state: &mut SimulationState, settings: &Settings, condition: Condition) {
        state.cp = std::cmp::min(settings.max_cp, state.cp + 20);
        if condition != Condition::Good && condition != Condition::Excellent {
            state.effects.set_heart_and_soul_active(false);
        }
    }
}

pub struct WasteNot {}
impl WasteNot {
    pub const CP_COST: u16 = 56;
}
impl ActionImpl for WasteNot {
    const LEVEL_REQUIREMENT: u8 = 15;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::WasteNot);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK.with_waste_not(0);
    const EFFECT_SET_MASK: Effects = Effects::new().with_waste_not(4);

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
}

pub struct Veneration {}
impl Veneration {
    pub const CP_COST: u16 = 18;
}
impl ActionImpl for Veneration {
    const LEVEL_REQUIREMENT: u8 = 15;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Veneration);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK.with_veneration(0);
    const EFFECT_SET_MASK: Effects = Effects::new().with_veneration(4);

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
}

pub struct StandardTouch {}
impl ActionImpl for StandardTouch {
    const LEVEL_REQUIREMENT: u8 = 18;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::StandardTouch);

    const EFFECT_RESET_MASK: Effects = Effects::from_bits(u64::MAX)
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        125
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(state: &SimulationState, _settings: &Settings) -> u16 {
        match state.effects.combo() {
            Combo::BasicTouch => 18,
            _ => 32,
        }
    }

    fn transform(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        if state.effects.combo() == Combo::BasicTouch {
            state.effects.set_combo(Combo::StandardTouch);
        } else {
            state.effects.set_combo(Combo::None);
        }
    }
}

pub struct GreatStrides {}
impl GreatStrides {
    pub const CP_COST: u16 = 32;
}
impl ActionImpl for GreatStrides {
    const LEVEL_REQUIREMENT: u8 = 21;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::GreatStrides);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK.with_great_strides(0);
    const EFFECT_SET_MASK: Effects = Effects::new().with_great_strides(3);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
}

pub struct Innovation {}
impl Innovation {
    pub const CP_COST: u16 = 18;
}
impl ActionImpl for Innovation {
    const LEVEL_REQUIREMENT: u8 = 26;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Innovation);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK.with_innovation(0);
    const EFFECT_SET_MASK: Effects = Effects::new().with_innovation(4);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
}

pub struct WasteNot2 {}
impl WasteNot2 {
    pub const CP_COST: u16 = 98;
}
impl ActionImpl for WasteNot2 {
    const LEVEL_REQUIREMENT: u8 = 47;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::WasteNot2);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK.with_waste_not(0);
    const EFFECT_SET_MASK: Effects = Effects::new().with_waste_not(8);

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
}

pub struct ByregotsBlessing {}
impl ActionImpl for ByregotsBlessing {
    const LEVEL_REQUIREMENT: u8 = 50;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::ByregotsBlessing);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_inner_quiet(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.inner_quiet() == 0 {
            Err(ActionError::SpecialConditionNotMet)
        } else if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(state: &SimulationState, _settings: &Settings) -> u32 {
        100 + 20 * u32::from(state.effects.inner_quiet())
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        24
    }
}

pub struct PreciseTouch {}
impl ActionImpl for PreciseTouch {
    const LEVEL_REQUIREMENT: u8 = 53;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::PreciseTouch);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.heart_and_soul_active()
            && condition != Condition::Good
            && condition != Condition::Excellent
        {
            Err(ActionError::SpecialConditionNotMet)
        } else if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        150
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        18
    }

    fn transform(state: &mut SimulationState, _settings: &Settings, condition: Condition) {
        let iq = state.effects.inner_quiet();
        state.effects.set_inner_quiet(std::cmp::min(10, iq + 1));
        if condition != Condition::Good && condition != Condition::Excellent {
            state.effects.set_heart_and_soul_active(false);
        }
    }
}

pub struct MuscleMemory {}
impl ActionImpl for MuscleMemory {
    const LEVEL_REQUIREMENT: u8 = 54;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::MuscleMemory);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_muscle_memory(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new().with_muscle_memory(5);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.combo() != Combo::SynthesisBegin {
            return Err(ActionError::ComboRequirementNotMet);
        }
        Ok(())
    }

    fn progress_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        300
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        6
    }
}

pub struct CarefulSynthesis {}
impl ActionImpl for CarefulSynthesis {
    const LEVEL_REQUIREMENT: u8 = 62;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::CarefulSynthesis);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_muscle_memory(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn progress_modifier(_state: &SimulationState, settings: &Settings) -> u32 {
        match settings.job_level {
            0..82 => 150,
            82.. => 180,
        }
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        7
    }
}

pub struct Manipulation {}
impl Manipulation {
    pub const CP_COST: u16 = 96;
}
impl ActionImpl for Manipulation {
    const LEVEL_REQUIREMENT: u8 = 65;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Manipulation);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK.with_manipulation(0);
    const EFFECT_SET_MASK: Effects = Effects::new().with_manipulation(8);

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
}

pub struct PrudentTouch {}
impl ActionImpl for PrudentTouch {
    const LEVEL_REQUIREMENT: u8 = 66;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::PrudentTouch);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.waste_not() != 0 {
            Err(ActionError::SpecialConditionNotMet)
        } else if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        100
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        5
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        25
    }
}

pub struct AdvancedTouch {}
impl ActionImpl for AdvancedTouch {
    const LEVEL_REQUIREMENT: u8 = 68;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::AdvancedTouch);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        150
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(state: &SimulationState, _settings: &Settings) -> u16 {
        match state.effects.combo() {
            Combo::StandardTouch => 18,
            _ => 46,
        }
    }
}

pub struct Reflect {}
impl ActionImpl for Reflect {
    const LEVEL_REQUIREMENT: u8 = 69;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Reflect);

    const EFFECT_RESET_MASK: Effects =
        DEFAULT_EFFECT_RESET_MASK.with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.combo() != Combo::SynthesisBegin {
            Err(ActionError::ComboRequirementNotMet)
        } else if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        300
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        6
    }

    fn transform(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        let iq = state.effects.inner_quiet();
        state.effects.set_inner_quiet(std::cmp::min(10, iq + 1));
    }
}

pub struct PreparatoryTouch {}
impl PreparatoryTouch {
    pub const CP_COST: u16 = 40;
}
impl ActionImpl for PreparatoryTouch {
    const LEVEL_REQUIREMENT: u8 = 71;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::PreparatoryTouch);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        200
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        20
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }

    fn transform(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        let iq = state.effects.inner_quiet();
        state.effects.set_inner_quiet(std::cmp::min(10, iq + 1));
    }
}

pub struct Groundwork {}
impl ActionImpl for Groundwork {
    const LEVEL_REQUIREMENT: u8 = 72;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Groundwork);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_muscle_memory(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn progress_modifier(state: &SimulationState, settings: &Settings) -> u32 {
        let base = match settings.job_level {
            0..86 => 300,
            86.. => 360,
        };
        if Self::durability_cost(state, settings, Condition::Normal) > state.durability {
            return base / 2;
        }
        base
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        20
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        18
    }
}

pub struct DelicateSynthesis {}
impl ActionImpl for DelicateSynthesis {
    const LEVEL_REQUIREMENT: u8 = 76;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::DelicateSynthesis);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_muscle_memory(0)
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn progress_modifier(_state: &SimulationState, settings: &Settings) -> u32 {
        match settings.job_level {
            0..94 => 100,
            94.. => 150,
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        100
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        32
    }
}

pub struct IntensiveSynthesis {}
impl ActionImpl for IntensiveSynthesis {
    const LEVEL_REQUIREMENT: u8 = 78;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::IntensiveSynthesis);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_muscle_memory(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.heart_and_soul_active()
            && condition != Condition::Good
            && condition != Condition::Excellent
        {
            return Err(ActionError::SpecialConditionNotMet);
        }
        Ok(())
    }

    fn progress_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        400
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        6
    }

    fn transform(state: &mut SimulationState, _settings: &Settings, condition: Condition) {
        if condition != Condition::Good && condition != Condition::Excellent {
            state.effects.set_heart_and_soul_active(false);
        }
    }
}

pub struct TrainedEye {}
impl ActionImpl for TrainedEye {
    const LEVEL_REQUIREMENT: u8 = 80;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::TrainedEye);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.combo() != Combo::SynthesisBegin {
            Err(ActionError::ComboRequirementNotMet)
        } else if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_increase(
        _state: &SimulationState,
        settings: &Settings,
        _condition: Condition,
    ) -> u32 {
        u32::from(settings.max_quality)
    }

    fn quality_modifier(_state: &SimulationState, settings: &Settings) -> u32 {
        u32::from(settings.max_quality)
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        250
    }
}

pub struct HeartAndSoul {}
impl ActionImpl for HeartAndSoul {
    const LEVEL_REQUIREMENT: u8 = 86;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::HeartAndSoul);
    const INCREASES_STEP_COUNT: bool = false;

    const EFFECT_RESET_MASK: Effects =
        DEFAULT_EFFECT_RESET_MASK.with_heart_and_soul_available(false);
    const EFFECT_SET_MASK: Effects = Effects::new().with_heart_and_soul_active(true);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.heart_and_soul_available() {
            return Err(ActionError::NoRemainingUses);
        }
        Ok(())
    }
}

pub struct PrudentSynthesis {}
impl ActionImpl for PrudentSynthesis {
    const LEVEL_REQUIREMENT: u8 = 88;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::PrudentSynthesis);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_muscle_memory(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.waste_not() != 0 {
            return Err(ActionError::SpecialConditionNotMet);
        }
        Ok(())
    }

    fn progress_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        180
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        5
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        18
    }
}

pub struct TrainedFinesse {}
impl ActionImpl for TrainedFinesse {
    const LEVEL_REQUIREMENT: u8 = 90;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::TrainedFinesse);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK.with_great_strides(0);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.inner_quiet() < 10 {
            Err(ActionError::SpecialConditionNotMet)
        } else if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        100
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        32
    }
}

pub struct RefinedTouch {}
impl RefinedTouch {
    pub const CP_COST: u16 = 24;
}
impl ActionImpl for RefinedTouch {
    const LEVEL_REQUIREMENT: u8 = 92;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::RefinedTouch);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.combo() != Combo::BasicTouch {
            Err(ActionError::ComboRequirementNotMet)
        } else if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        100
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }

    fn transform(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        let iq = state.effects.inner_quiet();
        state.effects.set_inner_quiet(std::cmp::min(10, iq + 1));
    }
}

pub struct QuickInnovation {}
impl ActionImpl for QuickInnovation {
    const LEVEL_REQUIREMENT: u8 = 96;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::QuickInnovation);
    const INCREASES_STEP_COUNT: bool = false;

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_quick_innovation_available(false)
        .with_innovation(0);
    const EFFECT_SET_MASK: Effects = Effects::new().with_innovation(1);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.innovation() != 0 {
            Err(ActionError::SpecialConditionNotMet)
        } else if !state.effects.quick_innovation_available() {
            Err(ActionError::NoRemainingUses)
        } else if !state.effects.quality_actions_allowed() {
            Err(ActionError::QualityAfterProgress)
        } else {
            Ok(())
        }
    }
}

pub struct ImmaculateMend {}
impl ImmaculateMend {
    pub const CP_COST: u16 = 112;
}
impl ActionImpl for ImmaculateMend {
    const LEVEL_REQUIREMENT: u8 = 98;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::ImmaculateMend);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK;
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }

    fn transform(state: &mut SimulationState, settings: &Settings, _condition: Condition) {
        state.durability = settings.max_durability;
    }
}

pub struct TrainedPerfection {}
impl ActionImpl for TrainedPerfection {
    const LEVEL_REQUIREMENT: u8 = 100;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::TrainedPerfection);

    const EFFECT_RESET_MASK: Effects =
        DEFAULT_EFFECT_RESET_MASK.with_trained_perfection_available(false);
    const EFFECT_SET_MASK: Effects = Effects::new().with_trained_perfection_active(true);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if !state.effects.trained_perfection_available() {
            return Err(ActionError::NoRemainingUses);
        }
        Ok(())
    }
}

pub struct StellarSteadyHand {}
impl ActionImpl for StellarSteadyHand {
    const LEVEL_REQUIREMENT: u8 = 90;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::StellarSteadyHand);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK.with_stellar_steady_hand(0);
    const EFFECT_SET_MASK: Effects = Effects::new().with_stellar_steady_hand(3);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        if state.effects.stellar_steady_hand_charges() == 0 {
            return Err(ActionError::NoRemainingUses);
        }
        Ok(())
    }

    fn transform(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        let remaining = state
            .effects
            .stellar_steady_hand_charges()
            .saturating_sub(1);
        state.effects.set_stellar_steady_hand_charges(remaining);
    }
}

pub struct RapidSynthesis {}
impl ActionImpl for RapidSynthesis {
    const LEVEL_REQUIREMENT: u8 = 9;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::RapidSynthesis);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_muscle_memory(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        // Only actions with 100% success rate are supported.
        // Stellar Steady Hand is the only way to achieve 100% success rate with Rapid Synthesis.
        if state.effects.stellar_steady_hand() == 0 {
            return Err(ActionError::UnreliableAction);
        }
        Ok(())
    }

    fn progress_modifier(_state: &SimulationState, settings: &Settings) -> u32 {
        match settings.job_level {
            0..63 => 250,
            63.. => 500,
        }
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }
}

pub struct HastyTouch {}
impl ActionImpl for HastyTouch {
    const LEVEL_REQUIREMENT: u8 = 9;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::HastyTouch);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new().with_expedience(true);

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        // Only actions with 100% success rate are supported.
        // Stellar Steady Hand is the only way to achieve 100% success rate with Rapid Synthesis.
        if state.effects.stellar_steady_hand() == 0 {
            return Err(ActionError::UnreliableAction);
        }
        Ok(())
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        100
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }
}

pub struct DaringTouch {}
impl ActionImpl for DaringTouch {
    const LEVEL_REQUIREMENT: u8 = 96;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::HastyTouch);

    const EFFECT_RESET_MASK: Effects = DEFAULT_EFFECT_RESET_MASK
        .with_great_strides(0)
        .with_trained_perfection_active(false);
    const EFFECT_SET_MASK: Effects = Effects::new();

    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), ActionError> {
        // Only actions with 100% success rate are supported.
        // Stellar Steady Hand is the only way to achieve 100% success rate with Rapid Synthesis.
        if state.effects.stellar_steady_hand() == 0 {
            return Err(ActionError::UnreliableAction);
        }
        if !state.effects.expedience() {
            return Err(ActionError::SpecialConditionNotMet);
        }
        Ok(())
    }

    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        150
    }

    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ActionError {
    /// Actions may be specifically disabled by setting the action mask in the settings.
    Disabled,

    /// The synthesis is already complete, e.g. by reaching max Progress or zero Durability.
    StateIsFinal,

    /// The level requirement of the action is higher the level of the crafter.
    InsufficientLevels,

    /// Not enough CP to use action.
    InsufficientCP,

    /// The action does not have a 100% success rate.
    /// Actions with inherent success rate below 100% may still be used under conditions that
    /// increase the reliability to 100%, such as Stellar Steady Hand.
    UnreliableAction,

    /// Some actions may only be used a limited number of times per synthesis.
    NoRemainingUses,

    /// If the `backload_progress` options is enabled, then it is forbidden to use a Quality-increasing
    /// action after Progress has increased at least once.
    QualityAfterProgress,

    /// Some actions can only be used as a combo right after some other action.
    /// This error is also used for actions that can only be used at the beginning of the synthesis.
    ComboRequirementNotMet,

    /// Some actions require additional specific conditions to be met.
    /// E.g. Prudent Touch can only be used if the Waste Not effect is not active.
    SpecialConditionNotMet,
}

#[derive(strum_macros::EnumIter, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Action {
    BasicSynthesis,
    BasicTouch,
    MasterMend,
    Observe,
    TricksOfTheTrade,
    WasteNot,
    Veneration,
    StandardTouch,
    GreatStrides,
    Innovation,
    WasteNot2,
    ByregotsBlessing,
    PreciseTouch,
    MuscleMemory,
    CarefulSynthesis,
    Manipulation,
    PrudentTouch,
    AdvancedTouch,
    Reflect,
    PreparatoryTouch,
    Groundwork,
    DelicateSynthesis,
    IntensiveSynthesis,
    TrainedEye,
    HeartAndSoul,
    PrudentSynthesis,
    TrainedFinesse,
    RefinedTouch,
    QuickInnovation,
    ImmaculateMend,
    TrainedPerfection,
    StellarSteadyHand,
    RapidSynthesis,
    HastyTouch,
    DaringTouch,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Combo {
    None,
    SynthesisBegin,
    BasicTouch,
    StandardTouch,
}

impl Combo {
    pub const fn into_bits(self) -> u8 {
        match self {
            Self::None => 0,
            Self::BasicTouch => 1,
            Self::StandardTouch => 2,
            Self::SynthesisBegin => 3,
        }
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::BasicTouch,
            2 => Self::StandardTouch,
            _ => Self::SynthesisBegin,
        }
    }
}

impl Action {
    pub const fn time_cost(self) -> u8 {
        match self {
            Self::BasicSynthesis => 3,
            Self::BasicTouch => 3,
            Self::MasterMend => 3,
            Self::Observe => 3,
            Self::TricksOfTheTrade => 3,
            Self::WasteNot => 2,
            Self::Veneration => 2,
            Self::StandardTouch => 3,
            Self::GreatStrides => 2,
            Self::Innovation => 2,
            Self::WasteNot2 => 2,
            Self::ByregotsBlessing => 3,
            Self::PreciseTouch => 3,
            Self::MuscleMemory => 3,
            Self::CarefulSynthesis => 3,
            Self::Manipulation => 2,
            Self::PrudentTouch => 3,
            Self::Reflect => 3,
            Self::PreparatoryTouch => 3,
            Self::Groundwork => 3,
            Self::DelicateSynthesis => 3,
            Self::IntensiveSynthesis => 3,
            Self::AdvancedTouch => 3,
            Self::HeartAndSoul => 3,
            Self::PrudentSynthesis => 3,
            Self::TrainedFinesse => 3,
            Self::RefinedTouch => 3,
            Self::ImmaculateMend => 3,
            Self::TrainedPerfection => 3,
            Self::TrainedEye => 3,
            Self::QuickInnovation => 3,
            Self::StellarSteadyHand => 2,
            Self::RapidSynthesis => 3,
            Self::HastyTouch => 3,
            Self::DaringTouch => 3,
        }
    }

    pub const fn action_id(self) -> u32 {
        match self {
            Self::BasicSynthesis => 100001,
            Self::BasicTouch => 100002,
            Self::MasterMend => 100003,
            Self::Observe => 100010,
            Self::TricksOfTheTrade => 100371,
            Self::WasteNot => 4631,
            Self::Veneration => 19297,
            Self::StandardTouch => 100004,
            Self::GreatStrides => 260,
            Self::Innovation => 19004,
            Self::WasteNot2 => 4639,
            Self::ByregotsBlessing => 100339,
            Self::PreciseTouch => 100128,
            Self::MuscleMemory => 100379,
            Self::CarefulSynthesis => 100203,
            Self::Manipulation => 4574,
            Self::PrudentTouch => 100227,
            Self::AdvancedTouch => 100411,
            Self::Reflect => 100387,
            Self::PreparatoryTouch => 100299,
            Self::Groundwork => 100403,
            Self::DelicateSynthesis => 100323,
            Self::IntensiveSynthesis => 100315,
            Self::TrainedEye => 100283,
            Self::HeartAndSoul => 100419,
            Self::PrudentSynthesis => 100427,
            Self::TrainedFinesse => 100435,
            Self::RefinedTouch => 100443,
            Self::QuickInnovation => 100459,
            Self::ImmaculateMend => 100467,
            Self::TrainedPerfection => 100475,
            Self::StellarSteadyHand => 46843,
            Self::RapidSynthesis => 100363,
            Self::HastyTouch => 100355,
            Self::DaringTouch => 100451,
        }
    }
}
