use raphael_sim::{Effects, SimulationState};
use rustc_hash::FxHashMap;

// It is important that this mask doesn't use any effect to its full bit range.
// Otherwise, `Value::effect_dominates` will break.
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

#[bitfield_struct::bitfield(u128, default = false)]
#[derive(PartialEq, Eq)]
struct Value {
    cp: u16,
    durability: u16,
    quality: u32,
    unreliable_quality: u32,
    effects: u32,
}

const VALUE_DIFF_GUARD: u128 = Value::new()
    .with_cp(1 << 15)
    .with_durability(1 << 15)
    .with_quality(1 << 31)
    .with_unreliable_quality(1 << 31)
    .with_effects(EFFECTS_KEY_MASK)
    .into_bits();

impl From<&SimulationState> for Value {
    fn from(state: &SimulationState) -> Self {
        Self::new()
            .with_cp(state.cp)
            .with_durability(state.durability)
            .with_quality(state.quality)
            .with_unreliable_quality(state.quality + state.unreliable_quality)
            .with_effects(state.effects.into_bits() & EFFECTS_VALUE_MASK)
    }
}

impl Value {
    #[inline]
    /// `A` dominates `B` if every member of `A` is geq the corresponding member in `B`.
    const fn dominates(&self, other: &Self) -> bool {
        let guarded_value = VALUE_DIFF_GUARD | self.into_bits();
        (guarded_value - other.into_bits()) & VALUE_DIFF_GUARD == VALUE_DIFF_GUARD
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
        if !is_dominated {
            bucket.retain(|value| !new_value.dominates(value));
            bucket.push(new_value);
        }
        !is_dominated
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
