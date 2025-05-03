use crate::{Combo, Settings};

#[bitfield_struct::bitfield(u32, default = false)]
#[derive(PartialEq, Eq, Hash)]
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

    pub adversarial_guard: bool,
    pub allow_quality_actions: bool,

    #[bits(2)]
    pub combo: Combo,
}

impl Effects {
    /// Effects at synthesis begin
    pub fn initial(settings: &Settings) -> Self {
        Self::new()
            .with_adversarial_guard(settings.adversarial)
            .with_allow_quality_actions(true)
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

    pub fn tick_down(&mut self) {
        let mask_0 = self.0 & MASK_0;
        let mask_1 = (self.0 & MASK_1) >> 1;
        let mask_2 = (self.0 & MASK_2) >> 2;
        let mask_3 = (self.0 & MASK_3) >> 3;
        let effect_tick = mask_0 | mask_1 | mask_2 | mask_3;
        self.0 -= effect_tick;
        if self.combo() != Combo::SynthesisBegin {
            // Guard does not wear off because the first condition is guaranteed to be Normal
            self.set_adversarial_guard(false);
        }
    }
}

const MASK_0: u32 = Effects::new()
    .with_waste_not(1)
    .with_innovation(1)
    .with_veneration(1)
    .with_great_strides(1)
    .with_muscle_memory(1)
    .with_manipulation(1)
    .into_bits();

const MASK_1: u32 = Effects::new()
    .with_waste_not(2)
    .with_innovation(2)
    .with_veneration(2)
    .with_great_strides(2)
    .with_muscle_memory(2)
    .with_manipulation(2)
    .into_bits();

const MASK_2: u32 = Effects::new()
    .with_waste_not(4)
    .with_innovation(4)
    .with_veneration(4)
    .with_muscle_memory(4)
    .with_manipulation(4)
    .into_bits();

const MASK_3: u32 = Effects::new()
    .with_waste_not(8)
    .with_manipulation(8)
    .into_bits();
