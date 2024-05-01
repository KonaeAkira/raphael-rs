use crate::game::{state::InProgress, units::Quality, Action, Settings, State};
use crate::solvers::FinishSolver;

use std::time::Instant;
use std::vec::Vec;
use typed_arena::Arena;

use strum::IntoEnumIterator;

use super::*;

struct MacroResult {
    quality: Quality,
    actions: Vec<Action>,
}

pub struct MacroSolver {
    settings: Settings,
    finish_solver: FinishSolver,
}

impl MacroSolver {
    pub fn new(settings: Settings) -> MacroSolver {
        MacroSolver {
            settings,
            finish_solver: FinishSolver::new(settings),
        }
    }

    pub fn solve(&mut self, state: State) -> Option<Vec<Action>> {
        match state {
            State::InProgress(state) => {
                let result = self.do_solve(state);
                match result {
                    Some(result) => Some(result.actions),
                    None => None,
                }
            }
            _ => None,
        }
    }

    fn do_solve(&mut self, state: InProgress) -> Option<MacroResult> {
        let timer = Instant::now();

        let mut search_queue = SearchQueue::new(self.settings);
        let explored_nodes: Arena<SearchNode> = Arena::new();

        search_queue.push(SearchNode { state, trace: None });

        let mut result = MacroResult {
            quality: Quality::new(0),
            actions: Vec::new(),
        };

        while let Some(current_node) = search_queue.pop() {
            let current_node: &SearchNode<'_> = explored_nodes.alloc(current_node);
            for sequence in ActionSequence::iter() {
                if !sequence.should_use(&current_node.state) {
                    continue;
                }
                let new_state =
                    sequence.apply(State::InProgress(current_node.state), &self.settings);
                if let State::InProgress(state) = new_state {
                    if !self.finish_solver.can_finish(&state) {
                        continue;
                    }
                    if state.quality > result.quality {
                        let mut actions = SearchTrace::new(current_node, sequence).actions();
                        actions
                            .append(&mut self.finish_solver.get_finish_sequence(&state).unwrap());
                        result = MacroResult {
                            quality: state.quality,
                            actions,
                        };
                    }
                    search_queue.push(SearchNode {
                        state,
                        trace: Some(SearchTrace::new(current_node, sequence)),
                    });
                }
            }
        }

        let seconds = timer.elapsed().as_secs_f32();
        let nodes = explored_nodes.len();
        let nodes_per_sec = nodes as f32 / seconds;
        dbg!(seconds, nodes, nodes_per_sec);

        Some(result)
    }
}
