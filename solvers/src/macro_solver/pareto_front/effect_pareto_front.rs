use rustc_hash::FxHashMap;
use simulator::{Combo, Settings, SimulationState};

use super::{Dominate, ParetoFront};

#[bitfield_struct::bitfield(u32)]
#[derive(PartialEq, Eq)]
struct Value {
    #[bits(3)]
    veneration: u8,
    #[bits(3)]
    innovation: u8,
    #[bits(3)]
    great_strides: u8,
    #[bits(4)]
    waste_not: u8,
    #[bits(4)]
    manipulation: u8,
    #[bits(5)]
    durability: u8,
    #[bits(5)]
    cp_mod: u8,
    #[bits(2)]
    combo: Combo,
    #[bits(3)]
    _padding: u8,
}

impl std::convert::From<SimulationState> for Value {
    fn from(state: SimulationState) -> Self {
        Self::default()
            .with_veneration(state.effects.veneration())
            .with_innovation(state.effects.innovation())
            .with_great_strides(state.effects.great_strides())
            .with_waste_not(state.effects.waste_not())
            .with_manipulation(state.effects.manipulation())
            .with_durability(state.durability as u8 / 5)
            .with_cp_mod((state.cp % 32) as u8)
            .with_combo(state.combo)
    }
}

fn combo_dominate(lhs: Combo, rhs: Combo) -> bool {
    lhs == rhs || rhs == Combo::None
}

impl Dominate for Value {
    fn dominate(&self, other: &Self) -> bool {
        self.veneration() >= other.veneration()
            && self.innovation() >= other.innovation()
            && self.great_strides() >= other.great_strides()
            && self.waste_not() >= other.waste_not()
            && self.manipulation() >= other.manipulation()
            && self.durability() >= other.durability()
            && self.cp_mod() >= other.cp_mod()
            && combo_dominate(self.combo(), other.combo())
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
        state.combo = Combo::None;
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
            .insert(Value::from(state))
    }
}

impl Drop for EffectParetoFront {
    fn drop(&mut self) {
        let pareto_entries: usize = self.buckets.values().map(|value| value.len()).sum();
        dbg!(self.buckets.len(), pareto_entries);
    }
}
