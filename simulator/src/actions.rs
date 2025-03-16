use crate::{ActionMask, Condition, Settings, SimulationState, SingleUse};

pub trait ActionImpl {
    const LEVEL_REQUIREMENT: u8;
    /// All bits of this mask must be present in the settings' action mask for the action to be enabled.
    const ACTION_MASK: ActionMask;
    /// Does this action trigger ticking effects (e.g. Manipulation)?
    const TICK_EFFECTS: bool = true;

    fn precondition(
        _state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        Ok(())
    }

    fn progress_increase(
        state: &SimulationState,
        settings: &Settings,
        _condition: Condition,
    ) -> u16 {
        let efficiency_mod = Self::base_progress_increase(state, settings) as u64;
        let mut effect_mod = 100;
        if state.effects.muscle_memory() != 0 {
            effect_mod += 100;
        }
        if state.effects.veneration() != 0 {
            effect_mod += 50;
        }
        (settings.base_progress as u64 * efficiency_mod * effect_mod / 10000) as u16
    }

    fn quality_increase(state: &SimulationState, settings: &Settings, condition: Condition) -> u16 {
        let efficieny_mod = Self::base_quality_increase(state, settings) as u64;
        let condition_mod = match condition {
            Condition::Good => 150,
            Condition::Excellent => 400,
            Condition::Poor => 50,
            _ => 100,
        };
        let mut effect_mod = 100;
        if state.effects.innovation() != 0 {
            effect_mod += 50;
        }
        if state.effects.great_strides() != 0 {
            effect_mod += 100;
        }
        let inner_quiet_mod = 100 + 10 * state.effects.inner_quiet() as u64;
        (settings.base_quality as u64
            * efficieny_mod
            * condition_mod
            * effect_mod
            * inner_quiet_mod
            / 100_000_000) as u16
    }

    fn durability_cost(state: &SimulationState, settings: &Settings, _condition: Condition) -> i8 {
        if matches!(state.effects.trained_perfection(), SingleUse::Active) {
            return 0;
        }
        match state.effects.waste_not() {
            0 => Self::base_durability_cost(state, settings),
            _ => (Self::base_durability_cost(state, settings) + 1) / 2,
        }
    }

    fn cp_cost(state: &SimulationState, settings: &Settings, _condition: Condition) -> i16 {
        Self::base_cp_cost(state, settings)
    }

    fn base_progress_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        0
    }
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        0
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        0
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        0
    }

    fn transform_pre(_state: &mut SimulationState, _settings: &Settings, _condition: Condition) {}
    fn transform_post(_state: &mut SimulationState, _settings: &Settings, _condition: Condition) {}

    fn combo(_state: &SimulationState, _settings: &Settings, _condition: Condition) -> Combo {
        Combo::None
    }
}

pub struct BasicSynthesis {}
impl ActionImpl for BasicSynthesis {
    const LEVEL_REQUIREMENT: u8 = 1;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::BasicSynthesis);
    fn base_progress_increase(_state: &SimulationState, settings: &Settings) -> u16 {
        if settings.job_level < 31 { 100 } else { 120 }
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
}

pub struct BasicTouch {}
impl ActionImpl for BasicTouch {
    const LEVEL_REQUIREMENT: u8 = 5;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::BasicTouch);
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        100
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        18
    }
    fn combo(_state: &SimulationState, _settings: &Settings, _condition: Condition) -> Combo {
        Combo::BasicTouch
    }
}

pub struct MasterMend {}
impl ActionImpl for MasterMend {
    const LEVEL_REQUIREMENT: u8 = 7;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::MasterMend);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        88
    }
    fn transform_post(state: &mut SimulationState, settings: &Settings, _condition: Condition) {
        state.durability =
            std::cmp::min(settings.max_durability, state.durability.saturating_add(30));
    }
}

