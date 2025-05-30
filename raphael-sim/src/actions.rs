use crate::{ActionMask, Condition, Settings, SimulationState};

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

    #[inline]
    fn progress_increase(state: &SimulationState, settings: &Settings) -> u32 {
        let action_mod = u32::from(Self::progress_modifier(state, settings));
        let effect_mod = u32::from(state.effects.progress_modifier());
        u32::from(settings.base_progress) * action_mod * effect_mod / 10000
    }

    #[inline]
    fn quality_increase(state: &SimulationState, settings: &Settings, condition: Condition) -> u32 {
        let action_mod = u32::from(Self::quality_modifier(state, settings));
        let effect_mod = u32::from(state.effects.quality_modifier());
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
    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        100
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
    fn combo(_state: &SimulationState, _settings: &Settings, _condition: Condition) -> Combo {
        Combo::BasicTouch
    }
}

pub struct MasterMend {}
impl MasterMend {
    pub const CP_COST: u16 = 88;
}
impl ActionImpl for MasterMend {
    const LEVEL_REQUIREMENT: u8 = 7;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::MasterMend);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
    fn transform_post(state: &mut SimulationState, settings: &Settings, _condition: Condition) {
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
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
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
        if !state.effects.heart_and_soul_active()
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
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_waste_not(4);
    }
}

pub struct Veneration {}
impl Veneration {
    pub const CP_COST: u16 = 18;
}
impl ActionImpl for Veneration {
    const LEVEL_REQUIREMENT: u8 = 15;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Veneration);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_veneration(4);
    }
}

pub struct StandardTouch {}
impl ActionImpl for StandardTouch {
    const LEVEL_REQUIREMENT: u8 = 18;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::StandardTouch);
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
    fn combo(state: &SimulationState, _settings: &Settings, _condition: Condition) -> Combo {
        match state.effects.combo() {
            Combo::BasicTouch => Combo::StandardTouch,
            _ => Combo::None,
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
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        match state.effects.allow_quality_actions() {
            false => Err("Forbidden by backload_progress setting"),
            true => Ok(()),
        }
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_great_strides(3);
    }
}

pub struct Innovation {}
impl Innovation {
    pub const CP_COST: u16 = 18;
}
impl ActionImpl for Innovation {
    const LEVEL_REQUIREMENT: u8 = 26;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::Innovation);
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        match state.effects.allow_quality_actions() {
            false => Err("Forbidden by backload_progress setting"),
            true => Ok(()),
        }
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_innovation(4);
    }
}

pub struct WasteNot2 {}
impl WasteNot2 {
    pub const CP_COST: u16 = 98;
}
impl ActionImpl for WasteNot2 {
    const LEVEL_REQUIREMENT: u8 = 47;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::WasteNot2);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
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
    fn quality_modifier(state: &SimulationState, _settings: &Settings) -> u32 {
        100 + 20 * state.effects.inner_quiet() as u32
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        10
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
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
        if !state.effects.heart_and_soul_active()
            && condition != Condition::Good
            && condition != Condition::Excellent
        {
            return Err("Precise Touch can only be used when the condition is Good or Excellent.");
        }
        Ok(())
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
    fn transform_post(state: &mut SimulationState, _settings: &Settings, condition: Condition) {
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
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.combo() != Combo::SynthesisBegin {
            return Err("Muscle Memory can only be used at synthesis begin.");
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
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_muscle_memory(5);
    }
}

pub struct CarefulSynthesis {}
impl ActionImpl for CarefulSynthesis {
    const LEVEL_REQUIREMENT: u8 = 62;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::CarefulSynthesis);
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
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
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
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.combo() != Combo::SynthesisBegin {
            return Err("Reflect can only be used at synthesis begin.");
        }
        Ok(())
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
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
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
    fn quality_modifier(_state: &SimulationState, _settings: &Settings) -> u32 {
        200
    }
    fn base_durability_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        20
    }
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
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
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        condition: Condition,
    ) -> Result<(), &'static str> {
        if !state.effects.heart_and_soul_active()
            && condition != Condition::Good
            && condition != Condition::Excellent
        {
            return Err(
                "Intensive Synthesis can only be used when the condition is Good or Excellent.",
            );
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
    fn transform_post(state: &mut SimulationState, _settings: &Settings, condition: Condition) {
        if condition != Condition::Good && condition != Condition::Excellent {
            state.effects.set_heart_and_soul_active(false);
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
        if state.effects.combo() != Combo::SynthesisBegin {
            return Err("Trained Eye can only be used at synthesis begin.");
        }
        Ok(())
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
    const TICK_EFFECTS: bool = false;
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if !state.effects.heart_and_soul_available() {
            return Err("Heart and Sould can only be used once per synthesis.");
        }
        Ok(())
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_heart_and_soul_available(false);
        state.effects.set_heart_and_soul_active(true);
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
    fn precondition(
        state: &SimulationState,
        _settings: &Settings,
        _condition: Condition,
    ) -> Result<(), &'static str> {
        if state.effects.combo() != Combo::BasicTouch {
            return Err("Refined Touch can only be used after Observe or Basic Touch.");
        }
        Ok(())
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
impl ImmaculateMend {
    pub const CP_COST: u16 = 112;
}
impl ActionImpl for ImmaculateMend {
    const LEVEL_REQUIREMENT: u8 = 98;
    const ACTION_MASK: ActionMask = ActionMask::none().add(Action::ImmaculateMend);
    fn base_cp_cost(_state: &SimulationState, _settings: &Settings) -> u16 {
        Self::CP_COST
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
        if !state.effects.trained_perfection_available() {
            return Err("Trained Perfection can only be used once per synthesis.");
        }
        Ok(())
    }
    fn transform_post(state: &mut SimulationState, _settings: &Settings, _condition: Condition) {
        state.effects.set_trained_perfection_available(false);
        state.effects.set_trained_perfection_active(true);
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
        }
    }
}
