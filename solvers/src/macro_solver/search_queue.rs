use std::collections::BTreeMap;

use simulator::{state::InProgress, Action, Settings};

use crate::utils::Backtracking;

use super::pareto_set::ParetoSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchScore {
    pub quality: u16,
    pub duration: u8,
    pub steps: u8,
    pub quality_overflow: u16,
}

impl SearchScore {
    pub const MAX: Self = Self {
        quality: u16::MAX,
        duration: 0,
        steps: 0,
        quality_overflow: u16::MAX,
    };

    pub fn new(quality: u16, duration: u8, steps: u8, settings: &Settings) -> Self {
        Self {
            quality: std::cmp::min(settings.max_quality, quality),
            duration,
            steps,
            quality_overflow: quality.saturating_sub(settings.max_quality),
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
            .then(other.duration.cmp(&self.duration))
            .then(other.steps.cmp(&self.steps))
            .then(self.quality_overflow.cmp(&other.quality_overflow))
    }
}

impl radix_heap::Radix for SearchScore {
    const RADIX_BITS: u32 = 48;
    fn radix_similarity(&self, other: &Self) -> u32 {
        if self.quality != other.quality {
            self.quality.radix_similarity(&other.quality)
        } else if self.duration != other.duration {
            self.duration.radix_similarity(&other.duration) + 16
        } else if self.steps != other.steps {
            self.steps.radix_similarity(&other.steps) + 24
        } else {
            self.quality_overflow
                .radix_similarity(&other.quality_overflow)
                + 32
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    state: InProgress,
    action: Action,
    parent_id: u32,
}

#[derive(Default)]
struct Bucket {
    nodes: Vec<SearchNode>,
    pareto_set: ParetoSet,
}

impl Bucket {
    fn push(&mut self, node: SearchNode, settings: &Settings) {
        if self.pareto_set.insert(*node.state.raw_state(), settings) {
            self.nodes.push(node);
        }
    }

    fn into_iter(self, settings: Settings) -> impl Iterator<Item = SearchNode> {
        let Self { nodes, pareto_set } = self;
        nodes
            .into_iter()
            .filter(move |node| pareto_set.contains(*node.state.raw_state(), &settings))
    }
}

pub struct SearchQueue {
    settings: Settings,
    pareto_set: ParetoSet,
    buckets: BTreeMap<SearchScore, Bucket>,
    backtracking: Backtracking<Action>,
    current_score: SearchScore,
    current_nodes: Vec<(InProgress, u32)>,
    locked: bool,
}

impl SearchQueue {
    pub fn new(initial_state: InProgress, initial_score: SearchScore, settings: Settings) -> Self {
        Self {
            settings,
            pareto_set: Default::default(),
            backtracking: Backtracking::new(),
            buckets: Default::default(),
            current_score: initial_score,
            current_nodes: vec![(initial_state, Backtracking::<Action>::SENTINEL)],
            locked: false,
        }
    }

    /// Clear all remaining buckets and prevent any more nodes from being added
    pub fn lock(&mut self) {
        self.buckets.clear();
        self.locked = true;
    }

    pub fn push(&mut self, state: InProgress, score: SearchScore, action: Action, parent_id: u32) {
        if self.locked {
            return;
        }
        assert!(self.current_score > score);
        self.buckets.entry(score).or_default().push(
            SearchNode {
                state,
                action,
                parent_id,
            },
            &self.settings,
        );
    }

    pub fn pop(&mut self) -> Option<(InProgress, SearchScore, u32)> {
        while self.current_nodes.is_empty() {
            if let Some((score, bucket)) = self.buckets.pop_last() {
                self.current_score = score;
                self.current_nodes = bucket
                    .into_iter(self.settings)
                    .filter(|node| {
                        self.pareto_set
                            .insert(*node.state.raw_state(), &self.settings)
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

    pub fn backtrack(&self, backtrack_id: u32) -> impl Iterator<Item = Action> {
        self.backtracking.get(backtrack_id)
    }
}
