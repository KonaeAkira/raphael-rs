use std::collections::BTreeMap;

use simulator::{Action, SimulationState};

use crate::{actions::ActionCombo, utils::Backtracking};

use super::pareto_front::{EffectParetoFront, QualityParetoFront};

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
    quality_pareto_front: QualityParetoFront,
    effect_pareto_front: EffectParetoFront,
    buckets: BTreeMap<SearchScore, Vec<SearchNode>>,
    backtracking: Backtracking<ActionCombo>,
    current_score: SearchScore,
    current_nodes: Vec<(SimulationState, usize)>,
    minimum_score: SearchScore,
}

impl SearchQueue {
    pub fn new(initial_state: SimulationState, minimum_score: SearchScore) -> Self {
        Self {
            quality_pareto_front: Default::default(),
            effect_pareto_front: Default::default(),
            backtracking: Backtracking::new(),
            buckets: Default::default(),
            current_score: SearchScore::MAX,
            current_nodes: vec![(initial_state, Backtracking::<Action>::SENTINEL)],
            minimum_score,
        }
    }

    pub fn update_min_score(&mut self, score: SearchScore) {
        if self.minimum_score >= score {
            return;
        }
        self.minimum_score = score;
        let mut dropped = 0;
        while let Some((bucket_score, _)) = self.buckets.first_key_value() {
            if *bucket_score >= self.minimum_score {
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
        assert!(self.current_score > score);
        if score > self.minimum_score {
            self.buckets.entry(score).or_default().push(SearchNode {
                state,
                action,
                parent_id,
            });
        }
    }

    pub fn pop(&mut self) -> Option<(SimulationState, SearchScore, usize)> {
        while self.current_nodes.is_empty() {
            if let Some((score, mut bucket)) = self.buckets.pop_last() {
                // sort the bucket to prevent inserting a node to the pareto front that is later dominated by another node in the same bucket
                bucket.sort_unstable_by(|lhs, rhs| {
                    pareto_weight(&rhs.state).cmp(&pareto_weight(&lhs.state))
                });
                self.current_score = score;
                self.current_nodes = bucket
                    .into_iter()
                    .filter(|node| {
                        self.quality_pareto_front.insert(node.state)
                            && self.effect_pareto_front.insert(node.state)
                    })
                    .map(|node| {
                        let backtrack_id = self.backtracking.push(node.action, node.parent_id);
                        (node.state, backtrack_id)
                    })
                    .collect();
            } else {
                return None;
            }
        }
        let (state, backtrack_id) = self.current_nodes.pop().unwrap();
        Some((state, self.current_score, backtrack_id))
    }

    pub fn backtrack(&self, backtrack_id: usize) -> impl Iterator<Item = ActionCombo> {
        self.backtracking.get_items(backtrack_id)
    }
}

fn pareto_weight(state: &SimulationState) -> u32 {
    state.cp as u32
        + state.durability as u32
        + state.quality as u32
        + state.unreliable_quality as u32
        + state.effects.into_bits()
}
