use std::u32;

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
    pub fn insert(&mut self, state: SimulationState) -> bool {
        const MAX_LEAF_SIZE: usize = 500;
        let new_value = Value::from(&state);
        let mut node = self.buckets.entry(Key::from(&state)).or_default();
        while let TreeNode::Intermediate(intermediate) = node {
            if new_value.cp() < intermediate.partition_point {
                node = intermediate.lhs.as_mut();
            } else {
                node = intermediate.rhs.as_mut();
            }
        }
        if let TreeNode::Leaf(leaf) = node {
            let is_dominated = leaf.values.iter().any(|value| value.dominates(&new_value));
            if !is_dominated {
                leaf.values.retain(|value| !new_value.dominates(value));
                leaf.values.push(new_value);
            }
            if leaf.values.len() > MAX_LEAF_SIZE && leaf.range_min + 1 != leaf.range_max {
                leaf.values.sort_unstable_by_key(|value| value.cp());
                let (lhs_values, rhs_values) = leaf.values.split_at(MAX_LEAF_SIZE / 2);
                let partition_point = rhs_values[0].cp();
                *node = TreeNode::Intermediate(IntermediateNode {
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
}
