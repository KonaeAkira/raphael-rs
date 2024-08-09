use simulator::{Action, ActionMask, Condition, Settings, SimulationState};

use super::quick_search::quick_search;
use super::search_queue::SearchScore;
use crate::actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS};
use crate::macro_solver::fast_lower_bound::fast_lower_bound;
use crate::macro_solver::search_queue::SearchQueue;
use crate::utils::NamedTimer;
use crate::{FinishSolver, QualityUpperBoundSolver, StepLowerBoundSolver};

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

type SolutionCallback<'a> = dyn Fn(&[Action]) + 'a;
type ProgressCallback<'a> = dyn Fn(f32) + 'a;

pub struct MacroSolver<'a> {
    settings: Settings,
    finish_solver: FinishSolver,
    quality_upper_bound_solver: QualityUpperBoundSolver,
    step_lower_bound_solver: StepLowerBoundSolver,
    solution_callback: Box<SolutionCallback<'a>>,
    progress_callback: Box<ProgressCallback<'a>>,
}

impl<'a> MacroSolver<'a> {
    pub fn new(
        settings: Settings,
        solution_callback: Box<SolutionCallback<'a>>,
        progress_callback: Box<ProgressCallback<'a>>,
    ) -> MacroSolver<'a> {
        MacroSolver {
            settings,
            finish_solver: FinishSolver::new(settings),
            quality_upper_bound_solver: QualityUpperBoundSolver::new(settings),
            step_lower_bound_solver: StepLowerBoundSolver::new(settings),
            solution_callback,
            progress_callback,
        }
    }

    /// Returns a list of Actions that maximizes Quality of the completed state.
    /// Returns `None` if the state cannot be completed (i.e. cannot max out Progress).
    /// The solver makes an effort to produce a short solution, but it is not (yet) guaranteed to be the shortest solution.
    pub fn solve(
        &mut self,
        state: SimulationState,
        backload_progress: bool,
        minimize_steps: bool,
    ) -> Option<Vec<Action>> {
        let timer = NamedTimer::new("Finish solver");
        if !self.finish_solver.can_finish(&state) {
            return None;
        }
        drop(timer);

        if !minimize_steps {
            if let Some(actions) = quick_search(
                state,
                &self.settings,
                &mut self.finish_solver,
                &mut self.quality_upper_bound_solver,
            ) {
                return Some(actions);
            }
        }

        let _timer = NamedTimer::new("Full search");
        self.do_solve(state, backload_progress)
    }

    fn do_solve(&mut self, state: SimulationState, backload_progress: bool) -> Option<Vec<Action>> {
        let mut search_queue = {
            let quality_upper_bound = self.quality_upper_bound_solver.quality_upper_bound(state);
            let step_lower_bound = if quality_upper_bound >= self.settings.max_quality {
                self.step_lower_bound_solver.step_lower_bound(state, true)
            } else {
                1 // quality dominates the search score, so no need to query the step solver
            };
            let initial_score =
                SearchScore::new(quality_upper_bound, 0, step_lower_bound, &self.settings);
            let quality_lower_bound = fast_lower_bound(
                state,
                &self.settings,
                &mut self.finish_solver,
                &mut self.quality_upper_bound_solver,
            );
            let minimum_score =
                SearchScore::new(quality_lower_bound, u8::MAX, u8::MAX, &self.settings);
            SearchQueue::new(state, initial_score, minimum_score, self.settings)
        };

        let mut solution: Option<Solution> = None;

        let mut popped = 0;
        while let Some((state, score, backtrack_id)) = search_queue.pop() {
            popped += 1;
            if popped % (1 << 16) == 0 {
                (self.progress_callback)(search_queue.progress_estimate());
            }
            let mut search_actions = match backload_progress && state.progress != 0 {
                true => PROGRESS_SEARCH_ACTIONS.intersection(self.settings.allowed_actions),
                false => FULL_SEARCH_ACTIONS.intersection(self.settings.allowed_actions),
            };
            if state.get_quality() >= self.settings.max_quality {
                search_actions = search_actions.minus(QUALITY_ACTIONS);
            }

            let current_steps = search_queue.steps(backtrack_id);

            for action in search_actions.actions_iter() {
                if let Ok(state) = state.use_action(action, Condition::Normal, &self.settings) {
                    if !state.is_final(&self.settings) {
                        if !self.finish_solver.can_finish(&state) {
                            // skip this state if it is impossible to max out Progress
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
                                self.quality_upper_bound_solver.quality_upper_bound(state)
                            };

                        let step_lower_bound = if quality_upper_bound >= self.settings.max_quality {
                            let fast_mode = popped < 200_000;
                            current_steps
                                + 1
                                + self
                                    .step_lower_bound_solver
                                    .step_lower_bound(state, fast_mode)
                        } else {
                            current_steps + 1
                        };

                        search_queue.push(
                            state,
                            SearchScore::new(
                                quality_upper_bound,
                                score.duration + action.time_cost() as u8,
                                step_lower_bound,
                                &self.settings,
                            ),
                            action,
                            backtrack_id,
                        );
                    } else if state.progress >= self.settings.max_progress {
                        let solution_score = SearchScore::new(
                            state.get_quality(),
                            score.duration,
                            current_steps + 1,
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
            dbg!(&solution.actions);
            Some(solution.actions)
        } else {
            None
        }
    }
}
