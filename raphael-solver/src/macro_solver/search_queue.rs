use std::collections::{BTreeSet, hash_map::Entry};

use raphael_sim::SimulationState;
use rustc_hash::FxHashMap;

use crate::{
    SolverSettings,
    actions::{ActionCombo, use_action_combo},
};

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
struct CandidateNode {
    parent_idx: usize,
    action: ActionCombo,
}

#[derive(Debug, Clone, Copy)]
struct VisitedNode {
    parent_idx: Option<usize>,
    action: Option<ActionCombo>,
    state: SimulationState,
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

pub struct SearchQueue {
    settings: SolverSettings,
    pareto_front: ParetoFront,
    batch_ordering: BTreeSet<SearchScore>,
    batches: FxHashMap<SearchScore, Vec<CandidateNode>>,
    visited_nodes: Vec<VisitedNode>,
    num_inserted_nodes: usize,
    initial_state_visited: bool,
}

impl SearchQueue {
    pub fn new(settings: SolverSettings, initial_state: SimulationState) -> Self {
        Self {
            settings,
            pareto_front: ParetoFront::default(),
            batch_ordering: BTreeSet::default(),
            batches: FxHashMap::default(),
            visited_nodes: vec![VisitedNode {
                parent_idx: None,
                action: None,
                state: initial_state,
            }],
            num_inserted_nodes: 1, // initial node
            initial_state_visited: false,
        }
    }

    pub fn push(&mut self, score: SearchScore, action: ActionCombo, parent_idx: usize) {
        let node = CandidateNode { parent_idx, action };
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
        if !self.initial_state_visited {
            self.initial_state_visited = true;
            let initial_state = self.visited_nodes[0].state;
            return Some(Batch {
                score: SearchScore::MAX,
                nodes: vec![(initial_state, 0)],
            });
        }
        if let Some(score) = self.batch_ordering.pop_last()
            && let Some(batch) = self.batches.remove(&score)
        {
            let mut batch = batch
                .into_iter()
                .map(|candidate_node| {
                    let parent_node_state = self.visited_nodes[candidate_node.parent_idx].state;
                    let candidate_node_state =
                        use_action_combo(&self.settings, parent_node_state, candidate_node.action);
                    VisitedNode {
                        parent_idx: Some(candidate_node.parent_idx),
                        action: Some(candidate_node.action),
                        state: candidate_node_state.unwrap(),
                    }
                })
                .collect::<Vec<_>>();
            // Filter out Pareto-dominated nodes.
            batch.sort_unstable_by(|lhs, rhs| {
                pareto_weight(&rhs.state).cmp(&pareto_weight(&lhs.state))
            });
            batch.retain(|node| self.pareto_front.insert(node.state));
            // Construct the returned batch.
            // Each node in the returned batch tracks its own idx, not the idx of its parent.
            let ret = batch
                .iter()
                .enumerate()
                .map(|(idx, node)| (node.state, self.visited_nodes.len() + idx))
                .collect::<Vec<_>>();
            self.visited_nodes.extend(batch);
            Some(Batch { score, nodes: ret })
        } else {
            None
        }
    }

    pub fn get_actions_from_node_idx(&self, idx: usize) -> impl Iterator<Item = ActionCombo> {
        let mut actions = Vec::new();
        let mut next_idx = Some(idx);
        while let Some(idx) = next_idx {
            let node = &self.visited_nodes[idx];
            if let Some(action) = node.action {
                actions.push(action);
            }
            next_idx = node.parent_idx;
        }
        actions.into_iter().rev()
    }

    pub fn runtime_stats(&self) -> SearchQueueStats {
        SearchQueueStats {
            inserted_nodes: self.num_inserted_nodes,
            processed_nodes: self.visited_nodes.len(),
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
