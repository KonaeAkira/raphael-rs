use crate::{Condition, Effects};

use super::Settings;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Action {
    BasicSynthesis,
    BasicTouch,
    MasterMend,
    Observe,
    TricksOfTheTrade,
    WasteNot,
    Veneration,
    StandardTouch, // out-of-combo version
    ComboStandardTouch,
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
    AdvancedTouch, // out-of-combo version
    ComboAdvancedTouch,
    PrudentSynthesis,
    TrainedFinesse,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ComboAction {
    SynthesisBegin,
    Observe,
    BasicTouch,
    StandardTouch,
}

impl Action {
    pub const fn level_requirement(self) -> u32 {
        match self {
            Action::BasicSynthesis => 1,
            Action::BasicTouch => 5,
            Action::MasterMend => 7,
            Action::Observe => 13,
            Action::TricksOfTheTrade => 13,
            Action::WasteNot => 15,
            Action::Veneration => 15,
            Action::StandardTouch => 18,
            Action::ComboStandardTouch => 18,
            Action::GreatStrides => 21,
            Action::Innovation => 26,
            Action::WasteNot2 => 47,
            Action::ByregotsBlessing => 50,
            Action::PreciseTouch => 53,
            Action::MuscleMemory => 54,
            Action::CarefulSynthesis => 62,
            Action::Manipulation => 65,
            Action::PrudentTouch => 66,
            Action::FocusedSynthesis => 67,
            Action::FocusedTouch => 68,
            Action::Reflect => 69,
            Action::PreparatoryTouch => 71,
            Action::Groundwork => 72,
            Action::DelicateSynthesis => 76,
            Action::IntensiveSynthesis => 78,
            Action::AdvancedTouch => 84,
            Action::ComboAdvancedTouch => 84,
            Action::PrudentSynthesis => 88,
            Action::TrainedFinesse => 90,
        }
    }

    pub const fn time_cost(self) -> i32 {
        match self {
            Action::BasicSynthesis => 3,
            Action::BasicTouch => 3,
            Action::MasterMend => 3,
            Action::Observe => 3,
            Action::TricksOfTheTrade => 2,
            Action::WasteNot => 2,
            Action::Veneration => 2,
            Action::StandardTouch => 3,
            Action::ComboStandardTouch => 3,
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
            Action::ComboAdvancedTouch => 3,
            Action::PrudentSynthesis => 3,
            Action::TrainedFinesse => 3,
        }
    }

    pub const fn base_cp_cost(self) -> i16 {
        match self {
            Action::BasicSynthesis => 0,
            Action::BasicTouch => 18,
            Action::MasterMend => 88,
            Action::Observe => 7,
            Action::TricksOfTheTrade => 0,
            Action::WasteNot => 56,
            Action::Veneration => 18,
            Action::StandardTouch => 32,
            Action::ComboStandardTouch => 18,
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
            Action::AdvancedTouch => 46,
            Action::ComboAdvancedTouch => 18,
            Action::PrudentSynthesis => 18,
            Action::TrainedFinesse => 32,
        }
    }

    pub const fn cp_cost(self, _: &Effects, condition: Condition) -> i16 {
        match condition {
            Condition::Pliant => (self.base_cp_cost() + 1) / 2,
            _ => self.base_cp_cost(),
        }
    }

    pub const fn base_durability_cost(self) -> i16 {
        match self {
            Action::BasicSynthesis => 10,
            Action::BasicTouch => 10,
            Action::MasterMend => 0,
            Action::Observe => 0,
            Action::TricksOfTheTrade => 0,
            Action::WasteNot => 0,
            Action::Veneration => 0,
            Action::StandardTouch => 10,
            Action::ComboStandardTouch => 10,
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
            Action::ComboAdvancedTouch => 10,
            Action::PrudentSynthesis => 5,
            Action::TrainedFinesse => 0,
        }
    }

    pub const fn durability_cost(self, effects: &Effects, condition: Condition) -> i16 {
        let base_cost = match condition {
            Condition::Sturdy => (self.base_durability_cost() + 1) / 2,
            _ => self.base_durability_cost(),
        };
        let mut effect_bonus = 0;
        if effects.waste_not > 0 {
            effect_bonus += base_cost / 2;
        }
        base_cost - effect_bonus
    }

    pub const fn progress_efficiency(self, job_level: u8) -> u64 {
        match self {
            Action::BasicSynthesis => {
                if job_level < 31 {
                    100
                } else {
                    120
                }
            }
            Action::MuscleMemory => 300,
            Action::CarefulSynthesis => {
                if job_level < 82 {
                    150
                } else {
                    180
                }
            }
            Action::FocusedSynthesis => 200,
            Action::Groundwork => {
                if job_level < 86 {
                    300
                } else {
                    360
                }
            }
            Action::DelicateSynthesis => 100,
            Action::IntensiveSynthesis => 400,
            Action::PrudentSynthesis => 180,
            _ => 0,
        }
    }

    pub const fn progress_increase(
        self,
        settings: &Settings,
        effects: &Effects,
        condition: Condition,
    ) -> u32 {
        let efficiency_mod = self.progress_efficiency(settings.job_level);
        let condition_mod = match condition {
            Condition::Malleable => 150,
            _ => 100,
        };
        let mut effect_mod = 100;
        if effects.muscle_memory > 0 {
            effect_mod += 100;
        }
        if effects.veneration > 0 {
            effect_mod += 50;
        }
        (settings.base_progress as u64 * efficiency_mod * condition_mod * effect_mod / 1000000)
            as u32
    }

    pub const fn quality_efficiency(self, inner_quiet: u8) -> u64 {
        match self {
            Action::BasicTouch => 100,
            Action::StandardTouch => 125,
            Action::ComboStandardTouch => 125,
            Action::PreciseTouch => 150,
            Action::PrudentTouch => 100,
            Action::FocusedTouch => 150,
            Action::Reflect => 100,
            Action::PreparatoryTouch => 200,
            Action::DelicateSynthesis => 100,
            Action::AdvancedTouch => 150,
            Action::ComboAdvancedTouch => 150,
            Action::TrainedFinesse => 100,
            Action::ByregotsBlessing => 100 + 20 * inner_quiet as u64,
            _ => 0,
        }
    }

    pub const fn quality_increase(
        self,
        settings: &Settings,
        effects: &Effects,
        condition: Condition,
    ) -> u32 {
        let efficieny_mod = self.quality_efficiency(effects.inner_quiet);
        let condition_mod = match condition {
            Condition::Good => 150,
            Condition::Excellent => 400,
            Condition::Poor => 50,
            _ => 100,
        };
        let mut effect_mod = 100;
        if effects.innovation != 0 {
            effect_mod += 50;
        }
        if effects.great_strides != 0 {
            effect_mod += 100;
        }
        let inner_quiet_mod = 100 + 10 * effects.inner_quiet as u64;
        (settings.base_quality as u64
            * efficieny_mod
            * condition_mod
            * effect_mod
            * inner_quiet_mod
            / 100000000) as u32
    }

    pub const fn required_combo(self) -> Option<ComboAction> {
        match self {
            Action::Reflect => Some(ComboAction::SynthesisBegin),
            Action::MuscleMemory => Some(ComboAction::SynthesisBegin),
            Action::ComboStandardTouch => Some(ComboAction::BasicTouch),
            Action::ComboAdvancedTouch => Some(ComboAction::StandardTouch),
            Action::FocusedSynthesis => Some(ComboAction::Observe),
            Action::FocusedTouch => Some(ComboAction::Observe),
            _ => None,
        }
    }

    pub const fn to_combo(self) -> Option<ComboAction> {
        match self {
            Action::BasicTouch => Some(ComboAction::BasicTouch),
            Action::ComboStandardTouch => Some(ComboAction::StandardTouch),
            Action::Observe => Some(ComboAction::Observe),
            _ => None,
        }
    }

    pub fn display_name(self) -> String {
        match self {
            Action::BasicSynthesis => "Basic Synthesis",
            Action::BasicTouch => "Basic Touch",
            Action::MasterMend => "Master's Mend",
            Action::Observe => "Observe",
            Action::TricksOfTheTrade => "Tricks of the Trade",
            Action::WasteNot => "Waste Not",
            Action::Veneration => "Veneration",
            Action::StandardTouch | Action::ComboStandardTouch => "Standard Touch",
            Action::GreatStrides => "Great Strides",
            Action::Innovation => "Innovation",
            Action::WasteNot2 => "Waste Not II",
            Action::ByregotsBlessing => "Byregot's Blessing",
            Action::PreciseTouch => "Precise Touch",
            Action::MuscleMemory => "Muscle Memory",
            Action::CarefulSynthesis => "Careful Synthesis",
            Action::Manipulation => "Manipulation",
            Action::PrudentTouch => "Prudent Touch",
            Action::FocusedSynthesis => "Focused Synthesis",
            Action::FocusedTouch => "Focused Touch",
            Action::Reflect => "Reflect",
            Action::PreparatoryTouch => "Preparatory Touch",
            Action::Groundwork => "Groundwork",
            Action::DelicateSynthesis => "Delicate Synthesis",
            Action::IntensiveSynthesis => "Intensive Synthesis",
            Action::AdvancedTouch | Action::ComboAdvancedTouch => "Advanced Touch",
            Action::PrudentSynthesis => "Prudent Synthesis",
            Action::TrainedFinesse => "Trained Finesse",
        }
        .to_string()
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
