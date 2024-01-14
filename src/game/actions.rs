use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

use crate::game::{conditions::Condition, effects::Effects};

pub const PROG_DENOM: f32 = 20.0;
pub const QUAL_DENOM: f32 = 800.0;

#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumCountMacro, EnumIter, Hash)]
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
    // Final Appraisal
    WasteNot2,
    ByregotsBlessing,
    PreciseTouch,
    MuscleMemory,
    CarefulSynthesis,
    Manipulation,
    PrudentTouch,
    FocusedSynthesis,
    FocusedTouch,
    Reflect,
    PreparatoryTouch,
    Groundwork,
    DelicateSynthesis,
    IntensiveSynthesis,
    AdvancedTouch,
    PrudentSynthesis,
    TrainedFinesse,
}

impl Action {
    pub const fn time_cost(&self) -> i32 {
        match *self {
            Action::BasicSynthesis => 3,
            Action::BasicTouch => 3,
            Action::MasterMend => 3,
            Action::Observe => 2,
            Action::TricksOfTheTrade => 2,
            Action::WasteNot => 2,
            Action::Veneration => 2,
            Action::StandardTouch => 3,
            Action::GreatStrides => 2,
            Action::Innovation => 2,
            Action::WasteNot2 => 2,
            Action::ByregotsBlessing => 3,
            Action::PreciseTouch => 3,
            Action::MuscleMemory => 3,
            Action::CarefulSynthesis => 3,
            Action::Manipulation => 2,
            Action::PrudentTouch => 3,
            Action::FocusedSynthesis => 3,
            Action::FocusedTouch => 3,
            Action::Reflect => 3,
            Action::PreparatoryTouch => 3,
            Action::Groundwork => 3,
            Action::DelicateSynthesis => 3,
            Action::IntensiveSynthesis => 3,
            Action::AdvancedTouch => 3,
            Action::PrudentSynthesis => 3,
            Action::TrainedFinesse => 3,
        }
    }

    pub const fn base_cp_cost(&self) -> i32 {
        match *self {
            Action::BasicSynthesis => 0,
            Action::BasicTouch => 18,
            Action::MasterMend => 88,
            Action::Observe => 7,
            Action::TricksOfTheTrade => 0,
            Action::WasteNot => 56,
            Action::Veneration => 18,
            Action::StandardTouch => 18,
            Action::GreatStrides => 32,
            Action::Innovation => 18,
            // Action::Final Appraisal => 0,
            Action::WasteNot2 => 98,
            Action::ByregotsBlessing => 24,
            Action::PreciseTouch => 18,
            Action::MuscleMemory => 6,
            Action::CarefulSynthesis => 7,
            Action::Manipulation => 96,
            Action::PrudentTouch => 25,
            Action::FocusedSynthesis => 5,
            Action::FocusedTouch => 18,
            Action::Reflect => 6,
            Action::PreparatoryTouch => 40,
            Action::Groundwork => 18,
            Action::DelicateSynthesis => 32,
            Action::IntensiveSynthesis => 6,
            Action::AdvancedTouch => 18,
            Action::PrudentSynthesis => 18,
            Action::TrainedFinesse => 32,
        }
    }

    pub fn cp_cost(&self, _: &Effects, condition: Condition) -> i32 {
        let base_cost = self.base_cp_cost() as f32;
        let condition_multiplier = match condition {
            Condition::Pliant => 0.50,
            _ => 1.00,
        };
        let final_cost = base_cost * condition_multiplier;
        return final_cost.ceil() as i32;
    }

    pub const fn base_durability_cost(&self) -> i32 {
        match *self {
            Action::BasicSynthesis => 10,
            Action::BasicTouch => 10,
            Action::MasterMend => 0,
            Action::Observe => 0,
            Action::TricksOfTheTrade => 0,
            Action::WasteNot => 0,
            Action::Veneration => 0,
            Action::StandardTouch => 10,
            Action::GreatStrides => 0,
            Action::Innovation => 0,
            // Action::Final Appraisal => 0,
            Action::WasteNot2 => 0,
            Action::ByregotsBlessing => 10,
            Action::PreciseTouch => 10,
            Action::MuscleMemory => 10,
            Action::CarefulSynthesis => 10,
            Action::Manipulation => 0,
            Action::PrudentTouch => 5,
            Action::FocusedSynthesis => 10,
            Action::FocusedTouch => 10,
            Action::Reflect => 10,
            Action::PreparatoryTouch => 20,
            Action::Groundwork => 20,
            Action::DelicateSynthesis => 10,
            Action::IntensiveSynthesis => 10,
            Action::AdvancedTouch => 10,
            Action::PrudentSynthesis => 5,
            Action::TrainedFinesse => 0,
        }
    }

