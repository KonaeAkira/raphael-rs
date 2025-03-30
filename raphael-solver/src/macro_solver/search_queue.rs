use std::collections::BTreeMap;

use super::pareto_front::{ParetoFront, ParetoValue};
use raphael_sim::{Action, Effects, SimulationState};
use rustc_hash::FxHashMap;

use crate::{actions::ActionCombo, utils::Backtracking};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    state: SimulationState,
    action: ActionCombo,
    parent_id: usize,
}

pub struct SearchQueue {
    pareto_fronts: FxHashMap<Effects, ParetoFront<ParetoValue>>,
    buckets: BTreeMap<SearchScore, Vec<SearchNode>>,
    backtracking: Backtracking<ActionCombo>,
    current_score: SearchScore,
    current_nodes: Vec<(SimulationState, usize)>,
    minimum_score: SearchScore,
    nodes_pushed: usize,
    nodes_popped: usize,
}

impl SearchQueue {
    pub fn new(initial_state: SimulationState, minimum_score: SearchScore) -> Self {
        log::debug!("New minimum score: {:?}", minimum_score);
        Self {
            pareto_fronts: FxHashMap::default(),
            backtracking: Backtracking::new(),
            buckets: BTreeMap::default(),
            current_score: SearchScore::MAX,
            current_nodes: vec![(initial_state, Backtracking::<Action>::SENTINEL)],
            minimum_score,
            nodes_pushed: 0,
            nodes_popped: 0,
        }
    }

    pub fn update_min_score(&mut self, score: SearchScore) {
        if self.minimum_score >= score {
            return;
        }
        self.minimum_score = score;
        let mut dropped = 0;
        while let Some((score, _)) = self.buckets.first_key_value() {
            if *score >= self.minimum_score {
                break;
            }
            dropped += self.buckets.pop_first().unwrap().1.len();
        }
        log::debug!("New minimum score: {:?}", score);
        log::debug!("Nodes dropped: {}", dropped);
    }

    pub fn push(
        &mut self,
        state: SimulationState,
        score: SearchScore,
        action: ActionCombo,
        parent_id: usize,
    ) {
        #[cfg(test)]
        assert!(score < self.current_score);
        if score > self.minimum_score {
            self.buckets.entry(score).or_default().push(SearchNode {
                state,
                action,
                parent_id,
            });
            self.nodes_pushed += 1;
        }
    }

    pub fn pop(&mut self) -> Option<(SimulationState, SearchScore, usize)> {
        while self.current_nodes.is_empty() {
            if let Some((score, mut nodes)) = self.buckets.pop_last() {
                self.current_score = score;
                // Nodes are sorted by their pareto key to improve cache efficiency on insertion.
                // Nodes with the same key are sorted by decreasing pareto weight to make sure that no inserted node is later dominated by another node in the same bucket.
                nodes.sort_unstable_by(|lhs, rhs| {
                    let lhs_pareto_key = lhs.state.effects.into_bits();
                    let rhs_pareto_key = rhs.state.effects.into_bits();
                    lhs_pareto_key
                        .cmp(&rhs_pareto_key)
                        .then(ParetoValue::weight(&rhs.state).cmp(&ParetoValue::weight(&lhs.state)))
                });
                for node in nodes.into_iter() {
                    let pareto_front = self.pareto_fronts.entry(node.state.effects).or_default();
                    if pareto_front.insert(ParetoValue::new(node.state)) {
                        let backtrack_id = self.backtracking.push(node.action, node.parent_id);
                        self.current_nodes.push((node.state, backtrack_id));
                    }
                }
            } else {
                return None;
            }
        }
        let (state, backtrack_id) = self.current_nodes.pop().unwrap();
        self.nodes_popped += 1;
        Some((state, self.current_score, backtrack_id))
    }

    pub fn backtrack(&self, backtrack_id: usize) -> impl Iterator<Item = ActionCombo> {
        self.backtracking.get_items(backtrack_id)
    }
}

impl Drop for SearchQueue {
    fn drop(&mut self) {
        log::debug!("Total nodes pushed: {}", self.nodes_pushed);
        log::debug!("Total nodes popped: {}", self.nodes_popped);
        log::debug!("Pareto front keys: {}", self.pareto_fronts.len());
    }
}
