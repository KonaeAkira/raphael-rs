use raphael_sim::*;
use rustc_hash::FxHashMap;

use super::{Dominate, ParetoFront};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Value {
    cp: i16,
    progress: u16,
    quality: u16,
    unreliable_quality: u16,
    durability: i8,
}

impl Value {
    pub fn new(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            progress: state.progress,
            quality: state.quality,
            unreliable_quality: state.unreliable_quality,
            durability: state.durability,
        }
    }
}

impl Dominate for Value {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.progress >= other.progress
            && self.quality >= other.quality
            && (self.unreliable_quality >= other.unreliable_quality
                || self.quality >= other.quality + other.unreliable_quality)
            && self.durability >= other.durability
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    effects: Effects,
}

impl Key {
    pub fn new(state: SimulationState) -> Self {
        #[cfg(test)]
        assert!(state.combo == Combo::None);
        Self {
            effects: state.effects,
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
