use pareto_front::{Dominate, ParetoFront};

use rustc_hash::FxHashMap as HashMap;

use crate::game::{state::InProgress, units::*, Action, ComboAction, Effects, Settings};

use super::ActionSequence;

#[derive(Debug, Clone, Copy)]
pub struct SearchTrace<'a> {
    pub parent: &'a Option<SearchTrace<'a>>,
    pub action: ActionSequence,
}

impl<'a> SearchTrace<'a> {
    pub fn new(parent: &'a Option<SearchTrace<'a>>, action: ActionSequence) -> Self {
        SearchTrace { parent, action }
    }

    pub fn actions(self) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();
        self.do_trace(&mut actions);
        actions.reverse();
        actions
    }

    fn do_trace(self, actions: &mut Vec<Action>) {
        for action in self.action.actions().iter().rev() {
            actions.push(*action);
        }
        if let Some(parent) = self.parent {
            parent.do_trace(actions);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchNode<'a> {
    pub state: InProgress,
    pub trace: Option<SearchTrace<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ParetoKey {
    pub combo: Option<ComboAction>,
    pub durability: Durability,
    pub effects: Effects,
}

impl From<&SearchNode<'_>> for ParetoKey {
    fn from(value: &SearchNode) -> Self {
        ParetoKey {
            combo: value.state.combo,
            durability: value.state.durability,
            effects: Effects {
                inner_quiet: 0,
                ..value.state.effects
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParetoValue {
    pub missing_progress: Progress,
    pub missing_quality: Quality,
    pub inner_quiet: u8,
}

impl<'a> From<&SearchNode<'a>> for ParetoValue {
    fn from(value: &SearchNode<'a>) -> Self {
        ParetoValue {
            missing_progress: value.state.missing_progress,
            missing_quality: value.state.missing_quality,
            inner_quiet: value.state.effects.inner_quiet,
        }
    }
}

impl Dominate for ParetoValue {
    fn dominate(&self, other: &Self) -> bool {
        self.missing_progress <= other.missing_progress
            && self.missing_quality <= other.missing_quality
            && self.inner_quiet >= other.inner_quiet
    }
}

impl<'a> Dominate for SearchNode<'a> {
    fn dominate(&self, other: &Self) -> bool {
        self.state.missing_progress <= other.state.missing_progress
            && self.state.missing_quality <= other.state.missing_quality
            && self.state.effects.inner_quiet >= other.state.effects.inner_quiet
    }
}

type FrontHashMap<T> = HashMap<ParetoKey, ParetoFront<T>>;

pub struct SearchQueue<'a> {
    current: Vec<SearchNode<'a>>,
    local_fronts: Vec<FrontHashMap<SearchNode<'a>>>,
    global_front: FrontHashMap<ParetoValue>,
    accepted_nodes: usize,
    locally_rejected_nodes: usize,
    globally_rejected_nodes: usize,
}

impl<'a> SearchQueue<'a> {
    pub fn new(settings: Settings) -> SearchQueue<'a> {
        SearchQueue {
            current: Vec::new(),
            local_fronts: vec![FrontHashMap::default(); (settings.max_cp + 1) as usize],
            global_front: FrontHashMap::default(),
            accepted_nodes: 0,
            locally_rejected_nodes: 0,
            globally_rejected_nodes: 0,
        }
    }

    pub fn push(&mut self, value: SearchNode<'a>) {
        if !self.local_fronts[value.state.cp as usize]
            .entry(ParetoKey::from(&value))
            .or_default()
            .push(value)
        {
            self.locally_rejected_nodes += 1;
        }
    }

    pub fn pop(&mut self) -> Option<SearchNode<'a>> {
        if let Some(node) = self.current.pop() {
            Some(node)
        } else if self.pop_bucket() {
            self.pop()
        } else {
            None
        }
    }

    fn pop_bucket(&mut self) -> bool {
        if let Some(local_front) = self.local_fronts.pop() {
            for (key, local_front) in local_front.into_iter() {
                let global_front = self.global_front.entry(key).or_default();
                for node in local_front.into_iter() {
                    if global_front.push(ParetoValue::from(&node)) {
                        self.current.push(node);
                        self.accepted_nodes += 1;
                    } else {
                        self.globally_rejected_nodes += 1;
                    }
                }
            }
            true
        } else {
            false
        }
    }
}

impl<'a> Drop for SearchQueue<'a> {
    fn drop(&mut self) {
        dbg!(
            self.accepted_nodes,
            self.locally_rejected_nodes,
            self.globally_rejected_nodes
        );
    }
}
