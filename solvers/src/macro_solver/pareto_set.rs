use pareto_front::{Dominate, ParetoFront};
use rustc_hash::FxHashMap;
use simulator::{ComboAction, Effects, Settings, SimulationState};

#[derive(Clone, Copy)]
struct GenericValue {
    cp: i16,
    quality: [u16; 2],
    inner_quiet: u8,
}

impl GenericValue {
    pub fn new(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            quality: state.unreliable_quality,
            inner_quiet: state.effects.inner_quiet(),
        }
    }
}

impl Dominate for GenericValue {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.quality[0] >= other.quality[0]
            && self.quality[1] >= other.quality[1]
            && self.inner_quiet >= other.inner_quiet
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GenericKey {
    durability: i8,
    missing_progress: u16,
    effects: Effects,
    combo: Option<ComboAction>,
}

impl GenericKey {
    pub fn new(state: SimulationState) -> Self {
        Self {
            durability: state.durability,
            missing_progress: state.missing_progress,
            effects: state.effects.with_inner_quiet(0), // iq is included in the pareto value
            combo: state.combo,
        }
    }
}

#[derive(Clone, Copy)]
struct ProgressValue {
    cp: i16,
    quality: u16,
    missing_progress: u16,
}

impl ProgressValue {
    pub fn new(state: SimulationState) -> Self {
        Self {
            cp: state.cp,
            quality: state.get_quality(),
            missing_progress: state.missing_progress,
        }
    }
}

impl Dominate for ProgressValue {
    fn dominate(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.quality >= other.quality
            && self.missing_progress <= other.missing_progress
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ProgressKey {
    durability: i8,
    effects: Effects,
    combo: Option<ComboAction>,
}

impl ProgressKey {
    pub fn new(state: SimulationState) -> Self {
        Self {
            durability: state.durability,
            effects: state
                .effects
                .with_inner_quiet(0)
                .with_innovation(0)
                .with_great_strides(0)
                .with_quick_innovation_used(true)
                .with_guard(0),
            combo: state.combo,
        }
    }
}

#[derive(Default)]
pub struct ParetoSet {
    generic_buckets: FxHashMap<GenericKey, ParetoFront<GenericValue>>,
    progress_buckets: FxHashMap<ProgressKey, ParetoFront<ProgressValue>>,
}

impl ParetoSet {
    pub fn insert(&mut self, state: SimulationState, settings: &Settings) -> bool {
        if state.get_quality() < settings.max_quality {
            self.generic_buckets
                .entry(GenericKey::new(state))
                .or_default()
                .push(GenericValue::new(state))
        } else {
            self.progress_buckets
                .entry(ProgressKey::new(state))
                .or_default()
                .push(ProgressValue::new(state))
        }
    }
}

impl Drop for ParetoSet {
    fn drop(&mut self) {
        let generic_pareto_entries: usize = self
            .generic_buckets
            .iter()
            .map(|(_key, value)| value.len())
            .sum();
        dbg!(self.generic_buckets.len(), generic_pareto_entries);
        let progress_pareto_entries: usize = self
            .progress_buckets
            .iter()
            .map(|(_key, value)| value.len())
            .sum();
        dbg!(self.progress_buckets.len(), progress_pareto_entries);
    }
}
