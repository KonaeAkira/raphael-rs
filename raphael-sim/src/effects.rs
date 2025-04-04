use crate::Settings;

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
    #[bits(3)]
    pub great_strides: u8,
    #[bits(3)]
    pub muscle_memory: u8,
    #[bits(4)]
    pub manipulation: u8,
    #[bits(2)]
    pub guard: u8,

    pub trained_perfection_available: bool,
    pub heart_and_soul_available: bool,
    pub quick_innovation_available: bool,
    pub trained_perfection_active: bool,
    pub heart_and_soul_active: bool,

    #[bits(1)]
    _padding: u8,
}

impl Effects {
    pub fn initial(settings: &Settings) -> Self {
        Self::new()
            .with_guard(if settings.adversarial { 2 } else { 0 })
            .with_trained_perfection_available(
                settings.is_action_allowed::<crate::actions::TrainedPerfection>(),
            )
            .with_heart_and_soul_available(
                settings.is_action_allowed::<crate::actions::HeartAndSoul>(),
            )
            .with_quick_innovation_available(
                settings.is_action_allowed::<crate::actions::QuickInnovation>(),
            )
    }

    pub fn tick_down(&mut self) {
        let mut effect_tick = 0;
        if self.waste_not() != 0 {
            effect_tick |= const { Self::from_bits(0).with_waste_not(1).into_bits() };
        }
        if self.innovation() != 0 {
            effect_tick |= const { Self::from_bits(0).with_innovation(1).into_bits() };
        }
        if self.veneration() != 0 {
            effect_tick |= const { Self::from_bits(0).with_veneration(1).into_bits() };
        }
        if self.great_strides() != 0 {
            effect_tick |= const { Self::from_bits(0).with_great_strides(1).into_bits() };
        }
        if self.muscle_memory() != 0 {
            effect_tick |= const { Self::from_bits(0).with_muscle_memory(1).into_bits() };
        }
        if self.manipulation() != 0 {
            effect_tick |= const { Self::from_bits(0).with_manipulation(1).into_bits() };
        }
        if self.guard() != 0 {
            effect_tick |= const { Self::from_bits(0).with_guard(1).into_bits() };
        }
        self.0 -= effect_tick;
    }
}
