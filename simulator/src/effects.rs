use crate::{Action, Settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SingleUse {
    Unavailable,
    Available,
    Active,
}

impl SingleUse {
    pub const fn into_bits(self) -> u8 {
        match self {
            Self::Unavailable => 0,
            Self::Available => 1,
            Self::Active => 2,
        }
    }

    pub const fn from_bits(value: u8) -> Self {
        match value {
            0 => Self::Unavailable,
            1 => Self::Available,
            _ => Self::Active,
        }
    }
}

#[bitfield_struct::bitfield(u32)]
#[derive(PartialEq, Eq, Hash)]
pub struct Effects {
    #[bits(2, default=SingleUse::Available)]
    pub trained_perfection: SingleUse,
    #[bits(2, default=SingleUse::Available)]
    pub heart_and_soul: SingleUse,
    #[bits(1)]
    pub quick_innovation_available: bool,
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
    #[bits(1)]
    _padding: u8,
}

impl Effects {
    pub fn from_settings(settings: &Settings) -> Self {
        Self::default()
            .with_guard(if settings.adversarial { 2 } else { 0 })
            .with_heart_and_soul(if settings.allowed_actions.has(Action::HeartAndSoul) {
                SingleUse::Available
            } else {
                SingleUse::Unavailable
            })
            .with_quick_innovation_available(settings.allowed_actions.has(Action::QuickInnovation))
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
