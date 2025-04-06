use raphael_sim::{Combo, Effects, SimulationState};
use rustc_hash::FxHashMap;

use super::{Dominate, ParetoFront};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Value {
    cp: i16,
    quality: u32,
    unreliable_quality: u32,
    inner_quiet: u8,
    durability: i16,
}

impl Value {
    pub fn new(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            quality: state.quality,
            unreliable_quality: state.unreliable_quality,
            inner_quiet: state.effects.inner_quiet(),
            durability: state.durability,
        }
    }
}

impl Dominate for Value {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.quality >= other.quality
            && (self.unreliable_quality >= other.unreliable_quality
                || self.quality >= other.quality + other.unreliable_quality)
            && self.inner_quiet >= other.inner_quiet
            && self.durability >= other.durability
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    progress: u32,
    effects: Effects,
    combo: Combo,
}

impl Key {
    pub fn new(state: SimulationState) -> Self {
        Self {
            progress: state.progress,
            effects: state.effects.with_inner_quiet(0), // iq is included in the pareto value
            combo: state.effects.combo(),
        }
    }
}

#[derive(Default)]
pub struct QualityParetoFront {
    buckets: FxHashMap<Key, ParetoFront<Value>>,
}

impl QualityParetoFront {
    pub fn insert(&mut self, state: SimulationState) -> bool {
        self.buckets
            .entry(Key::new(state))
            .or_default()
            .insert(Value::new(state))
    }
}

impl Drop for QualityParetoFront {
    fn drop(&mut self) {
        let pareto_entries: usize = self.buckets.values().map(ParetoFront::len).sum();
        log::debug!(
            "QualityParetoFront - buckets: {}, entries: {}",
            self.buckets.len(),
            pareto_entries
        );
    }
}
