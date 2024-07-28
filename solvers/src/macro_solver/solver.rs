use simulator::state::InProgress;
use simulator::{Action, ActionMask, Condition, Settings};

use super::quick_search::quick_search;
use super::search_queue::SearchScore;
use crate::actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS};
use crate::macro_solver::fast_lower_bound::fast_lower_bound;
use crate::macro_solver::search_queue::SearchQueue;
use crate::utils::NamedTimer;
use crate::{FinishSolver, UpperBoundSolver};

use std::vec::Vec;

const FULL_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .union(DURABILITY_ACTIONS);

const PROGRESS_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::DelicateSynthesis);

#[derive(Clone)]
struct Solution {
    score: SearchScore,
    actions: Vec<Action>,
}

pub struct MacroSolver<'a> {
    settings: Settings,
    finish_solver: FinishSolver,
    bound_solver: UpperBoundSolver,
    solution_callback: Box<dyn Fn(&[Action]) + 'a>,
    progress_callback: Box<dyn Fn(f32) + 'a>,
}

impl<'a> MacroSolver<'a> {
    pub fn new(
        settings: Settings,
        solution_callback: Box<dyn Fn(&[Action]) + 'a>,
        progress_callback: Box<dyn Fn(f32) + 'a>,
    ) -> MacroSolver<'a> {
        MacroSolver {
            settings,
            finish_solver: FinishSolver::new(settings),
            bound_solver: UpperBoundSolver::new(settings),
            solution_callback,
            progress_callback,
        }
    }

    /// Returns a list of Actions that maximizes Quality of the completed state.
    /// Returns `None` if the state cannot be completed (i.e. cannot max out Progress).
    /// The solver makes an effort to produce a short solution, but it is not (yet) guaranteed to be the shortest solution.
    pub fn solve(&mut self, state: InProgress, backload_progress: bool) -> Option<Vec<Action>> {
        let timer = NamedTimer::new("Finish solver");
        if !self.finish_solver.can_finish(&state) {
            return None;
        }
        drop(timer);

        if let Some(actions) = quick_search(
            state,
            &self.settings,
            &mut self.finish_solver,
            &mut self.bound_solver,
        ) {
            return Some(actions);
        }

        let _timer = NamedTimer::new("Full search");
        self.do_solve(state, backload_progress)
    }

    fn do_solve(&mut self, state: InProgress, backload_progress: bool) -> Option<Vec<Action>> {
        let mut finish_solver_rejected_nodes: usize = 0;

        let initial_score = SearchScore::new(
            self.bound_solver.quality_upper_bound(state),
            0,
            0,
            &self.settings,
        );
        let quality_lower_bound = fast_lower_bound(
            state,
            &self.settings,
            &mut self.finish_solver,
            &mut self.bound_solver,
        );
        let minimum_score = SearchScore::new(quality_lower_bound, u8::MAX, u8::MAX, &self.settings);
        let mut search_queue = SearchQueue::new(state, initial_score, minimum_score, self.settings);

        let mut solution: Option<Solution> = None;

        let mut popped = 0;
        while let Some((state, score, backtrack_id)) = search_queue.pop() {
            popped += 1;
            if popped % (1 << 16) == 0 {
                (self.progress_callback)(search_queue.progress_estimate());
            }
            let mut search_actions = match backload_progress
                && state.raw_state().missing_progress != self.settings.max_progress
            {
                true => PROGRESS_SEARCH_ACTIONS.intersection(self.settings.allowed_actions),
                false => FULL_SEARCH_ACTIONS.intersection(self.settings.allowed_actions),
            };
            if state.raw_state().get_quality() >= self.settings.max_quality {
                search_actions = search_actions.minus(QUALITY_ACTIONS);
            }
            for action in search_actions.actions_iter() {
                if let Ok(state) = state.use_action(action, Condition::Normal, &self.settings) {
                    if let Ok(in_progress) = InProgress::try_from(state) {
                        if !self.finish_solver.can_finish(&in_progress) {
                            // skip this state if it is impossible to max out Progress
                            finish_solver_rejected_nodes += 1;
                            continue;
                        }

                        search_queue.update_min_score(SearchScore::new(
                            state.get_quality(),
                            u8::MAX,
                            u8::MAX,
                            &self.settings,
                        ));

                        let quality_upper_bound =
                            if state.get_quality() >= self.settings.max_quality {
                                state.get_quality()
                            } else {
                                self.bound_solver.quality_upper_bound(in_progress)
                            };
                        search_queue.push(
                            in_progress,
                            SearchScore::new(
                                quality_upper_bound,
                                score.duration + action.time_cost() as u8,
                                score.steps + 1,
                                &self.settings,
                            ),
                            action,
                            backtrack_id,
                        );
                    } else if state.missing_progress == 0 {
                        let solution_score = SearchScore::new(
                            state.get_quality(),
                            score.duration,
                            score.steps,
                            &self.settings,
                        );
                        search_queue.update_min_score(solution_score);
                        if solution.is_none() || solution.as_ref().unwrap().score < solution_score {
                            solution = Some(Solution {
                                score: solution_score,
                                actions: search_queue
                                    .backtrack(backtrack_id)
                                    .chain(std::iter::once(action))
                                    .collect(),
                            });
                            (self.solution_callback)(&solution.as_ref().unwrap().actions);
                            (self.progress_callback)(search_queue.progress_estimate());
                        }
                    }
                }
            }
        }

        if let Some(solution) = solution {
            dbg!(&solution.actions, finish_solver_rejected_nodes);
            Some(solution.actions)
        } else {
            None
        }
    }
}
