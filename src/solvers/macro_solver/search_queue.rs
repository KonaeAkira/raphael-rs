use pareto_front::{Dominate, ParetoFront};

use rustc_hash::FxHashMap as HashMap;

use crate::game::{state::InProgress, units::*, Action, ComboAction, Effects, Settings};

use super::ActionSequence;

#[derive(Debug, Clone, Copy)]
pub struct SearchTrace<'a> {
    pub parent: &'a SearchNode<'a>,
    pub action: ActionSequence,
}

impl<'a> SearchTrace<'a> {
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
        if let Some(parent) = self.parent.trace {
            parent.do_trace(actions);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchNode<'a> {
    pub state: InProgress,
    pub trace: Option<SearchTrace<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParetoValue {
    pub progress: Progress,
    pub quality: Quality,
    pub inner_quiet: i8,
}

impl<'a> From<&SearchNode<'a>> for ParetoValue {
    fn from(value: &SearchNode<'a>) -> Self {
        ParetoValue {
            progress: value.state.progress,
            quality: value.state.quality,
            inner_quiet: value.state.effects.inner_quiet,
        }
    }
}

impl Dominate for ParetoValue {
    fn dominate(&self, other: &Self) -> bool {
        self.progress >= other.progress
            && self.quality >= other.quality
            && self.inner_quiet >= other.inner_quiet
    }
}

type FrontHashMap<T> = HashMap<ParetoKey, ParetoFront<T>>;

pub struct SearchQueue<'a> {
    current: Vec<SearchNode<'a>>,
    buckets: Vec<Vec<SearchNode<'a>>>,
    pareto_front: FrontHashMap<ParetoValue>,
}

impl<'a> SearchQueue<'a> {
    pub fn new(settings: Settings) -> SearchQueue<'a> {
        SearchQueue {
            current: Vec::new(),
            buckets: vec![Vec::new(); (settings.max_cp + 1) as usize],
            pareto_front: FrontHashMap::default(),
        }
    }

    pub fn push(&mut self, value: SearchNode<'a>) {
        self.buckets[value.state.cp as usize].push(value);
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
        if let Some(front) = self.buckets.pop() {
            for value in front {
                let key = ParetoKey::from(&value);
                let global_front = self.pareto_front.entry(key).or_default();
                if global_front.push(ParetoValue::from(&value)) {
                    self.current.push(value);
                }
            }
            true
        } else {
            false
        }
    }
}
