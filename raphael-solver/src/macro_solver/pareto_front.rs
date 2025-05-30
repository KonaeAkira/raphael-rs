use raphael_sim::{Effects, SimulationState};
use rustc_hash::FxHashMap;

const EFFECTS_VALUE_MASK: u32 = Effects::new()
    .with_inner_quiet(1)
    .with_manipulation(3)
    .with_waste_not(3)
    .with_great_strides(1)
    .with_veneration(3)
    .with_innovation(3)
    .into_bits();

const EFFECTS_KEY_MASK: u32 = !EFFECTS_VALUE_MASK;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct Key {
    progress: u32,
    cp: u16,
    durability: u16,
    effects: u32,
}

impl From<&SimulationState> for Key {
    fn from(state: &SimulationState) -> Self {
        Self {
            progress: state.progress,
            cp: state.cp.next_multiple_of(64),
            durability: state.durability.next_multiple_of(15),
            effects: state.effects.into_bits() & EFFECTS_KEY_MASK,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Value {
    cp: u16,
    durability: u16,
    quality: u32,
    unreliable_quality: u32,
    effects: Effects,
}

impl From<&SimulationState> for Value {
    fn from(state: &SimulationState) -> Self {
        Self {
            cp: state.cp,
            durability: state.durability,
            quality: state.quality,
            unreliable_quality: state.unreliable_quality,
            effects: Effects::from_bits(state.effects.into_bits() & EFFECTS_VALUE_MASK),
        }
    }
}

impl Value {
    fn dominates(&self, other: &Self) -> bool {
        self.cp >= other.cp
            && self.durability >= other.durability
            && self.quality_dominates(other)
            && self.effect_dominates(other)
    }

    #[inline]
    fn quality_dominates(&self, other: &Self) -> bool {
        let adversarial_dominates = self.unreliable_quality >= other.unreliable_quality
            || self.quality >= other.quality + other.unreliable_quality;
        self.quality >= other.quality && adversarial_dominates
    }

    #[inline]
    fn effect_dominates(&self, other: &Self) -> bool {
        self.effects.inner_quiet() >= other.effects.inner_quiet()
            && self.effects.innovation() >= other.effects.innovation()
            && self.effects.veneration() >= other.effects.veneration()
            && self.effects.great_strides() >= other.effects.great_strides()
            && self.effects.manipulation() >= other.effects.manipulation()
            && self.effects.waste_not() >= other.effects.waste_not()
    }
}

#[derive(Default)]
pub struct ParetoFront {
    buckets: FxHashMap<Key, Vec<Value>>,
}

impl ParetoFront {
    pub fn insert(&mut self, state: SimulationState) -> bool {
        let bucket = self.buckets.entry(Key::from(&state)).or_default();
        let new_value = Value::from(&state);
        let is_dominated = bucket.iter().any(|value| value.dominates(&new_value));
        if is_dominated {
            false
        } else {
            bucket.retain(|value| !new_value.dominates(value));
            bucket.push(new_value);
            true
        }
    }

    /// Returns the sum of the squared size of all Pareto buckets.
    /// This is a useful performance metric because the total insertion cost of each Pareto bucket scales with the square of its size.
    pub fn buckets_squared_size_sum(&self) -> usize {
        self.buckets
            .values()
            .map(|bucket| bucket.len() * bucket.len())
            .sum()
    }
}

impl Drop for ParetoFront {
    fn drop(&mut self) {
        let largest_bucket = self.buckets.iter().max_by_key(|(_key, elems)| elems.len());
        let pareto_entries: usize = self.buckets.values().map(Vec::len).sum();
        log::debug!(
            "ParetoFront - buckets: {}, entries: {}, largest_bucket_len: {}",
            self.buckets.len(),
            pareto_entries,
            largest_bucket.map_or(0, |(_key, elems)| elems.len())
        );
        log::trace!(
            "ParetoFront - largest_bucket_key: {:?}",
            largest_bucket.map_or(Key::default(), |(key, _elems)| *key)
        );
    }
}
