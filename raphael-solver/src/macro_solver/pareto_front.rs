use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::{cmp::Reverse, collections::hash_map::Entry};

use raphael_sim::{Effects, SimulationState};

use crate::macro_solver::search_queue::SearchNode;

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
    effects: u32,
}

impl From<&SimulationState> for Key {
    fn from(state: &SimulationState) -> Self {
        Self {
            progress: state.progress,
            effects: state.effects.into_bits() & EFFECTS_KEY_MASK,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Value(wide::u32x4);

impl Value {
    pub const GUARD: wide::u32x4 = wide::u32x4::new([
        0x80008000,       // CP and Durability
        0x80000000,       // Quality
        0x80000000,       // Unreliable quality
        EFFECTS_KEY_MASK, // Effects
    ]);

    /// `A` dominates `B` if every member of `A` is geq the corresponding member in `B`.
    fn dominates(&self, other: &Self) -> bool {
        let guarded_value = Self::GUARD | self.0;
        (guarded_value - other.0) & Self::GUARD == Self::GUARD
    }
}

impl From<&SimulationState> for Value {
    fn from(state: &SimulationState) -> Self {
        Self(wide::u32x4::new([
            (u32::from(state.cp) << 16) + u32::from(state.durability),
            state.quality,
            state.quality + state.unreliable_quality,
            state.effects.into_bits() & EFFECTS_VALUE_MASK,
        ]))
    }
}

#[derive(Default)]
pub struct ParetoFront {
    buckets: FxHashMap<Key, Vec<Value>>,
}

impl ParetoFront {
    pub fn par_insert(
        &mut self,
        search_nodes: Vec<SearchNode>,
    ) -> impl Iterator<Item = SearchNode> {
        let mut insertion_tasks: FxHashMap<Key, InsertionTask> = FxHashMap::default();
        for node in search_nodes {
            let key = Key::from(&node.state);
            match insertion_tasks.entry(key) {
                Entry::Occupied(occupied_entry) => {
                    occupied_entry.into_mut().search_nodes.push(node)
                }
                Entry::Vacant(vacant_entry) => {
                    let mut insertion_task = InsertionTask {
                        existing_values: Vec::new(),
                        search_nodes: vec![node],
                    };
                    std::mem::swap(
                        self.buckets.entry(key).or_default(),
                        &mut insertion_task.existing_values,
                    );
                    vacant_entry.insert(insertion_task);
                }
            }
        }
        let mut finished_tasks = insertion_tasks
            .into_par_iter()
            .map(|(key, task)| (key, task.execute()))
            .collect_vec_list();
        for (key, task) in finished_tasks.iter_mut().flatten() {
            std::mem::swap(
                self.buckets.entry(*key).or_default(),
                &mut task.existing_values,
            );
        }
        finished_tasks
            .into_iter()
            .flatten()
            .flat_map(|(_key, task)| task.search_nodes)
    }
}

struct InsertionTask {
    existing_values: Vec<Value>,
    search_nodes: Vec<SearchNode>,
}

impl InsertionTask {
    fn execute(mut self) -> Self {
        self.search_nodes
            .sort_unstable_by_key(|node| Reverse(pareto_weight(&node.state)));
        self.search_nodes
            .retain(|node| check_and_insert_node(&mut self.existing_values, node));
        self
    }
}

fn pareto_weight(state: &SimulationState) -> u32 {
    state.cp as u32
        + state.durability as u32
        + state.quality
        + state.unreliable_quality
        + state.effects.into_bits()
}

fn check_and_insert_node(values: &mut Vec<Value>, node: &SearchNode) -> bool {
    let new_value = Value::from(&node.state);
    let is_dominated = values.iter().any(|value| value.dominates(&new_value));
    if !is_dominated {
        values.retain(|value| !new_value.dominates(value));
        values.push(new_value);
    }
    !is_dominated
}