pub struct Observe {}
impl ActionImpl for Observe {
    const LEVEL_REQUIREMENT: u8 = 13;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Observe);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        7
    }
    fn combo(_state: &SimulationState, _settings: &Settings, _condition: Condition) -> Combo {
        Combo::StandardTouch
    }
}

pub struct TricksOfTheTrade {}
impl ActionImpl for TricksOfTheTrade {
    const LEVEL_REQUIREMENT: u8 = 13;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::TricksOfTheTrade);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.heart_and_soul() != SingleUse::Active
            && condition != Condition::Good
            && condition != Condition::Excellent
        {
            return Err(
                "Tricks of the Trade can only be used when the condition is Good or Excellent.",
            );
        }
        Ok(())
    }
    fn transform_post(state: &mut SimulationState, settings: &Settings, condition: Condition) {
        state.cp = std::cmp::min(settings.max_cp, state.cp + 20);
        if condition != Condition::Good && condition != Condition::Excellent {
            state.effects.set_heart_and_soul(SingleUse::Unavailable);
        }
    }
}

pub struct WasteNot {}
impl ActionImpl for WasteNot {
    const LEVEL_REQUIREMENT: u8 = 15;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::WasteNot);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        56
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_waste_not(4);
    }
}

pub struct Veneration {}
impl ActionImpl for Veneration {
    const LEVEL_REQUIREMENT: u8 = 15;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Veneration);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        18
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_veneration(4);
    }
}

pub struct StandardTouch {}
impl ActionImpl for StandardTouch {
    const LEVEL_REQUIREMENT: u8 = 18;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::StandardTouch);
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        125
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(state: &SimulationState, _settings: &Settings) -> i16 {
        match state.combo {
            Combo::BasicTouch => 18,
            _ => 32,
        }
    }
    fn combo(state: &SimulationState, _settings: &Settings, _condition: Condition) -> Combo {
        match state.combo {
            Combo::BasicTouch => Combo::StandardTouch,
            _ => Combo::None,
        }
    }
}

pub struct GreatStrides {}
impl ActionImpl for GreatStrides {
    const LEVEL_REQUIREMENT: u8 = 21;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::GreatStrides);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        32
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_great_strides(3);
    }
}

pub struct Innovation {}
impl ActionImpl for Innovation {
    const LEVEL_REQUIREMENT: u8 = 26;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Innovation);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        18
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_innovation(4);
    }
}

pub struct WasteNot2 {}
impl ActionImpl for WasteNot2 {
    const LEVEL_REQUIREMENT: u8 = 47;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::WasteNot2);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        98
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_waste_not(8);
    }
}

pub struct ByregotsBlessing {}
impl ActionImpl for ByregotsBlessing {
    const LEVEL_REQUIREMENT: u8 = 50;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::ByregotsBlessing);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        match state.effects.inner_quiet() {
            0 => Err("Cannot use Byregot's Blessing when Inner Quiet is 0."),
            _ => Ok(()),
        }
    }
    fn base_quality_increase(state: &SimulationState, _settings: &Settings) -> u16 {
        100 + 20 * state.effects.inner_quiet() as u16
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        24
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_inner_quiet(0);
    }
}

pub struct PreciseTouch {}
impl ActionImpl for PreciseTouch {
    const LEVEL_REQUIREMENT: u8 = 53;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::PreciseTouch);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.heart_and_soul() != SingleUse::Active
            && condition != Condition::Good
            && condition != Condition::Excellent
        {
            return Err("Precise Touch can only be used when the condition is Good or Excellent.");
        }
        Ok(())
    }
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        150
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        18
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, condition: Condition) {
        let iq = state.effects.inner_quiet();
        state.effects.set_inner_quiet(std::cmp::min(10, iq + 1));
        if condition != Condition::Good && condition != Condition::Excellent {
            state.effects.set_heart_and_soul(SingleUse::Unavailable);
        }
    }
}

