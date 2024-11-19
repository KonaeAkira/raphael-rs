use simulator::{Action, ActionMask, Condition, Settings, SimulationState};

use super::search_queue::SearchScore;
use crate::actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS};
use crate::branch_pruning::{is_progress_only_state, strip_quality_effects};
use crate::macro_solver::fast_lower_bound::fast_lower_bound;
use crate::macro_solver::search_queue::SearchQueue;
use crate::utils::NamedTimer;
use crate::{FinishSolver, QualityUpperBoundSolver, StepLowerBoundSolver};

use std::vec::Vec;

use log::debug;

const FULL_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .union(DURABILITY_ACTIONS);

const PROGRESS_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::DelicateSynthesis);

#[derive(Clone)]
struct Solution {
    score: (SearchScore, u16),
    actions: Vec<Action>,
}

type SolutionCallback<'a> = dyn Fn(&[Action]) + 'a;
type ProgressCallback<'a> = dyn Fn(usize) + 'a;

pub struct MacroSolver<'a> {
    settings: Settings,
    backload_progress: bool,
    unsound_branch_pruning: bool,
    finish_solver: FinishSolver,
    quality_upper_bound_solver: QualityUpperBoundSolver,
    step_lower_bound_solver: StepLowerBoundSolver,
    solution_callback: Box<SolutionCallback<'a>>,
    progress_callback: Box<ProgressCallback<'a>>,
}

impl<'a> MacroSolver<'a> {
    pub fn new(
        settings: Settings,
        backload_progress: bool,
        unsound_branch_pruning: bool,
        solution_callback: Box<SolutionCallback<'a>>,
        progress_callback: Box<ProgressCallback<'a>>,
    ) -> MacroSolver<'a> {
        MacroSolver {
            settings,
            backload_progress,
            unsound_branch_pruning,
            finish_solver: FinishSolver::new(settings),
            quality_upper_bound_solver: QualityUpperBoundSolver::new(
                settings,
                backload_progress,
                unsound_branch_pruning,
            ),
            step_lower_bound_solver: StepLowerBoundSolver::new(
                settings,
                backload_progress,
                unsound_branch_pruning,
            ),
            solution_callback,
            progress_callback,
        }
    }

    /// Returns a list of Actions that maximizes Quality of the completed state.
    /// Returns `None` if the state cannot be completed (i.e. cannot max out Progress).
    pub fn solve(&mut self, state: SimulationState) -> Option<Vec<Action>> {
        let timer = NamedTimer::new("Finish solver");
        if !self.finish_solver.can_finish(&state) {
            return None;
        }
        drop(timer);

        let _timer = NamedTimer::new("Full search");
        self.do_solve(state)
    }

    fn do_solve(&mut self, state: SimulationState) -> Option<Vec<Action>> {
        let mut search_queue = {
            let _timer = NamedTimer::new("Initial upper bound");
            let quality_upper_bound = self.quality_upper_bound_solver.quality_upper_bound(state);
            let step_lower_bound = if quality_upper_bound >= self.settings.max_quality {
                self.step_lower_bound_solver.step_lower_bound(state)
            } else {
                1 // quality dominates the search score, so no need to query the step solver
            };
            let initial_score = SearchScore::new(quality_upper_bound, step_lower_bound, 0);
            let quality_lower_bound = fast_lower_bound(
                state,
                &self.settings,
                &mut self.finish_solver,
                &mut self.quality_upper_bound_solver,
            );
            let minimum_score = SearchScore::new(quality_lower_bound, u8::MAX, u8::MAX);
            SearchQueue::new(state, initial_score, minimum_score)
        };

        let mut solution: Option<Solution> = None;

        let mut popped = 0;
        while let Some((state, score, backtrack_id)) = search_queue.pop() {
            popped += 1;
            if popped % (1 << 14) == 0 {
                (self.progress_callback)(popped);
            }

            let progress_only =
                is_progress_only_state(&state, self.backload_progress, self.unsound_branch_pruning);
            let search_actions = match progress_only {
                true => PROGRESS_SEARCH_ACTIONS,
                false => FULL_SEARCH_ACTIONS,
            };

            let current_steps = search_queue.steps(backtrack_id);

            for action in search_actions.actions_iter() {
                if let Ok(state) = state.use_action(action, Condition::Normal, &self.settings) {
                    if !state.is_final(&self.settings) {
                        if !self.finish_solver.can_finish(&state) {
                            // skip this state if it is impossible to max out Progress
                            continue;
                        }

                        search_queue.update_min_score(SearchScore::new(
                            std::cmp::min(state.quality, self.settings.max_quality),
                            u8::MAX,
                            u8::MAX,
                        ));

                        let quality_upper_bound = if state.quality >= self.settings.max_quality {
                            self.settings.max_quality
                        } else {
                            std::cmp::min(
                                score.quality,
                                self.quality_upper_bound_solver.quality_upper_bound(state),
                            )
                        };

                        let step_lb_hint = score.steps.saturating_sub(current_steps + 1);
                        let step_lower_bound = if quality_upper_bound >= self.settings.max_quality {
                            self.step_lower_bound_solver
                                .step_lower_bound_with_hint(state, step_lb_hint)
                                .saturating_add(current_steps + 1)
                        } else {
                            current_steps + 1
                        };

                        let progress_only = is_progress_only_state(
                            &state,
                            self.backload_progress,
                            self.unsound_branch_pruning,
                        );
                        search_queue.push(
                            if progress_only {
                                strip_quality_effects(state)
                            } else {
                                state
                            },
                            SearchScore::new(
                                quality_upper_bound,
                                step_lower_bound,
                                score.duration + action.time_cost() as u8,
                            ),
                            action,
                            backtrack_id,
                        );
                    } else if state.progress >= self.settings.max_progress {
                        let solution_score = SearchScore::new(
                            std::cmp::min(state.quality, self.settings.max_quality),
                            current_steps + 1,
                            score.duration,
                        );
                        search_queue.update_min_score(solution_score);
                        if solution.is_none()
                            || solution.as_ref().unwrap().score < (solution_score, state.quality)
                        {
                            solution = Some(Solution {
                                score: (solution_score, state.quality),
                                actions: search_queue
                                    .backtrack(backtrack_id)
                                    .chain(std::iter::once(action))
                                    .collect(),
                            });
                            (self.solution_callback)(&solution.as_ref().unwrap().actions);
                            (self.progress_callback)(popped);
                        }
                    }
                }
            }
        }

        if let Some(solution) = solution {
            debug!("Found solution actions: {:?}", &solution.actions);
            Some(solution.actions)
        } else {
            None
        }
    }
}
