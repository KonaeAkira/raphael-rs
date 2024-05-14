use crate::game::Condition;
use crate::game::{state::InProgress, units::Quality, Action, Settings, State};
use crate::solvers::actions::{
    ActionMask, DURABILITY_ACTIONS, MIXED_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS,
};
use crate::solvers::{FinishSolver, UpperBoundSolver};

use std::time::Instant;
use std::vec::Vec;
use typed_arena::Arena;

use super::*;

const FULL_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .union(MIXED_ACTIONS)
    .union(DURABILITY_ACTIONS);

#[derive(Debug)]
pub struct MacroResult {
    pub quality: Quality,
    pub actions: Vec<Action>,
}

pub struct MacroSolver {
    settings: Settings,
    finish_solver: FinishSolver,
    bound_solver: UpperBoundSolver,
}

impl MacroSolver {
    pub fn new(settings: Settings) -> MacroSolver {
        MacroSolver {
            settings,
            finish_solver: FinishSolver::new(settings),
            bound_solver: UpperBoundSolver::new(settings),
        }
    }

    pub fn solve(&mut self, state: State) -> Option<Vec<Action>> {
        match state {
            State::InProgress(state) => {
                if !self.finish_solver.can_finish(&state) {
                    return None;
                }
                let mut lower_bound = self.bound_solver.quality_upper_bound(state);
                while lower_bound != Quality::new(0) {
                    lower_bound = lower_bound.saturating_sub(Quality::new(20));
                    let result = MacroResult {
                        quality: lower_bound,
                        actions: Vec::new(),
                    };
                    let result = self._do_solve(state, result, &FULL_SEARCH_ACTIONS.actions());
                    if !result.actions.is_empty() {
                        return Some(result.actions);
                    }
                }
                // impossible to get any quality
                Some(self.finish_solver.get_finish_sequence(state).unwrap())
            }
            _ => None,
        }
    }

    fn _do_solve(
        &mut self,
        state: InProgress,
        mut best_result: MacroResult,
        allowed_actions: &[Action],
    ) -> MacroResult {
        let timer = Instant::now();
        let mut finish_solver_rejected_node: usize = 0;
        let mut upper_bound_solver_rejected_nodes: usize = 0;

        let traces: Arena<Option<SearchTrace>> = Arena::new();
        let mut search_queue = SearchQueue::new(self.settings);

        search_queue.push(SearchNode { state, trace: None });

        while let Some(current_node) = search_queue.pop() {
            let trace: &Option<SearchTrace> = traces.alloc(current_node.trace);
            for action in allowed_actions {
                let new_state =
                    current_node
                        .state
                        .use_action(*action, Condition::Normal, &self.settings);
                if let State::InProgress(state) = new_state {
                    if !self.finish_solver.can_finish(&state) {
                        finish_solver_rejected_node += 1;
                        continue;
                    }
                    if best_result.quality >= self.bound_solver.quality_upper_bound(state) {
                        upper_bound_solver_rejected_nodes += 1;
                        continue;
                    }
                    let quality = self.settings.max_quality.sub(state.missing_quality);
                    if quality > best_result.quality {
                        let mut actions = SearchTrace::new(trace, *action).actions();
                        actions.extend(self.finish_solver.get_finish_sequence(state).unwrap());
                        best_result = MacroResult { quality, actions };
                    }
                    search_queue.push(SearchNode {
                        state,
                        trace: Some(SearchTrace::new(trace, *action)),
                    });
                }
            }
        }

        let seconds = timer.elapsed().as_secs_f32();
        let nodes = traces.len();
        let nodes_per_sec = nodes as f32 / seconds;
        dbg!(seconds, nodes, nodes_per_sec);

        dbg!(
            finish_solver_rejected_node,
            upper_bound_solver_rejected_nodes
        );

        dbg!(&best_result);
        best_result
    }
}
