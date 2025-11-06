use std::collections::{BTreeSet, hash_map::Entry};

use raphael_sim::{Action, SimulationState};
use rustc_hash::FxHashMap;

use crate::{actions::ActionCombo, utils::Backtracking};

use super::pareto_front::ParetoFront;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SearchScore {
    pub quality_upper_bound: u32,
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
        quality_upper_bound: u32::MAX,
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

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    state: SimulationState,
    action: ActionCombo,
    parent_id: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchQueueStats {
    pub inserted_nodes: usize,
    pub processed_nodes: usize,
}

pub struct SearchQueue {
    pareto_front: ParetoFront,
    batch_ordering: BTreeSet<SearchScore>,
    batches: FxHashMap<SearchScore, Vec<SearchNode>>,
    backtracking: Backtracking<ActionCombo>,
    initial_state: SimulationState,
    inserted_nodes: usize,
    processed_nodes: usize,
}

impl SearchQueue {
    pub fn new(initial_state: SimulationState) -> Self {
        Self {
            pareto_front: ParetoFront::default(),
            backtracking: Backtracking::new(),
            batch_ordering: BTreeSet::default(),
            batches: FxHashMap::default(),
            initial_state,
            inserted_nodes: 1, // initial node
            processed_nodes: 0,
        }
    }

    pub fn push(
        &mut self,
        state: SimulationState,
        score: SearchScore,
        action: ActionCombo,
        parent_id: usize,
    ) {
        let node = SearchNode {
            state,
            action,
            parent_id,
        };
        match self.batches.entry(score) {
            Entry::Occupied(occupied_entry) => {
                occupied_entry.into_mut().push(node);
            }
            Entry::Vacant(vacant_entry) => {
                self.batch_ordering.insert(score);
                vacant_entry.insert(vec![node]);
            }
        }
        self.inserted_nodes += 1;
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

    pub fn next_batch_score(&self) -> Option<&SearchScore> {
        self.batch_ordering.last()
    }

    pub fn pop_batch(&mut self) -> Option<(SearchScore, Vec<(SimulationState, usize)>)> {
        if self.processed_nodes == 0 {
            self.processed_nodes += 1;
            return Some((
                SearchScore::MAX,
                vec![(self.initial_state, Backtracking::<Action>::SENTINEL)],
            ));
        }
        if let Some(score) = self.batch_ordering.pop_last()
            && let Some(mut batch) = self.batches.remove(&score)
        {
            // sort the bucket to prevent inserting a node to the pareto front that is later dominated by another node in the same bucket
            batch.sort_unstable_by(|lhs, rhs| {
                pareto_weight(&rhs.state).cmp(&pareto_weight(&lhs.state))
            });
            let filtered_batch = batch
                .into_iter()
                .filter(|node| self.pareto_front.insert(node.state))
                .map(|node| {
                    let backtrack_id = self.backtracking.push(node.action, node.parent_id);
                    (node.state, backtrack_id)
                })
                .collect::<Vec<_>>();
            self.processed_nodes += filtered_batch.len();
            Some((score, filtered_batch))
        } else {
            None
        }
    }

    pub fn backtrack(&self, backtrack_id: usize) -> impl Iterator<Item = ActionCombo> {
        self.backtracking.get_items(backtrack_id)
    }

    pub fn runtime_stats(&self) -> SearchQueueStats {
        SearchQueueStats {
            inserted_nodes: self.inserted_nodes,
            processed_nodes: self.processed_nodes,
        }
    }
}

fn pareto_weight(state: &SimulationState) -> u32 {
    state.cp as u32
        + state.durability as u32
        + state.quality
        + state.unreliable_quality
        + state.effects.into_bits()
}
