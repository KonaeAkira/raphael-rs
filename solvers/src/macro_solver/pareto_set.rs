use pareto_front::{Dominate, ParetoFront};
use rustc_hash::FxHashMap;
use simulator::{ComboAction, Effects, SimulationState};

#[derive(Clone, Copy)]
struct Value {
    cp: i16,
    missing_quality: [u16; 2],
}

impl Value {
    pub fn new(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            missing_quality: state.unreliable_quality,
        }
    }
}

impl Dominate for Value {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.missing_quality[0] <= other.missing_quality[0]
            && self.missing_quality[1] <= other.missing_quality[1]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    durability: i8,
    missing_progress: u16,
    effects: Effects,
    combo: Option<ComboAction>,
}

impl Key {
    pub fn new(state: SimulationState) -> Self {
        let effects = match state.get_missing_quality() == 0 {
            true => {
                // Ignore effects that are only relevant for Quality when Quality is already maxed out
                state
                    .effects
                    .with_inner_quiet(0)
                    .with_innovation(0)
                    .with_great_strides(0)
            }
            false => state.effects,
        };
        Self {
            durability: state.durability,
            missing_progress: state.missing_progress,
            effects,
            combo: state.combo,
        }
    }
}

#[derive(Default)]
pub struct ParetoSet {
    buckets: FxHashMap<Key, ParetoFront<Value>>,
}

impl ParetoSet {
    pub fn insert(&mut self, state: SimulationState) -> bool {
        self.buckets
            .entry(Key::new(state))
            .or_default()
            .push(Value::new(state))
    }
}

impl Drop for ParetoSet {
    fn drop(&mut self) {
        let pareto_entries: usize = self.buckets.iter().map(|bucket| bucket.1.len()).sum();
        dbg!(self.buckets.len(), pareto_entries);
    }
}
