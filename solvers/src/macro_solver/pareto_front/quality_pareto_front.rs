use rustc_hash::FxHashMap;
use simulator::{Combo, Effects, Settings, SimulationState};

use super::{Dominate, ParetoFront};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Value {
    cp: i16,
    quality: [u16; 2],
    inner_quiet: u8,
}

impl Value {
    pub fn new(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            quality: state.unreliable_quality,
            inner_quiet: state.effects.inner_quiet(),
        }
    }
}

impl Dominate for Value {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.quality[0] >= other.quality[0]
            && self.quality[1] >= other.quality[1]
            && self.inner_quiet >= other.inner_quiet
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    progress: u16,
    durability: i8,
    effects: Effects,
    combo: Combo,
}

impl Key {
    pub fn new(state: SimulationState, settings: &Settings) -> Self {
        let effects = if state.get_quality() >= settings.max_quality {
            state
                .effects
                .with_inner_quiet(0)
                .with_innovation(0)
                .with_great_strides(0)
                .with_guard(0)
                .with_quick_innovation_used(true)
        } else {
            state.effects.with_inner_quiet(0) // iq is included in the pareto value
        };
        Self {
            durability: state.durability,
            progress: state.progress,
            effects,
            combo: state.combo,
        }
    }
}

#[derive(Default)]
pub struct QualityParetoFront {
    buckets: FxHashMap<Key, ParetoFront<Value>>,
}

impl QualityParetoFront {
    pub fn insert(&mut self, state: SimulationState, settings: &Settings) -> bool {
        self.buckets
            .entry(Key::new(state, settings))
            .or_default()
            .insert(Value::new(state))
    }
}

impl Drop for QualityParetoFront {
    fn drop(&mut self) {
        let pareto_entries: usize = self.buckets.values().map(|value| value.len()).sum();
        dbg!(self.buckets.len(), pareto_entries);
    }
}
