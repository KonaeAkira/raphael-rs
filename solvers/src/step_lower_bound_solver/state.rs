use simulator::{Combo, Effects, SimulationState, SingleUse};

pub trait ReducedState: Clone + Copy + PartialEq + Eq + std::hash::Hash {
    fn steps_budget(&self) -> u8;
    fn from_state(state: SimulationState, steps_budget: u8) -> Self;
    fn to_state(self) -> SimulationState;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedStateWithDurability {
    steps_budget: u8,
    durability: i8,
    combo: Combo,
    inner_quiet: u8,
    innovation: u8,
    veneration: u8,
    great_strides: u8,
    muscle_memory: u8,
    waste_not: u8,
    trained_perfection: SingleUse,
    heart_and_soul: SingleUse,
    quick_innovation_used: bool,
}

impl ReducedState for ReducedStateWithDurability {
    fn steps_budget(&self) -> u8 {
        self.steps_budget
    }

    fn from_state(state: SimulationState, steps_budget: u8) -> Self {
        Self {
            steps_budget,
            durability: state.durability,
            combo: match state.combo {
                Combo::None => Combo::None,
                Combo::SynthesisBegin => Combo::SynthesisBegin,
                // Can't optimize this combo away because there is no replacement for RefinedTouch
                Combo::BasicTouch => Combo::BasicTouch,
                // AdvancedTouch replaces ComboAdvancedTouch (no CP cost)
                Combo::StandardTouch => Combo::None,
            },
            inner_quiet: state.effects.inner_quiet(),
            innovation: state.effects.innovation(),
            veneration: state.effects.veneration(),
            great_strides: state.effects.great_strides(),
            muscle_memory: state.effects.muscle_memory(),
            waste_not: state.effects.waste_not(),
            trained_perfection: state.effects.trained_perfection(),
            heart_and_soul: state.effects.heart_and_soul(),
            quick_innovation_used: state.effects.quick_innovation_used(),
        }
    }

    fn to_state(self) -> SimulationState {
        SimulationState {
            durability: self.durability,
            cp: 1000,
            progress: 0,
            unreliable_quality: [0, 0],
            effects: Effects::new()
                .with_inner_quiet(self.inner_quiet)
                .with_innovation(self.innovation)
                .with_veneration(self.veneration)
                .with_great_strides(self.great_strides)
                .with_muscle_memory(self.muscle_memory)
                .with_waste_not(self.waste_not)
                .with_manipulation(1) // storing manipulation in the reduced state leads to too many state combinations
                .with_trained_perfection(self.trained_perfection)
                .with_heart_and_soul(self.heart_and_soul)
                .with_quick_innovation_used(self.quick_innovation_used)
                .with_guard(1),
            combo: self.combo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedStateWithoutDurability {
    steps_budget: u8,
    combo: Combo,
    inner_quiet: u8,
    innovation: u8,
    veneration: u8,
    great_strides: u8,
    muscle_memory: u8,
    heart_and_soul: SingleUse,
    quick_innovation_used: bool,
}

impl ReducedState for ReducedStateWithoutDurability {
    fn steps_budget(&self) -> u8 {
        self.steps_budget
    }

    fn from_state(state: SimulationState, steps_budget: u8) -> Self {
        Self {
            steps_budget,
            combo: match state.combo {
                Combo::None => Combo::None,
                Combo::SynthesisBegin => Combo::SynthesisBegin,
                // StandardTouch replaces ComboStandardTouch (no CP cost, next combo not needed)
                // PreparatoryTouch replaces RefinedTouch (no CP cost, no durability cost)
                Combo::BasicTouch => Combo::None,
                // AdvancedTouch replaces ComboAdvancedTouch (no CP cost)
                Combo::StandardTouch => Combo::None,
            },
            inner_quiet: state.effects.inner_quiet(),
            innovation: state.effects.innovation(),
            veneration: state.effects.veneration(),
            great_strides: state.effects.great_strides(),
            muscle_memory: state.effects.muscle_memory(),
            heart_and_soul: state.effects.heart_and_soul(),
            quick_innovation_used: state.effects.quick_innovation_used(),
        }
    }

    fn to_state(self) -> SimulationState {
        SimulationState {
            durability: i8::MAX,
            cp: 1000,
            progress: 0,
            unreliable_quality: [0, 0],
            effects: Effects::new()
                .with_inner_quiet(self.inner_quiet)
                .with_innovation(self.innovation)
                .with_veneration(self.veneration)
                .with_great_strides(self.great_strides)
                .with_muscle_memory(self.muscle_memory)
                .with_heart_and_soul(self.heart_and_soul)
                .with_quick_innovation_used(self.quick_innovation_used)
                .with_guard(1),
            combo: self.combo,
        }
    }
}
