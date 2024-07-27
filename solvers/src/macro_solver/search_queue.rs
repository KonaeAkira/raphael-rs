use std::collections::BTreeMap;

use radix_heap::Radix;
use simulator::{state::InProgress, Action, Settings};

use crate::utils::Backtracking;

use super::pareto_set::ParetoSet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct MacroLength {
    pub duration: u8,
    pub steps: u8,
}

impl MacroLength {
    pub fn add(self, action: Action) -> Self {
        Self {
            duration: self.duration + action.time_cost() as u8,
            steps: self.steps + 1,
        }
    }
}

impl Radix for MacroLength {
    fn radix_similarity(&self, other: &Self) -> u32 {
        if self.duration != other.duration {
            self.duration.radix_similarity(&other.duration)
        } else {
            self.steps.radix_similarity(&other.steps) + 8
        }
    }

    const RADIX_BITS: u32 = 16;
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
    buckets: BTreeMap<MacroLength, Bucket>,
    backtracking: Backtracking<Action>,
    current_macro_length: MacroLength,
    current_nodes: Vec<(InProgress, u32)>,
}

impl SearchQueue {
    pub fn new(settings: Settings) -> Self {
        let initial_state = InProgress::new(&settings);
        Self {
            settings,
            pareto_set: Default::default(),
            backtracking: Backtracking::new(),
            buckets: Default::default(),
            current_macro_length: Default::default(),
            current_nodes: vec![(initial_state, Backtracking::<Action>::SENTINEL)],
        }
    }

    pub fn push(&mut self, state: InProgress, action: Action, parent_id: u32) {
        let key = self.current_macro_length.add(action);
        self.buckets.entry(key).or_default().push(
            SearchNode {
                state,
                action,
                parent_id,
            },
            &self.settings,
        );
    }

    pub fn pop(&mut self) -> Option<(InProgress, u32)> {
        if self.current_nodes.is_empty() {
            if let Some((macro_length, bucket)) = self.buckets.pop_first() {
                self.current_macro_length = macro_length;
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
        self.current_nodes.pop()
    }

    pub fn backtrack(&self, backtrack_id: u32) -> impl Iterator<Item = Action> {
        self.backtracking.get(backtrack_id)
    }
}
