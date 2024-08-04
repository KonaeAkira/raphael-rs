#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
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
            1 => Self::Available,
            2 => Self::Active,
            _ => Self::Unavailable,
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
    pub quick_innovation_used: bool,
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
    pub fn tick_down(&mut self) {
        self.set_waste_not(self.waste_not().saturating_sub(1));
        self.set_innovation(self.innovation().saturating_sub(1));
        self.set_veneration(self.veneration().saturating_sub(1));
        self.set_great_strides(self.great_strides().saturating_sub(1));
        self.set_muscle_memory(self.muscle_memory().saturating_sub(1));
        self.set_manipulation(self.manipulation().saturating_sub(1));
        self.set_guard(self.guard().saturating_sub(1));
    }
}
