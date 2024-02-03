#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Effects {
    pub inner_quiet: i8,
    pub waste_not: i8,
    pub innovation: i8,
    pub veneration: i8,
    pub great_strides: i8,
    pub muscle_memory: i8,
    pub manipulation: i8,
}

impl Effects {
    pub fn tick_down(&mut self) {
        self.waste_not = std::cmp::max(0, self.waste_not - 1);
        self.innovation = std::cmp::max(0, self.innovation - 1);
        self.veneration = std::cmp::max(0, self.veneration - 1);
        self.great_strides = std::cmp::max(0, self.great_strides - 1);
        self.muscle_memory = std::cmp::max(0, self.muscle_memory - 1);
        self.manipulation = std::cmp::max(0, self.manipulation - 1);
    }
}
