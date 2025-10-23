use std::collections::BTreeMap;

use raphael_sim::{Action, SimulationState};

use crate::{SolverException, actions::ActionCombo, macros::internal_error, utils::Backtracking};

use super::pareto_front::ParetoFront;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
pub struct SearchNode {
    pub state: SimulationState,
    pub action: ActionCombo,
    pub parent_id: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchQueueStats {
    pub inserted_nodes: usize,
    pub processed_nodes: usize,
}

pub struct SearchQueue {
    pareto_front: ParetoFront,
    batches: BTreeMap<SearchScore, Vec<SearchNode>>,
    backtracking: Backtracking<ActionCombo>,
    initial_state: SimulationState,
    current_score: SearchScore,
    minimum_score: SearchScore,
    inserted_nodes: usize,
    processed_nodes: usize,
}

impl SearchQueue {
    pub fn new(initial_state: SimulationState) -> Self {
        Self {
            pareto_front: ParetoFront::default(),
            backtracking: Backtracking::new(),
            batches: BTreeMap::default(),
            initial_state,
            current_score: SearchScore::MAX,
            minimum_score: SearchScore::MIN,
            inserted_nodes: 1, // initial node
            processed_nodes: 0,
        }
    }

    pub fn min_score(&self) -> SearchScore {
        self.minimum_score
    }

    pub fn update_min_score(&mut self, score: SearchScore) {
        if self.minimum_score >= score {
            return;
        }
        self.minimum_score = score;
        let mut dropped = 0;
        while let Some((bucket_score, _)) = self.batches.first_key_value() {
            if *bucket_score >= self.minimum_score {
                break;
            }
            dropped += self.batches.pop_first().unwrap().1.len();
        }
        log::trace!(
            "New minimum score: ({}, {}, {}). Nodes dropped: {}",
            score.quality_upper_bound,
            score.steps_lower_bound,
            score.duration_lower_bound,
            dropped
        );
    }

    pub fn try_push(
        &mut self,
        state: SimulationState,
        score: SearchScore,
        action: ActionCombo,
        parent_id: usize,
    ) -> Result<(), SolverException> {
        if score >= self.current_score {
            return Err(internal_error!(
                "Search score isn't strictly monotonic.",
                state,
                action,
                score,
                self.current_score
            ));
        }
        if score > self.minimum_score {
            self.batches.entry(score).or_default().push(SearchNode {
                state,
                action,
                parent_id,
            });
            self.inserted_nodes += 1;
        }
        Ok(())
    }

    pub fn pop_batch(&mut self) -> Option<Vec<(SimulationState, SearchScore, usize)>> {
        if self.processed_nodes == 0 {
            self.processed_nodes += 1;
            return Some(vec![(
                self.initial_state,
                self.current_score,
                Backtracking::<Action>::SENTINEL,
            )]);
        }
        if let Some((score, batch)) = self.batches.pop_last() {
            self.current_score = score;
            let filtered_batch = self.pareto_front.par_insert(batch);
            self.processed_nodes += filtered_batch.len();
            Some(
                filtered_batch
                    .into_iter()
                    .map(|node| {
                        let backtrack_id = self.backtracking.push(node.action, node.parent_id);
                        (node.state, score, backtrack_id)
                    })
                    .collect(),
            )
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
