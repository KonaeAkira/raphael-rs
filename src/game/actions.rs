use strum_macros::{EnumCount as EnumCountMacro, EnumIter};

use crate::game::{conditions::Condition, effects::Effects};

pub const PROG_DENOM: i32 = 20;
pub const QUAL_DENOM: i32 = 800;

#[macro_export]
macro_rules! progress {
    ($l:literal) => {
        ($l as f32 * PROG_DENOM as f32) as i32
    };
}

#[macro_export]
macro_rules! quality {
    ($l:literal) => {
        ($l as f32 * QUAL_DENOM as f32) as i32
    };
}

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

    pub const fn cp_cost(&self, _: &Effects, condition: Condition) -> i32 {
        match condition {
            Condition::Pliant => (self.base_cp_cost() + 1) / 2,
            _ => self.base_cp_cost(),
        }
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
        let base_cost: i32 = match condition {
            Condition::Sturdy => (self.base_durability_cost() + 1) / 2,
            _ => self.base_durability_cost(),
        };
        let mut effect_bonus: i32 = 0;
        if effects.waste_not > 0 {
            effect_bonus += base_cost / 2;
        }
        base_cost - effect_bonus
    }

    pub const fn base_progress_increase(&self) -> i32 {
        PROG_DENOM
            * match *self {
                Action::BasicSynthesis => 120,
                Action::MuscleMemory => 300,
                Action::CarefulSynthesis => 180,
                Action::FocusedSynthesis => 200,
                Action::Groundwork => 360,
                Action::DelicateSynthesis => 100,
                Action::IntensiveSynthesis => 400,
                Action::PrudentSynthesis => 180,
                _ => 0,
            }
    }

    pub const fn progress_increase(&self, effects: &Effects, condition: Condition) -> i32 {
        let base_increase: i32 = match condition {
            Condition::Malleable => self.base_progress_increase() * 3 / 2,
            _ => self.base_progress_increase(),
        };
        let mut effect_bonus: i32 = 0;
        if effects.muscle_memory > 0 {
            effect_bonus += base_increase;
        }
        if effects.veneration > 0 {
            effect_bonus += base_increase / 2;
        }
        base_increase + effect_bonus
    }

    pub const fn base_quality_increase(&self) -> i32 {
        QUAL_DENOM
            * match *self {
                Action::BasicTouch => 100,
                Action::StandardTouch => 125,
                Action::PreciseTouch => 150,
                Action::PrudentTouch => 100,
                Action::FocusedTouch => 150,
                Action::Reflect => 100,
                Action::PreparatoryTouch => 200,
                Action::DelicateSynthesis => 100,
                Action::AdvancedTouch => 150,
                Action::TrainedFinesse => 100,
                Action::ByregotsBlessing => 100,
                _ => 0,
            }
    }

    pub const fn quality_increase(&self, effects: &Effects, condition: Condition) -> i32 {
        let mut base_increase: i32 = match *self {
            Action::ByregotsBlessing => {
                self.base_quality_increase() * (2 * effects.inner_quiet as i32 + 10) / 10
            }
            _ => self.base_quality_increase(),
        };
        match condition {
            Condition::Good => base_increase += base_increase / 2,
            Condition::Excellent => base_increase *= 4,
            Condition::Poor => base_increase /= 2,
            _ => (),
        };
        base_increase += base_increase * effects.inner_quiet as i32 / 10;
        let mut effect_bonus: i32 = 0;
        if effects.innovation > 0 {
            effect_bonus += base_increase / 2;
        }
        if effects.great_strides > 0 {
            effect_bonus += base_increase;
        }
        return base_increase + effect_bonus;
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
