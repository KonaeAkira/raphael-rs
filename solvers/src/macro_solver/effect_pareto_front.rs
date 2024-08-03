use pareto_front::{Dominate, ParetoFront};
use rustc_hash::FxHashMap;
use simulator::{Settings, SimulationState};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Value {
    veneration: u8,
    innovation: u8,
    great_strides: u8,
    waste_not: u8,
    manipulation: u8,
    durability: i8,
    cp_mod: i8,
}

impl Value {
    pub fn new(state: SimulationState) -> Self {
        Self {
            veneration: state.effects.veneration(),
            innovation: state.effects.innovation(),
            great_strides: state.effects.great_strides(),
            waste_not: state.effects.waste_not(),
            manipulation: state.effects.manipulation(),
            durability: state.durability,
            cp_mod: (state.cp % 32) as i8,
        }
    }
}

impl Dominate for Value {
    fn dominate(&self, other: &Self) -> bool {
        self.veneration >= other.veneration
            && self.innovation >= other.innovation
            && self.great_strides >= other.great_strides
            && self.waste_not >= other.waste_not
            && self.manipulation >= other.manipulation
            && self.durability >= other.durability
            && self.cp_mod >= other.cp_mod
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    state: SimulationState,
}

impl Key {
    pub fn new(mut state: SimulationState) -> Self {
        state.effects.set_veneration(0);
        state.effects.set_innovation(0);
        state.effects.set_great_strides(0);
        state.effects.set_waste_not(0);
        state.effects.set_manipulation(0);
        state.durability = 0;
        state.cp /= 32;
        Self { state }
    }
}

#[derive(Default)]
pub struct EffectParetoFront {
    buckets: FxHashMap<Key, ParetoFront<Value>>,
}

impl EffectParetoFront {
    pub fn insert(&mut self, state: SimulationState, _settings: &Settings) -> bool {
        self.buckets
            .entry(Key::new(state))
            .or_default()
            .push(Value::new(state))
    }
}

impl Drop for EffectParetoFront {
    fn drop(&mut self) {
        let pareto_entries: usize = self.buckets.values().map(|value| value.len()).sum();
        dbg!(self.buckets.len(), pareto_entries);
    }
}
