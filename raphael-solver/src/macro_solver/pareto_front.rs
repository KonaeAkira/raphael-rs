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

    fn cp(&self) -> u16 {
        (self.0.as_array_ref()[0] >> 16) as u16
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

struct IntermediateNode {
    partition_point: u16,
    lhs: Box<TreeNode>,
    rhs: Box<TreeNode>,
}

struct LeafNode {
    range_min: u16,
    range_max: u16,
    values: Vec<Value>,
}

enum TreeNode {
    Intermediate(IntermediateNode),
    Leaf(LeafNode),
}

impl Default for TreeNode {
    fn default() -> Self {
        Self::Leaf(LeafNode {
            range_min: u16::MIN,
            range_max: u16::MAX,
            values: Vec::new(),
        })
    }
}

#[derive(Default)]
pub struct ParetoFront {
    buckets: FxHashMap<Key, TreeNode>,
}

impl ParetoFront {
    pub fn par_insert(&mut self, search_nodes: Vec<SearchNode>) -> Vec<SearchNode> {
        let mut insertion_tasks: FxHashMap<Key, InsertionTask> = FxHashMap::default();
        for node in search_nodes {
            let key = Key::from(&node.state);
            match insertion_tasks.entry(key) {
                Entry::Occupied(occupied_entry) => {
                    occupied_entry.into_mut().search_nodes.push(node)
                }
                Entry::Vacant(vacant_entry) => {
                    let segment_tree_root = self.buckets.entry(key).or_default();
                    let mut insertion_task = InsertionTask {
                        segment_tree_root: TreeNode::default(),
                        search_nodes: vec![node],
                    };
                    // The task takes ownership of the current tree root and gives ownership back when the task is complete.
                    std::mem::swap(segment_tree_root, &mut insertion_task.segment_tree_root);
                    vacant_entry.insert(insertion_task);
                }
            }
        }
        let finished_tasks = insertion_tasks
            .into_par_iter()
            .map(|(key, task)| (key, task.execute()))
            .collect_vec_list();
        let mut result = Vec::new();
        for (key, task) in finished_tasks.into_iter().flatten() {
            // Give back ownership of the tree root.
            self.buckets.insert(key, task.segment_tree_root);
            result.extend(task.search_nodes);
        }
        result
    }
}

struct InsertionTask {
    segment_tree_root: TreeNode,
    search_nodes: Vec<SearchNode>,
}

impl InsertionTask {
    fn execute(mut self) -> Self {
        self.search_nodes
            .sort_unstable_by_key(|node| Reverse(pareto_weight(&node.state)));
        self.search_nodes
            .retain(|node| insert_to_tree(&mut self.segment_tree_root, node));
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

fn insert_to_tree(mut tree_node: &mut TreeNode, search_node: &SearchNode) -> bool {
    const MAX_LEAF_SIZE: usize = 200;
    let new_value = Value::from(&search_node.state);
    while let TreeNode::Intermediate(intermediate) = tree_node {
        if new_value.cp() < intermediate.partition_point {
            tree_node = intermediate.lhs.as_mut();
        } else {
            tree_node = intermediate.rhs.as_mut();
        }
    }
    if let TreeNode::Leaf(leaf) = tree_node {
        let is_dominated = leaf.values.iter().any(|value| value.dominates(&new_value));
        if !is_dominated {
            leaf.values.retain(|value| !new_value.dominates(value));
            leaf.values.push(new_value);
        }
        if leaf.values.len() > MAX_LEAF_SIZE && leaf.range_min + 1 != leaf.range_max {
            leaf.values.sort_unstable_by_key(Value::cp);
            let (lhs_values, rhs_values) = leaf.values.split_at(MAX_LEAF_SIZE / 2);
            let partition_point = rhs_values[0].cp();
            *tree_node = TreeNode::Intermediate(IntermediateNode {
                partition_point,
                lhs: Box::new(TreeNode::Leaf(LeafNode {
                    range_min: leaf.range_min,
                    range_max: partition_point,
                    values: lhs_values.into(),
                })),
                rhs: Box::new(TreeNode::Leaf(LeafNode {
                    range_min: partition_point,
                    range_max: leaf.range_max,
                    values: rhs_values.into(),
                })),
            });
        }
        !is_dominated
    } else {
        unreachable!()
    }
}
