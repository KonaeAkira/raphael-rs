use crate::{Condition, Effects, SingleUse};

use super::Settings;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Action {
    BasicSynthesis,
    BasicTouch,
    MasterMend,
    Observe,
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
    AdvancedTouch, // out-of-combo version
    ComboAdvancedTouch,
    Reflect,
    PreparatoryTouch,
    Groundwork,
    DelicateSynthesis,
    IntensiveSynthesis,
    HeartAndSoul,
    PrudentSynthesis,
    TrainedFinesse,
    TrainedEye,
    ComboRefinedTouch,
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
        self as _
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
    pub const fn level_requirement(self) -> u8 {
        match self {
            Action::BasicSynthesis => 1,
            Action::BasicTouch => 5,
            Action::MasterMend => 7,
            Action::Observe => 13,
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
            Action::AdvancedTouch => 68,
            Action::ComboAdvancedTouch => 68,
            Action::Reflect => 69,
            Action::PreparatoryTouch => 71,
            Action::Groundwork => 72,
            Action::DelicateSynthesis => 76,
            Action::IntensiveSynthesis => 78,
            Action::HeartAndSoul => 86,
            Action::PrudentSynthesis => 88,
            Action::TrainedFinesse => 90,
            Action::TrainedEye => 90,
            Action::ComboRefinedTouch => 92,
            Action::QuickInnovation => 96,
            Action::ImmaculateMend => 98,
            Action::TrainedPerfection => 100,
        }
    }

    pub const fn time_cost(self) -> i16 {
        match self {
            Action::BasicSynthesis => 3,
            Action::BasicTouch => 3,
            Action::MasterMend => 3,
            Action::Observe => 3,
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
            Action::Reflect => 3,
            Action::PreparatoryTouch => 3,
            Action::Groundwork => 3,
            Action::DelicateSynthesis => 3,
            Action::IntensiveSynthesis => 3,
            Action::AdvancedTouch => 3,
            Action::ComboAdvancedTouch => 3,
            Action::HeartAndSoul => 3,
            Action::PrudentSynthesis => 3,
            Action::TrainedFinesse => 3,
            Action::ComboRefinedTouch => 3,
            Action::ImmaculateMend => 3,
            Action::TrainedPerfection => 3,
            Action::TrainedEye => 3,
            Action::QuickInnovation => 3,
        }
    }

    pub const fn cp_cost(self) -> i16 {
        match self {
            Action::BasicSynthesis => 0,
            Action::BasicTouch => 18,
            Action::MasterMend => 88,
            Action::Observe => 7,
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
            Action::Reflect => 6,
            Action::PreparatoryTouch => 40,
            Action::Groundwork => 18,
            Action::DelicateSynthesis => 32,
            Action::IntensiveSynthesis => 6,
            Action::AdvancedTouch => 46,
            Action::ComboAdvancedTouch => 18,
            Action::HeartAndSoul => 0,
            Action::PrudentSynthesis => 18,
            Action::TrainedFinesse => 32,
            Action::ComboRefinedTouch => 24,
            Action::ImmaculateMend => 112,
            Action::TrainedPerfection => 0,
            Action::TrainedEye => 250,
            Action::QuickInnovation => 0,
        }
    }

    pub const fn base_durability_cost(self) -> i8 {
        match self {
            Action::BasicSynthesis => 10,
            Action::BasicTouch => 10,
            Action::MasterMend => 0,
            Action::Observe => 0,
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
            Action::Reflect => 10,
            Action::PreparatoryTouch => 20,
            Action::Groundwork => 20,
            Action::DelicateSynthesis => 10,
            Action::IntensiveSynthesis => 10,
            Action::AdvancedTouch => 10,
            Action::ComboAdvancedTouch => 10,
            Action::HeartAndSoul => 0,
            Action::PrudentSynthesis => 5,
            Action::TrainedFinesse => 0,
            Action::ComboRefinedTouch => 10,
            Action::ImmaculateMend => 0,
            Action::TrainedPerfection => 0,
            Action::TrainedEye => 0,
            Action::QuickInnovation => 0,
        }
    }

    pub const fn durability_cost(self, effects: &Effects) -> i8 {
        if matches!(effects.trained_perfection(), SingleUse::Active) {
            return 0;
        }
        match effects.waste_not() {
            0 => self.base_durability_cost(),
            _ => (self.base_durability_cost() + 1) / 2,
        }
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
            Action::Groundwork => {
                if job_level < 86 {
                    300
                } else {
                    360
                }
            }
            Action::DelicateSynthesis => {
                if job_level < 94 {
                    100
                } else {
                    150
                }
            }
            Action::IntensiveSynthesis => 400,
            Action::PrudentSynthesis => 180,
            _ => 0,
        }
    }

    pub const fn progress_increase(self, settings: &Settings, effects: &Effects) -> u16 {
        let efficiency_mod = self.progress_efficiency(settings.job_level);
        let mut effect_mod = 100;
        if effects.muscle_memory() > 0 {
            effect_mod += 100;
        }
        if effects.veneration() > 0 {
            effect_mod += 50;
        }
        (settings.base_progress as u64 * efficiency_mod * effect_mod / 10000) as u16
    }

    pub const fn quality_efficiency(self, inner_quiet: u8) -> u64 {
        match self {
            Action::BasicTouch => 100,
            Action::StandardTouch => 125,
            Action::ComboStandardTouch => 125,
            Action::PreciseTouch => 150,
            Action::PrudentTouch => 100,
            Action::Reflect => 300,
            Action::PreparatoryTouch => 200,
            Action::DelicateSynthesis => 100,
            Action::AdvancedTouch => 150,
            Action::ComboAdvancedTouch => 150,
            Action::TrainedFinesse => 100,
            Action::ComboRefinedTouch => 100,
            Action::ByregotsBlessing => 100 + 20 * inner_quiet as u64,
            _ => 0,
        }
    }

    pub const fn quality_increase(
        self,
        settings: &Settings,
        effects: &Effects,
        condition: Condition,
    ) -> u16 {
        if matches!(self, Action::TrainedEye) {
            return settings.max_quality;
        }
        let efficieny_mod = self.quality_efficiency(effects.inner_quiet());
        let condition_mod = match condition {
            Condition::Good => 150,
            Condition::Excellent => 400,
            Condition::Poor => 50,
            _ => 100,
        };
        let mut effect_mod = 100;
        if effects.innovation() != 0 {
            effect_mod += 50;
        }
        if effects.great_strides() != 0 {
            effect_mod += 100;
        }
        let inner_quiet_mod = 100 + 10 * effects.inner_quiet() as u64;
        (settings.base_quality as u64
            * efficieny_mod
            * condition_mod
            * effect_mod
            * inner_quiet_mod
            / 100000000) as u16
    }

    pub const fn combo_fulfilled(self, combo: Combo) -> bool {
        match self {
            Action::Reflect | Action::MuscleMemory | Action::TrainedEye => {
                matches!(combo, Combo::SynthesisBegin)
            }
            Action::ComboStandardTouch => matches!(combo, Combo::BasicTouch),
            Action::ComboAdvancedTouch => {
                matches!(combo, Combo::StandardTouch)
            }
            Action::ComboRefinedTouch => matches!(combo, Combo::BasicTouch),
            _ => true,
        }
    }

    pub const fn to_combo(self) -> Combo {
        match self {
            Action::BasicTouch => Combo::BasicTouch,
            Action::ComboStandardTouch => Combo::StandardTouch,
            // Observe and StandardTouch unlock the same action (ComboAdvancedTouch)
            Action::Observe => Combo::StandardTouch,
            _ => Combo::None,
        }
    }
}
