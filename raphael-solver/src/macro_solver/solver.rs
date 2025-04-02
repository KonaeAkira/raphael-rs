use raphael_sim::*;

use super::search_queue::SearchScore;
use crate::actions::{
    ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS, is_progress_only_state,
    use_action_combo,
};
use crate::macro_solver::fast_lower_bound::fast_lower_bound;
use crate::macro_solver::search_queue::SearchQueue;
use crate::utils::AtomicFlag;
use crate::utils::ScopedTimer;
use crate::{FinishSolver, QualityUbSolver, SolverException, SolverSettings, StepLowerBoundSolver};

use std::vec::Vec;

#[derive(Clone)]
struct Solution {
    score: (SearchScore, u16),
    solver_actions: Vec<ActionCombo>,
}

impl Solution {
    fn actions(&self) -> Vec<Action> {
        let mut actions = Vec::new();
        for solver_action in &self.solver_actions {
            actions.extend_from_slice(solver_action.actions());
        }
        actions
    }
}

type SolutionCallback<'a> = dyn Fn(&[Action]) + 'a;
type ProgressCallback<'a> = dyn Fn(usize) + 'a;

pub struct MacroSolver<'a> {
    settings: SolverSettings,
    solution_callback: Box<SolutionCallback<'a>>,
    progress_callback: Box<ProgressCallback<'a>>,
    interrupt_signal: AtomicFlag,
}

impl<'a> MacroSolver<'a> {
    pub fn new(
        settings: SolverSettings,
        solution_callback: Box<SolutionCallback<'a>>,
        progress_callback: Box<ProgressCallback<'a>>,
        interrupt_signal: AtomicFlag,
    ) -> Self {
        Self {
            settings,
            solution_callback,
            progress_callback,
            interrupt_signal,
        }
    }

    pub fn solve(&mut self) -> Result<Vec<Action>, SolverException> {
        let initial_state = SimulationState::new(&self.settings.simulator_settings);

        let mut finish_solver = FinishSolver::new(self.settings);
        let timer = ScopedTimer::new("Finish Solver");
        if !finish_solver.can_finish(&initial_state) {
            return Err(SolverException::NoSolution);
        }
        drop(timer);

        fn initialize_quality_ub_solver(
            settings: SolverSettings,
            interrupt_signal: AtomicFlag,
        ) -> QualityUbSolver {
            let _timer = ScopedTimer::new("Quality UB Solver");
            let mut seed_state = SimulationState::new(&settings.simulator_settings);
            seed_state.combo = Combo::None;
            let solver = QualityUbSolver::new(settings, interrupt_signal);
            _ = solver.quality_upper_bound(seed_state);
            solver
        }

        fn initialize_step_lb_solver(
            settings: SolverSettings,
            interrupt_signal: AtomicFlag,
        ) -> StepLowerBoundSolver {
            let _timer = ScopedTimer::new("Step LB Solver");
            let mut seed_state = SimulationState::new(&settings.simulator_settings);
            seed_state.combo = Combo::None;
            let step_lb_solver = StepLowerBoundSolver::new(settings, interrupt_signal);
            _ = step_lb_solver.step_lower_bound_with_hint(seed_state, 0);
            step_lb_solver
        }

        let (quality_ub_solver, mut step_lb_solver) = rayon::join(
            || initialize_quality_ub_solver(self.settings, self.interrupt_signal.clone()),
            || initialize_step_lb_solver(self.settings, self.interrupt_signal.clone()),
        );

        let _timer = ScopedTimer::new("Search");
        Ok(self
            .do_solve(
                initial_state,
                &mut finish_solver,
                &quality_ub_solver,
                &mut step_lb_solver,
            )?
            .actions())
    }

    fn do_solve(
        &mut self,
        state: SimulationState,
        finish_solver: &mut FinishSolver,
        quality_ub_solver: &QualityUbSolver,
        step_lb_solver: &mut StepLowerBoundSolver,
    ) -> Result<Solution, SolverException> {
        let mut search_queue = {
            let quality_lower_bound = fast_lower_bound(
                state,
                self.settings,
                self.interrupt_signal.clone(),
                finish_solver,
                quality_ub_solver,
            )?;
            let minimum_score = SearchScore {
                quality_upper_bound: quality_lower_bound,
                ..SearchScore::MIN
            };
            SearchQueue::new(state, minimum_score)
        };

        let mut solution: Option<Solution> = None;

        let mut popped = 0;
        while let Some((state, score, backtrack_id)) = search_queue.pop() {
            if self.interrupt_signal.is_set() {
                return Err(SolverException::Interrupted);
            }

            popped += 1;
            if popped % (1 << 12) == 0 {
                (self.progress_callback)(popped);
            }

            let progress_only = is_progress_only_state(&self.settings, &state);
            let search_actions = match progress_only {
                true => PROGRESS_ONLY_SEARCH_ACTIONS,
                false => FULL_SEARCH_ACTIONS,
            };

            for action in search_actions {
                if let Ok(state) = use_action_combo(&self.settings, state, *action) {
                    if !state.is_final(&self.settings.simulator_settings) {
                        if !finish_solver.can_finish(&state) {
                            // skip this state if it is impossible to max out Progress
                            continue;
                        }

                        search_queue.update_min_score(SearchScore {
                            quality_upper_bound: std::cmp::min(
                                state.quality,
                                self.settings.simulator_settings.max_quality,
                            ),
                            ..SearchScore::MIN
                        });

                        let quality_upper_bound =
                            if state.quality >= self.settings.simulator_settings.max_quality {
                                self.settings.simulator_settings.max_quality
                            } else {
                                std::cmp::min(
                                    score.quality_upper_bound,
                                    quality_ub_solver.quality_upper_bound(state)?,
                                )
                            };

                        let step_lb_hint = score
                            .steps_lower_bound
                            .saturating_sub(score.current_steps + action.steps());
                        let steps_lower_bound = match quality_upper_bound
                            >= self.settings.simulator_settings.max_quality
                        {
                            true => step_lb_solver
                                .step_lower_bound_with_hint(state, step_lb_hint)?
                                .saturating_add(score.current_steps + action.steps()),
                            false => score.current_steps + action.steps(),
                        };

                        search_queue.push(
                            state,
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
                    } else if state.progress >= self.settings.simulator_settings.max_progress {
                        let solution_score = SearchScore {
                            quality_upper_bound: std::cmp::min(
                                state.quality,
                                self.settings.simulator_settings.max_quality,
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

        solution.ok_or(SolverException::NoSolution)
    }
}
