#[bitfield_struct::bitfield(u32)]
#[derive(PartialEq, Eq, Hash)]
pub struct Effects {
    #[bits(4)]
    pub inner_quiet: u8,
    #[bits(4)]
    pub waste_not: u8,
    #[bits(4)]
    pub innovation: u8,
    #[bits(4)]
    pub veneration: u8,
    #[bits(4)]
    pub great_strides: u8,
    #[bits(4)]
    pub muscle_memory: u8,
    #[bits(4)]
    pub manipulation: u8,
    #[bits(4)]
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
    }
}
