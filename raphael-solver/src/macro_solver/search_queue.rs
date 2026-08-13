use std::collections::{BTreeSet, hash_map::Entry};

use raphael_sim::SimulationState;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, use_action_combo},
};

use super::pareto_front::ParetoFront;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SearchScore {
    pub quality_upper_bound: u16,
    pub steps_lower_bound: u8,
    pub duration_lower_bound: u8,
    pub current_steps: u8,
    pub current_duration: u8,
}

impl SearchScore {
    pub const MIN: Self = Self {
        quality_upper_bound: 0,
        steps_lower_bound: u8::MAX,
        duration_lower_bound: u8::MAX,
        current_steps: u8::MAX,
        current_duration: u8::MAX,
    };

    pub const MAX: Self = Self {
        quality_upper_bound: u16::MAX,
        steps_lower_bound: 0,
        duration_lower_bound: 0,
        current_steps: 0,
        current_duration: 0,
    };
}

impl std::cmp::PartialOrd for SearchScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl std::cmp::Ord for SearchScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quality_upper_bound
            .cmp(&other.quality_upper_bound)
            .then(other.steps_lower_bound.cmp(&self.steps_lower_bound))
            .then(other.duration_lower_bound.cmp(&self.duration_lower_bound))
            .then(other.current_steps.cmp(&self.current_steps))
            .then(other.current_duration.cmp(&self.current_duration))
    }
}

#[cfg(target_pointer_width = "32")]
#[bitfield_struct::bitfield(u32)]
struct SearchNode {
    #[bits(26)]
    parent_idx: usize,
    #[bits(6)]
    action: ActionCombo,
}

#[cfg(target_pointer_width = "64")]
#[bitfield_struct::bitfield(u64)]
struct SearchNode {
    #[bits(58)]
    parent_idx: usize,
    #[bits(6)]
    action: ActionCombo,
}

#[derive(Debug)]
pub struct Batch {
    pub score: SearchScore,
    pub nodes: Vec<(SimulationState, usize)>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchQueueStats {
    pub inserted_nodes: usize,
    pub processed_nodes: usize,
}

struct SearchRoot {
    state: SimulationState,
    prefix: Vec<ActionCombo>,
}

pub struct SearchQueue {
    settings: SolverSettings,
    pareto_front: ParetoFront,
    batch_ordering: BTreeSet<SearchScore>,
    batches: FxHashMap<SearchScore, Vec<SearchNode>>,
    visited_nodes: Vec<SearchNode>,
    visited_root_ids: Vec<usize>,
    roots: Vec<SearchRoot>,
    num_inserted_nodes: usize,
}

impl SearchQueue {
    pub fn new(
        settings: SolverSettings,
        roots: Vec<(SearchScore, SimulationState, Vec<ActionCombo>)>,
    ) -> Self {
        let mut search_queue = Self {
            settings,
            pareto_front: ParetoFront::default(),
            batch_ordering: BTreeSet::default(),
            batches: FxHashMap::default(),
            visited_nodes: Vec::new(),
            visited_root_ids: Vec::new(),
            roots: roots
                .iter()
                .map(|(_, state, prefix)| SearchRoot {
                    state: *state,
                    prefix: prefix.clone(),
                })
                .collect(),
            num_inserted_nodes: 0,
        };
        for (root_id, (score, _, _)) in roots.into_iter().enumerate() {
            let _ = search_queue.push(score, ActionCombo::None, root_id);
        }
        search_queue
    }

    pub fn push(
        &mut self,
        score: SearchScore,
        action: ActionCombo,
        parent_idx: usize,
    ) -> Result<(), SolverException> {
        let node = SearchNode::new()
            .with_parent_idx_checked(parent_idx)
            .map_err(|_| SolverException::SearchQueueCapacityExceeded)?
            .with_action(action);
        match self.batches.entry(score) {
            Entry::Occupied(occupied_entry) => {
                occupied_entry.into_mut().push(node);
            }
            Entry::Vacant(vacant_entry) => {
                self.batch_ordering.insert(score);
                vacant_entry.insert(vec![node]);
            }
        }
        self.num_inserted_nodes += 1;
        Ok(())
    }

    pub fn drop_nodes_below_score(&mut self, min_score: SearchScore) {
        let mut dropped = 0;
        while let Some(&score) = self.batch_ordering.first()
            && score < min_score
        {
            self.batch_ordering.pop_first();
            dropped += self.batches.remove(&score).map_or(0, |batch| batch.len());
        }
        if dropped != 0 {
            log::trace!("{dropped} nodes dropped ({min_score:?})");
        }
    }

    pub fn pop_batch(&mut self) -> Option<Batch> {
        if let Some(score) = self.batch_ordering.pop_last()
            && let Some(batch) = self.batches.remove(&score)
        {
            // Because each node only stores the previous action and idx of the parent, we first need
            // to backtrack and replay all actions from the initial state to get the current state.
            let batch = batch
                .into_par_iter()
                .map(|search_node| {
                    if search_node.action() == ActionCombo::None {
                        let root_id = search_node.parent_idx();
                        (search_node, self.roots[root_id].state, root_id)
                    } else {
                        let parent_idx = search_node.parent_idx();
                        let root_id = self.visited_root_ids[parent_idx];
                        let mut state = self.roots[root_id].state;
                        for action in self.get_actions_after_root(parent_idx) {
                            state = use_action_combo(&self.settings, state, action).unwrap();
                        }
                        state =
                            use_action_combo(&self.settings, state, search_node.action()).unwrap();
                        (search_node, state, root_id)
                    }
                })
                .collect();
            // Filter out Pareto-dominated nodes.
            let non_dominated_nodes = self
                .pareto_front
                .insert_batch(batch, |expanded_node| &expanded_node.1)
                .collect::<Vec<_>>();
            let batch = Batch {
                score,
                nodes: non_dominated_nodes
                    .iter()
                    .enumerate()
                    .map(|(idx, node)| {
                        let state = node.1;
                        let self_idx = self.visited_nodes.len() + idx;
                        (state, self_idx)
                    })
                    .collect(),
            };
            self.visited_root_ids.extend(
                non_dominated_nodes
                    .iter()
                    .map(|expanded_node| expanded_node.2),
            );
            self.visited_nodes.extend(
                non_dominated_nodes
                    .into_iter()
                    .map(|expanded_node| expanded_node.0),
            );
            Some(batch)
        } else {
            None
        }
    }

    fn get_actions_after_root(&self, mut idx: usize) -> SmallVec<[ActionCombo; 56]> {
        let mut actions = SmallVec::new();
        loop {
            let search_node = self.visited_nodes[idx];
            if search_node.action() == ActionCombo::None {
                break;
            }
            actions.push(search_node.action());
            idx = search_node.parent_idx();
        }
        actions.reverse();
        actions
    }

    pub fn get_actions_from_node_idx(&self, idx: usize) -> SmallVec<[ActionCombo; 56]> {
        let root_id = self.visited_root_ids[idx];
        let mut actions = SmallVec::from_slice(&self.roots[root_id].prefix);
        actions.extend(self.get_actions_after_root(idx));
        actions
    }

    pub fn runtime_stats(&self) -> SearchQueueStats {
        SearchQueueStats {
            inserted_nodes: self.num_inserted_nodes,
            processed_nodes: self.visited_nodes.len(),
        }
    }
}
