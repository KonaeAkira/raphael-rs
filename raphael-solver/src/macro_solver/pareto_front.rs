use raphael_sim::{Effects, SimulationState};
use rustc_hash::FxHashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    progress: u32,
    quality_div: u16,
    cp_div: u16,
    durability_div: u16,
    manipulation_div: u8,
    waste_not_div: u8,
    heart_and_soul_available: bool,
    quick_innovation_available: bool,
    trained_perfection_available: bool,
    trained_perfection_active: bool,
}

impl From<&SimulationState> for Key {
    fn from(state: &SimulationState) -> Self {
        Self {
            progress: state.progress,
            quality_div: (state.quality / 2048) as u16,
            cp_div: state.cp / 32,
            durability_div: state.durability / 15,
            manipulation_div: state.effects.manipulation() / 4,
            waste_not_div: state.effects.waste_not() / 4,
            heart_and_soul_available: state.effects.heart_and_soul_available(),
            quick_innovation_available: state.effects.quick_innovation_available(),
            trained_perfection_available: state.effects.trained_perfection_available(),
            trained_perfection_active: state.effects.trained_perfection_active(),
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
            effects: state.effects,
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
        let allow_quality_actions_dominates =
            self.effects.allow_quality_actions() || !other.effects.allow_quality_actions();
        let adversarial_guard_dominates =
            self.effects.adversarial_guard() || !other.effects.adversarial_guard();
        self.effects.inner_quiet() >= other.effects.inner_quiet()
            && self.effects.muscle_memory() >= other.effects.muscle_memory()
            && self.effects.innovation() >= other.effects.innovation()
            && self.effects.veneration() >= other.effects.veneration()
            && self.effects.great_strides() >= other.effects.great_strides()
            && self.effects.manipulation() >= other.effects.manipulation()
            && self.effects.waste_not() >= other.effects.waste_not()
            && allow_quality_actions_dominates
            && adversarial_guard_dominates
    }
}

#[derive(Default)]
pub struct ParetoFront {
    buckets: FxHashMap<Key, Vec<Value>>,
}

impl ParetoFront {
    pub fn insert(&mut self, state: SimulationState) -> bool {
        #[cfg(test)]
        {
            // These effects should not appear and are therefore not taken into account when determining Pareto-optimality.
            assert_eq!(state.effects.heart_and_soul_active(), false);
            assert_eq!(state.effects.combo(), raphael_sim::Combo::None);
        }
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
}

impl Drop for ParetoFront {
    fn drop(&mut self) {
        let pareto_entries: usize = self.buckets.values().map(Vec::len).sum();
        log::debug!(
            "ParetoFront - buckets: {}, entries: {}",
            self.buckets.len(),
            pareto_entries
        );
    }
}
