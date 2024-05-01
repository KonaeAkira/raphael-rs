#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Effects {
    pub inner_quiet: u8,
    pub waste_not: u8,
    pub innovation: u8,
    pub veneration: u8,
    pub great_strides: u8,
    pub muscle_memory: u8,
    pub manipulation: u8,
}

impl Effects {
    pub fn tick_down(&mut self) {
        self.waste_not = self.waste_not.saturating_sub(1);
        self.innovation = self.innovation.saturating_sub(1);
        self.veneration = self.veneration.saturating_sub(1);
        self.great_strides = self.great_strides.saturating_sub(1);
        self.muscle_memory = self.muscle_memory.saturating_sub(1);
        self.manipulation = self.manipulation.saturating_sub(1);
    }
}
