use pareto_front::{Dominate, ParetoFront};
use rustc_hash::FxHashMap;
use simulator::{ComboAction, Effects, SimulationState};

#[derive(Clone, Copy)]
struct Value {
    cp: i16,
    missing_progress: u16,
    missing_quality: u16,
}

impl Dominate for Value {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.missing_progress <= other.missing_progress
            && self.missing_quality <= other.missing_quality
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct HashKey {
    durability: i8,
    effects: Effects,
    combo: Option<ComboAction>,
}

#[derive(Default)]
pub struct ParetoSet {
    buckets: FxHashMap<HashKey, ParetoFront<Value>>,
}

impl ParetoSet {
    pub fn insert(&mut self, state: SimulationState) -> bool {
        let effects = match state.missing_quality == 0 {
            // Ignore effects that are only relevant for Quality when Quality is already maxed out
            true => state
                .effects
                .with_inner_quiet(0)
                .with_innovation(0)
                .with_great_strides(0),
            false => state.effects,
        };
        let hash_key = HashKey {
            durability: state.durability,
            effects,
            combo: state.combo,
        };
        let value = Value {
            cp: state.cp,
            missing_progress: state.missing_progress,
            missing_quality: state.missing_quality,
        };
        self.buckets.entry(hash_key).or_default().push(value)
    }
}
