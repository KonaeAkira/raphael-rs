use simulator::{Action, Settings, SimulationState};

use super::search_queue::SearchScore;
use crate::actions::{
    use_solver_action, SolverAction, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS,
};
use crate::branch_pruning::{is_progress_only_state, strip_quality_effects};
use crate::macro_solver::fast_lower_bound::fast_lower_bound;
use crate::macro_solver::search_queue::SearchQueue;
use crate::utils::AtomicFlag;
use crate::utils::NamedTimer;
use crate::{FinishSolver, QualityUpperBoundSolver, StepLowerBoundSolver};

use std::vec::Vec;

#[derive(Clone)]
struct Solution {
    score: (SearchScore, u16),
    solver_actions: Vec<SolverAction>,
}

impl Solution {
    fn actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        for solver_action in self.solver_actions.iter() {
            actions.extend_from_slice(solver_action.actions());
        }
        actions
    }
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
    interrupt_signal: AtomicFlag,
}

impl<'a> MacroSolver<'a> {
    pub fn new(
        settings: Settings,
        backload_progress: bool,
        unsound_branch_pruning: bool,
        solution_callback: Box<SolutionCallback<'a>>,
        progress_callback: Box<ProgressCallback<'a>>,
        interrupt_signal: AtomicFlag,
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
                interrupt_signal.clone(),
            ),
            step_lower_bound_solver: StepLowerBoundSolver::new(
                settings,
                backload_progress,
                unsound_branch_pruning,
                interrupt_signal.clone(),
            ),
            solution_callback,
            progress_callback,
            interrupt_signal,
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
        Some(self.do_solve(state)?.actions())
    }

    fn do_solve(&mut self, state: SimulationState) -> Option<Solution> {
        let mut search_queue = {
            let _timer = NamedTimer::new("Initial upper bound");
            let quality_upper_bound = self.quality_upper_bound_solver.quality_upper_bound(state)?;
            let steps_lower_bound = match quality_upper_bound >= self.settings.max_quality {
                true => self.step_lower_bound_solver.step_lower_bound(state)?,
                false => 1, // quality dominates the search score, so no need to query the step solver
            };
            let initial_score = SearchScore {
                quality_upper_bound,
                steps_lower_bound,
                duration_lower_bound: 0,
                current_steps: 0,
                current_duration: 0,
            };
            let quality_lower_bound = fast_lower_bound(
                state,
                &self.settings,
                &mut self.finish_solver,
                &mut self.quality_upper_bound_solver,
            )?;
            let minimum_score = SearchScore {
                quality_upper_bound: quality_lower_bound,
                steps_lower_bound: u8::MAX,
                duration_lower_bound: u8::MAX,
                current_steps: u8::MAX,
                current_duration: u8::MAX,
            };
            SearchQueue::new(state, initial_score, minimum_score)
        };

        let mut solution: Option<Solution> = None;

        let mut popped = 0;
        while let Some((state, score, backtrack_id)) = search_queue.pop() {
            if self.interrupt_signal.is_set() {
                return None;
            }

            popped += 1;
            if popped % (1 << 12) == 0 {
                (self.progress_callback)(popped);
            }

            let progress_only =
                is_progress_only_state(&state, self.backload_progress, self.unsound_branch_pruning);
            let search_actions = match progress_only {
                true => PROGRESS_ONLY_SEARCH_ACTIONS,
                false => FULL_SEARCH_ACTIONS,
            };

            for action in search_actions.iter() {
                if let Ok(state) = use_solver_action(&self.settings, state, *action) {
                    if !state.is_final(&self.settings) {
                        if !self.finish_solver.can_finish(&state) {
                            // skip this state if it is impossible to max out Progress
                            continue;
                        }

                        search_queue.update_min_score(SearchScore {
                            quality_upper_bound: std::cmp::min(
                                state.quality,
                                self.settings.max_quality,
                            ),
                            steps_lower_bound: u8::MAX,
                            duration_lower_bound: u8::MAX,
                            current_steps: u8::MAX,
                            current_duration: u8::MAX,
                        });

                        let quality_upper_bound = if state.quality >= self.settings.max_quality {
                            self.settings.max_quality
                        } else {
                            std::cmp::min(
                                score.quality_upper_bound,
                                self.quality_upper_bound_solver.quality_upper_bound(state)?,
                            )
                        };

                        let step_lb_hint = score
                            .steps_lower_bound
                            .saturating_sub(score.current_steps + action.steps());
                        let steps_lower_bound =
                            match quality_upper_bound >= self.settings.max_quality {
                                true => self
                                    .step_lower_bound_solver
                                    .step_lower_bound_with_hint(state, step_lb_hint)?
                                    .saturating_add(score.current_steps + action.steps()),
                                false => score.current_steps + action.steps(),
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
                            SearchScore {
                                quality_upper_bound,
                                steps_lower_bound,
                                duration_lower_bound: score.current_duration
                                    + action.duration()
                                    + 3,
                                current_steps: score.current_steps + action.steps(),
                                current_duration: score.current_duration + action.duration(),
                            },
                            *action,
                            backtrack_id,
                        );
                    } else if state.progress >= self.settings.max_progress {
                        let solution_score = SearchScore {
                            quality_upper_bound: std::cmp::min(
                                state.quality,
                                self.settings.max_quality,
                            ),
                            steps_lower_bound: score.current_steps + action.steps(),
                            duration_lower_bound: score.current_duration + action.duration(),
                            current_steps: score.current_steps + action.steps(),
                            current_duration: score.current_duration + action.duration(),
                        };
                        search_queue.update_min_score(solution_score);
                        if solution.is_none()
                            || solution.as_ref().unwrap().score < (solution_score, state.quality)
                        {
                            solution = Some(Solution {
                                score: (solution_score, state.quality),
                                solver_actions: search_queue
                                    .backtrack(backtrack_id)
                                    .chain(std::iter::once(*action))
                                    .collect(),
                            });
                            (self.solution_callback)(&solution.as_ref().unwrap().actions());
                        }
                    }
                }
            }
        }

        solution
    }
}
