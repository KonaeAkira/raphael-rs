use pareto_front::{Dominate, ParetoFront};
use rustc_hash::FxHashMap;
use simulator::{ComboAction, Effects, Settings, SimulationState};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Value {
    cp: i16,
    missing_progress: u16,
    quality: [u16; 2],
    inner_quiet: u8,
}

impl Value {
    pub fn new(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            missing_progress: state.missing_progress,
            quality: state.unreliable_quality,
            inner_quiet: state.effects.inner_quiet(),
        }
    }
}

impl Dominate for Value {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.missing_progress <= other.missing_progress
            && self.quality[0] >= other.quality[0]
            && self.quality[1] >= other.quality[1]
            && self.inner_quiet >= other.inner_quiet
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    durability: i8,
    effects: Effects,
    combo: Option<ComboAction>,
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
    pub fn insert(&mut self, state: SimulationState, settings: &Settings) -> bool {
        self.buckets
            .entry(Key::new(state, settings))
            .or_default()
            .push(Value::new(state))
    }

    pub fn contains(&self, state: SimulationState, settings: &Settings) -> bool {
        match self.buckets.get(&Key::new(state, settings)) {
            Some(pareto_front) => pareto_front.as_slice().contains(&Value::new(state)),
            None => false,
        }
    }
}

// impl Drop for ParetoSet {
//     fn drop(&mut self) {
//         let pareto_entries: usize = self
//             .generic_buckets
//             .iter()
//             .map(|(_key, value)| value.len())
//             .sum();
//         dbg!(self.buckets.len(), pareto_entries);
//     }
// }
