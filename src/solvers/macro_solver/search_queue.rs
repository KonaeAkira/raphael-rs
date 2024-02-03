use crate::game::{state::InProgress, Action, Settings};

use super::ActionSequence;
use super::ParetoFront;

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

pub struct SearchQueue<'a> {
    seed: Vec<SearchNode<'a>>,
    buckets: Vec<Vec<SearchNode<'a>>>,
    pareto_front: ParetoFront,
}

impl<'a> SearchQueue<'a> {
    pub fn new(settings: Settings) -> SearchQueue<'a> {
        SearchQueue {
            seed: Vec::new(),
            buckets: vec![Vec::new(); (settings.max_cp + 1) as usize],
            pareto_front: ParetoFront::new(),
        }
    }

    pub fn push_seed(&mut self, node: SearchNode<'a>) {
        self.seed.push(node);
    }

    pub fn push(&mut self, node: SearchNode<'a>) {
        self.buckets[node.state.cp as usize].push(node);
    }

    pub fn pop(&mut self) -> Option<SearchNode<'a>> {
        if let Some(node) = self.seed.pop() {
            return Some(node);
        } else if self.pop_bucket() {
            return self.pop();
        } else {
            return None;
        }
    }

    fn pop_bucket(&mut self) -> bool {
        if let Some(bucket) = self.buckets.pop() {
            let mut unique: Vec<SearchNode> = Vec::new();
            for node in bucket {
                if self.pareto_front.insert(&node.state) {
                    unique.push(node);
                }
            }
            for node in unique {
                if self.pareto_front.has(&node.state) {
                    self.seed.push(node);
                }
            }
            return true;
        } else {
            return false;
        }
    }
}