    pub fn durability_cost(&self, effects: &Effects, condition: Condition) -> i32 {
        let base_cost = self.base_durability_cost() as f32;
        let condition_multiplier = match condition {
            Condition::Sturdy => 0.50,
            _ => 1.00,
        };
        let effect_multiplier = match effects.waste_not > 0 {
            true => 0.50,
            false => 1.00,
        };
        let final_cost = base_cost * condition_multiplier * effect_multiplier;
        return final_cost.ceil() as i32;
    }

    pub const fn base_progress_increase(&self) -> f32 {
        match *self {
            Action::BasicSynthesis => 1.20,
            Action::MuscleMemory => 3.00,
            Action::CarefulSynthesis => 1.80,
            Action::FocusedSynthesis => 2.00,
            Action::Groundwork => 3.60,
            Action::DelicateSynthesis => 1.00,
            Action::IntensiveSynthesis => 4.00,
            Action::PrudentSynthesis => 1.80,
            _ => 0.00,
        }
    }

    pub fn base_progress_increase_int(&self) -> i32 {
        (self.base_progress_increase() * PROG_DENOM) as i32
    }

    pub fn progress_increase(&self, effects: &Effects, condition: Condition) -> i32 {
        let base_increase = self.base_progress_increase();
        let condition_multiplier = match condition {
            Condition::Malleable => 1.50,
            _ => 1.00,
        };
        let mut effect_multiplier = 1.00;
        if effects.muscle_memory > 0 {
            effect_multiplier += 1.00;
        }
        if effects.veneration > 0 {
            effect_multiplier += 0.50;
        }
        return (base_increase * condition_multiplier * effect_multiplier * PROG_DENOM) as i32;
    }

    pub const fn base_quality_increase(&self) -> f32 {
        match *self {
            Action::BasicTouch => 1.00,
            Action::StandardTouch => 1.25,
            Action::PreciseTouch => 1.50,
            Action::PrudentTouch => 1.00,
            Action::FocusedTouch => 1.50,
            Action::Reflect => 1.00,
            Action::PreparatoryTouch => 2.00,
            Action::DelicateSynthesis => 1.00,
            Action::AdvancedTouch => 1.50,
            Action::TrainedFinesse => 1.00,
            Action::ByregotsBlessing => 1.00,
            _ => 0.00,
        }
    }

    pub fn quality_increase(&self, effects: &Effects, condition: Condition) -> i32 {
        let base_increase = self.base_quality_increase()
            + match *self {
                Action::ByregotsBlessing => effects.inner_quiet as f32 * 0.20,
                _ => 0.00,
            };
        let condition_multiplier = match condition {
            Condition::Good => 1.50,
            Condition::Excellent => 4.00,
            Condition::Poor => 0.50,
            _ => 1.00,
        };
        let iq_multiplier = 1.00 + effects.inner_quiet as f32 * 0.10;
        let mut effect_multiplier = 1.00;
        if effects.innovation > 0 {
            effect_multiplier += 0.50;
        }
        if effects.great_strides > 0 {
            effect_multiplier += 1.00;
        }
        return (base_increase
            * condition_multiplier
            * iq_multiplier
            * effect_multiplier
            * QUAL_DENOM) as i32;
    }

    pub const fn combo_action(&self) -> Option<Self> {
        match *self {
            Action::StandardTouch => Some(Action::BasicTouch),
            Action::AdvancedTouch => Some(Action::StandardTouch),
            Action::FocusedSynthesis => Some(Action::Observe),
            Action::FocusedTouch => Some(Action::Observe),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_cost() {
        let effects: Effects = Default::default();
        assert_eq!(25, Action::PrudentTouch.cp_cost(&effects, Condition::Good));
        assert_eq!(
            13,
            Action::PrudentTouch.cp_cost(&effects, Condition::Pliant)
        );
    }

    #[test]
    fn test_durability_cost() {
        let effects_default: Effects = Default::default();
        let effects_waste_not: Effects = Effects {
            waste_not: 1,
            ..Default::default()
        };
        assert_eq!(
            5,
            Action::PrudentSynthesis.durability_cost(&effects_default, Condition::Normal)
        );
        assert_eq!(
            3,
            Action::PrudentSynthesis.durability_cost(&effects_default, Condition::Sturdy)
        );
        assert_eq!(
            10,
            Action::BasicTouch.durability_cost(&effects_default, Condition::Normal)
        );
        assert_eq!(
            5,
            Action::BasicTouch.durability_cost(&effects_default, Condition::Sturdy)
        );
        assert_eq!(
            5,
            Action::BasicTouch.durability_cost(&effects_waste_not, Condition::Normal)
        );
        assert_eq!(
            3,
            Action::BasicTouch.durability_cost(&effects_waste_not, Condition::Sturdy)
        );
    }
}