pub struct MuscleMemory {}
impl ActionImpl for MuscleMemory {
    const LEVEL_REQUIREMENT: u8 = 54;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::MuscleMemory);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.combo != Combo::SynthesisBegin {
            return Err("Muscle Memory can only be used at synthesis begin.");
        }
        Ok(())
    }
    fn base_progress_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        300
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        6
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_muscle_memory(5);
    }
}

pub struct CarefulSynthesis {}
impl ActionImpl for CarefulSynthesis {
    const LEVEL_REQUIREMENT: u8 = 62;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::CarefulSynthesis);
    fn base_progress_increase(_state: &SimulationState, settings: &Settings) -> u16 {
        match settings.job_level {
            0..82 => 150,
            82.. => 180,
        }
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        7
    }
}

pub struct Manipulation {}
impl ActionImpl for Manipulation {
    const LEVEL_REQUIREMENT: u8 = 65;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Manipulation);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        96
    }
    fn transform_pre(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_manipulation(0);
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_manipulation(8);
    }
}

pub struct PrudentTouch {}
impl ActionImpl for PrudentTouch {
    const LEVEL_REQUIREMENT: u8 = 66;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::PrudentTouch);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.waste_not() != 0 {
            return Err("Prudent Touch cannot be used while Waste Not is active.");
        }
        Ok(())
    }
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        100
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        5
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        25
    }
}

pub struct AdvancedTouch {}
impl ActionImpl for AdvancedTouch {
    const LEVEL_REQUIREMENT: u8 = 68;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::AdvancedTouch);
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        150
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(state: &SimulationState, _settings: &Settings) -> i16 {
        match state.combo {
            Combo::StandardTouch => 18,
            _ => 46,
        }
    }
}

pub struct Reflect {}
impl ActionImpl for Reflect {
    const LEVEL_REQUIREMENT: u8 = 69;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Reflect);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.combo != Combo::SynthesisBegin {
            return Err("Reflect can only be used at synthesis begin.");
        }
        Ok(())
    }
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        300
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        6
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        let iq = state.effects.inner_quiet();
        state.effects.set_inner_quiet(std::cmp::min(10, iq + 1));
    }
}

pub struct PreparatoryTouch {}
impl ActionImpl for PreparatoryTouch {
    const LEVEL_REQUIREMENT: u8 = 71;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::PreparatoryTouch);
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        200
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        20
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        40
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        let iq = state.effects.inner_quiet();
        state.effects.set_inner_quiet(std::cmp::min(10, iq + 1));
    }
}

pub struct Groundwork {}
impl ActionImpl for Groundwork {
    const LEVEL_REQUIREMENT: u8 = 72;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Groundwork);
    fn base_progress_increase(state: &SimulationState, settings: &Settings) -> u16 {
        let base = match settings.job_level {
            0..86 => 300,
            86.. => 360,
        };
        if Self::durability_cost(state, settings, Condition::Normal) > state.durability {
            return base / 2;
        }
        base
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        20
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        18
    }
}

pub struct DelicateSynthesis {}
impl ActionImpl for DelicateSynthesis {
    const LEVEL_REQUIREMENT: u8 = 76;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::DelicateSynthesis);
    fn base_progress_increase(_state: &SimulationState, settings: &Settings) -> u16 {
        match settings.job_level {
            0..94 => 100,
            94.. => 150,
        }
    }
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        100
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        32
    }
}

pub struct IntensiveSynthesis {}
impl ActionImpl for IntensiveSynthesis {
    const LEVEL_REQUIREMENT: u8 = 78;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::IntensiveSynthesis);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.heart_and_soul() != SingleUse::Active
            && condition != Condition::Good
            && condition != Condition::Excellent
        {
            return Err(
                "Intensive Synthesis can only be used when the condition is Good or Excellent.",
            );
        }
        Ok(())
    }
    fn base_progress_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        400
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        6
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, condition: Condition) {
        if condition != Condition::Good && condition != Condition::Excellent {
            state.effects.set_heart_and_soul(SingleUse::Unavailable);
        }
    }
}

