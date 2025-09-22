use crate::{Combo, Settings};

#[bitfield_struct::bitfield(u32, default = false)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Effects {
    #[bits(4)]
    pub inner_quiet: u8,
    #[bits(4)]
    pub waste_not: u8,
    #[bits(3)]
    pub innovation: u8,
    #[bits(3)]
    pub veneration: u8,
    #[bits(2)]
    pub great_strides: u8,
    #[bits(3)]
    pub muscle_memory: u8,
    #[bits(4)]
    pub manipulation: u8,

    pub trained_perfection_available: bool,
    pub heart_and_soul_available: bool,
    pub quick_innovation_available: bool,
    pub trained_perfection_active: bool,
    pub heart_and_soul_active: bool,

    #[bits(2)]
    /// This effect does not exist in-game and is only used by the solver.
    pub special_quality_state: SpecialQualityState,

    #[bits(2)]
    pub combo: Combo,
}

impl Effects {
    /// Effects at synthesis begin
    pub fn initial(settings: &Settings) -> Self {
        let special_quality_state = match settings.adversarial {
            true => SpecialQualityState::AdversarialGuard2,
            false => SpecialQualityState::Normal,
        };
        Self::new()
            .with_special_quality_state(special_quality_state)
            .with_trained_perfection_available(
                settings.is_action_allowed::<crate::actions::TrainedPerfection>(),
            )
            .with_heart_and_soul_available(
                settings.is_action_allowed::<crate::actions::HeartAndSoul>(),
            )
            .with_quick_innovation_available(
                settings.is_action_allowed::<crate::actions::QuickInnovation>(),
            )
            .with_combo(Combo::SynthesisBegin)
    }

    pub const fn progress_modifier(self) -> u32 {
        let mm_mod = 2 * (self.muscle_memory() != 0) as u32;
        let vene_mod = (self.veneration() != 0) as u32;
        50 * (2 + mm_mod + vene_mod)
    }

    pub const fn quality_modifier(self) -> u32 {
        let gs_mod = 2 * (self.great_strides() != 0) as u32;
        let inno_mod = (self.innovation() != 0) as u32;
        5 * (self.inner_quiet() as u32 + 10) * (2 + gs_mod + inno_mod)
    }

    pub const fn adversarial_guard_active(self) -> bool {
        matches!(
            self.special_quality_state(),
            SpecialQualityState::AdversarialGuard | SpecialQualityState::AdversarialGuard2
        )
    }

    pub const fn quality_actions_allowed(self) -> bool {
        !matches!(self.special_quality_state(), SpecialQualityState::Forbidden)
    }

    #[must_use]
    pub const fn tick_down(self) -> Self {
        // Calculate the decrement bits for all ticking effects.
        // The decrement contains the least-significant bit of all active ticking effects.
        let effects_tick = {
            let mask_0 = self.into_bits() & EFFECTS_BIT_0;
            let mask_1 = (self.into_bits() & EFFECTS_BIT_1) >> 1;
            let mask_2 = (self.into_bits() & EFFECTS_BIT_2) >> 2;
            let mask_3 = (self.into_bits() & EFFECTS_BIT_3) >> 3;
            mask_0 | mask_1 | mask_2 | mask_3
        };
        Self::from_bits(self.into_bits() - effects_tick)
    }

    /// Removes all effects that are only relevant for Quality.
    #[must_use]
    pub const fn strip_quality_effects(self) -> Self {
        self.with_special_quality_state(SpecialQualityState::Forbidden)
            .with_inner_quiet(0)
            .with_innovation(0)
            .with_great_strides(0)
            .with_quick_innovation_available(false)
    }
}

const EFFECTS_BIT_0: u32 = Effects::new()
    .with_waste_not(1)
    .with_innovation(1)
    .with_veneration(1)
    .with_great_strides(1)
    .with_muscle_memory(1)
    .with_manipulation(1)
    .into_bits();

const EFFECTS_BIT_1: u32 = Effects::new()
    .with_special_quality_state(SpecialQualityState::AdversarialGuard)
    .with_waste_not(2)
    .with_innovation(2)
    .with_veneration(2)
    .with_great_strides(2)
    .with_muscle_memory(2)
    .with_manipulation(2)
    .into_bits();

const EFFECTS_BIT_2: u32 = Effects::new()
    .with_waste_not(4)
    .with_innovation(4)
    .with_veneration(4)
    .with_muscle_memory(4)
    .with_manipulation(4)
    .into_bits();

const EFFECTS_BIT_3: u32 = Effects::new()
    .with_waste_not(8)
    .with_manipulation(8)
    .into_bits();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SpecialQualityState {
    Forbidden,         // Quality-increasing actions are forbidden
    Normal,            // Default mode of operation
    AdversarialGuard,  // Guarded from adversarial mode (expires in 1 turn)
    AdversarialGuard2, // Guard from adversarial mode (expires in 2 turns)
}

impl SpecialQualityState {
    pub const fn into_bits(self) -> u8 {
        match self {
            Self::Forbidden => 0,
            Self::Normal => 1,
            Self::AdversarialGuard => 2,
            Self::AdversarialGuard2 => 3,
        }
    }

    pub const fn from_bits(bits: u8) -> Self {
        match bits {
            0 => Self::Forbidden,
            1 => Self::Normal,
            2 => Self::AdversarialGuard,
            _ => Self::AdversarialGuard2,
        }
    }
}
