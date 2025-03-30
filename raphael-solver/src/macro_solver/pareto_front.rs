use raphael_sim::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ParetoValue {
    cp: i16,
    progress: u16,
    quality: u16,
    unreliable_quality: u16,
    durability: i8,
}

impl ParetoValue {
    pub fn new(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            progress: state.progress,
            quality: state.quality,
            unreliable_quality: state.unreliable_quality,
            durability: state.durability,
        }
    }

    /// The "weight" of a state is an arbitrary metric that correlates to the pareto-optimality of states.
    /// A state with a higher "weight" can never be dominated by a state with a lower "weight".
    pub fn weight(state: &SimulationState) -> u32 {
        state.cp as u32
            + state.durability as u32
            + state.progress as u32
            + state.quality as u32
            + state.unreliable_quality as u32
    }
}

impl Dominate for ParetoValue {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.progress >= other.progress
            && self.quality >= other.quality
            && (self.unreliable_quality >= other.unreliable_quality
                || self.quality >= other.quality + other.unreliable_quality)
            && self.durability >= other.durability
    }
}

pub trait Dominate {
    fn dominate(&self, other: &Self) -> bool;
}

pub struct ParetoFront<T: Clone + Copy + Dominate> {
    values: Vec<T>,
}

impl<T: Clone + Copy + Dominate> ParetoFront<T> {
    fn is_dominated(&self, new_value: &T) -> bool {
        self.values.iter().any(|value| value.dominate(new_value))
    }

    pub fn insert(&mut self, new_value: T) -> bool {
        if !self.is_dominated(&new_value) {
            self.values.retain(|value| !new_value.dominate(value));
            self.values.push(new_value);
            true
        } else {
            false
        }
    }
}

impl<T: Clone + Copy + Dominate> Default for ParetoFront<T> {
    fn default() -> Self {
        Self { values: Vec::new() }
    }
}
