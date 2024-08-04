use std::collections::BTreeMap;

use simulator::{Action, Settings, SimulationState};

use crate::utils::Backtracking;

use super::pareto_front::{EffectParetoFront, QualityParetoFront};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchScore {
    pub quality: u16,
    pub duration: u8,
    pub steps: u8,
    pub quality_overflow: u16,
}

impl SearchScore {
    pub fn new(quality: u16, duration: u8, steps: u8, settings: &Settings) -> Self {
        Self {
            quality: std::cmp::min(settings.max_quality, quality),
            duration,
            steps,
            quality_overflow: quality.saturating_sub(settings.max_quality),
        }
    }

    fn difference(self, other: &Self) -> f32 {
        if self.quality != other.quality {
            self.quality.abs_diff(other.quality) as f32
        } else {
            self.steps.abs_diff(other.steps) as f32 / 255.0
        }
    }
}

impl std::cmp::PartialOrd for SearchScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl std::cmp::Ord for SearchScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quality
            .cmp(&other.quality)
            .then(other.steps.cmp(&self.steps))
            .then(other.duration.cmp(&self.duration))
            .then(self.quality_overflow.cmp(&other.quality_overflow))
    }
}

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    state: SimulationState,
    action: Action,
    parent_id: usize,
}

pub struct SearchQueue {
    settings: Settings,
    quality_pareto_front: QualityParetoFront,
    effect_pareto_front: EffectParetoFront,
    buckets: BTreeMap<SearchScore, Vec<SearchNode>>,
    backtracking: Backtracking<Action>,
    current_score: SearchScore,
    current_nodes: Vec<(SimulationState, usize)>,
    minimum_score: SearchScore,
    initial_score_difference: f32,
}

impl SearchQueue {
    pub fn new(
        initial_state: SimulationState,
        initial_score: SearchScore,
        minimum_score: SearchScore,
        settings: Settings,
    ) -> Self {
        Self {
            settings,
            quality_pareto_front: Default::default(),
            effect_pareto_front: Default::default(),
            backtracking: Backtracking::new(),
            buckets: Default::default(),
            current_score: initial_score,
            current_nodes: vec![(initial_state, Backtracking::<Action>::SENTINEL)],
            minimum_score,
            initial_score_difference: initial_score.difference(&minimum_score),
        }
    }

    pub fn progress_estimate(&self) -> f32 {
        1.0 - self.current_score.difference(&self.minimum_score) / self.initial_score_difference
    }

    pub fn update_min_score(&mut self, score: SearchScore) {
        if self.minimum_score >= score {
            return;
        }
        self.minimum_score = score;
        while let Some((bucket_score, _)) = self.buckets.first_key_value() {
            if *bucket_score >= self.minimum_score {
                break;
            }
            self.buckets.pop_first();
        }
    }

    pub fn push(
        &mut self,
        state: SimulationState,
        score: SearchScore,
        action: Action,
        parent_id: usize,
    ) {
        assert!(self.current_score > score);
        if score < self.minimum_score {
            return;
        }
        self.buckets.entry(score).or_default().push(SearchNode {
            state,
            action,
            parent_id,
        });
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
                        self.quality_pareto_front.insert(node.state, &self.settings)
                            && self.effect_pareto_front.insert(node.state, &self.settings)
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

    pub fn backtrack(&self, backtrack_id: usize) -> impl Iterator<Item = Action> {
        self.backtracking.get(backtrack_id)
    }
}

fn pareto_weight(state: &SimulationState) -> u32 {
    state.cp as u32
        + state.durability as u32
        + state.unreliable_quality[0] as u32
        + state.unreliable_quality[1] as u32
        + state.effects.into_bits()
        + state.combo.into_bits() as u32
}