pub struct TrainedEye {}
impl ActionImpl for TrainedEye {
    const LEVEL_REQUIREMENT: u8 = 80;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::TrainedEye);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.combo != Combo::SynthesisBegin {
            return Err("Trained Eye can only be used at synthesis begin.");
        }
        Ok(())
    }
    fn quality_increase(
        _state: &SimulationState,
        settings: &Settings,
        _condition: Condition,
    ) -> u16 {
        settings.max_quality
    }
    fn base_quality_increase(_state: &SimulationState, settings: &Settings) -> u16 {
        settings.max_quality
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        250
    }
}

pub struct HeartAndSoul {}
impl ActionImpl for HeartAndSoul {
    const LEVEL_REQUIREMENT: u8 = 86;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::HeartAndSoul);
    const TICK_EFFECTS: bool = false;
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.heart_and_soul() != SingleUse::Available {
            return Err("Heart and Sould can only be used once per synthesis.");
        }
        Ok(())
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_heart_and_soul(SingleUse::Active);
    }
}

pub struct PrudentSynthesis {}
impl ActionImpl for PrudentSynthesis {
    const LEVEL_REQUIREMENT: u8 = 88;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::PrudentSynthesis);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.waste_not() != 0 {
            return Err("Prudent Synthesis cannot be used while Waste Not is active.");
        }
        Ok(())
    }
    fn base_progress_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        180
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        5
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        18
    }
}

pub struct TrainedFinesse {}
impl ActionImpl for TrainedFinesse {
    const LEVEL_REQUIREMENT: u8 = 90;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::TrainedFinesse);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.inner_quiet() < 10 {
            return Err("Trained Finesse can only be used when Inner Quiet is 10.");
        }
        Ok(())
    }
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        100
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        32
    }
}

pub struct RefinedTouch {}
impl ActionImpl for RefinedTouch {
    const LEVEL_REQUIREMENT: u8 = 92;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::RefinedTouch);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.combo != Combo::BasicTouch {
            return Err("Refined Touch can only be used after Observe or Standard Touch.");
        }
        Ok(())
    }
    fn base_quality_increase(_state: &SimulationState, _settings: &Settings) -> u16 {
        100
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> i8 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        24
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        let iq = state.effects.inner_quiet();
        state.effects.set_inner_quiet(std::cmp::min(10, iq + 1));
    }
}

pub struct QuickInnovation {}
impl ActionImpl for QuickInnovation {
    const LEVEL_REQUIREMENT: u8 = 96;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::QuickInnovation);
    const TICK_EFFECTS: bool = false;
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.innovation() != 0 {
            return Err("Quick Innovation cannot be used while Innovation is active.");
        }
        if !state.effects.quick_innovation_available() {
            return Err("Quick Innovation can only be used once per synthesis.");
        }
        Ok(())
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_innovation(1);
        state.effects.set_quick_innovation_available(false);
    }
}

pub struct ImmaculateMend {}
impl ActionImpl for ImmaculateMend {
    const LEVEL_REQUIREMENT: u8 = 98;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::ImmaculateMend);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> i16 {
        112
    }
    fn transform_post(state: &mut SimulationState, settings: &Settings, _condition: Condition) {
        state.durability = settings.max_durability;
    }
}

pub struct TrainedPerfection {}
impl ActionImpl for TrainedPerfection {
    const LEVEL_REQUIREMENT: u8 = 100;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::TrainedPerfection);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.trained_perfection() != SingleUse::Available {
            return Err("Trained Perfection can only be used once per synthesis.");
        }
        Ok(())
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_trained_perfection(SingleUse::Active);
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
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
            Self::SynthesisBegin => 1,
            Self::BasicTouch => 2,
            Self::StandardTouch => 3,
        }
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            1 => Self::SynthesisBegin,
            2 => Self::BasicTouch,
            3 => Self::StandardTouch,
            _ => Self::None,
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
        }
    }
}
