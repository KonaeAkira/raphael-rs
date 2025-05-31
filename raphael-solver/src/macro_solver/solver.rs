use raphael_sim::*;

use super::search_queue::{SearchQueueStats, SearchScore};
use crate::actions::{ActionCombo, FULL_SEARCH_ACTIONS, use_action_combo};
use crate::macro_solver::search_queue::SearchQueue;
use crate::quality_upper_bound_solver::QualityUbSolverStats;
use crate::step_lower_bound_solver::StepLbSolverStats;
use crate::utils::AtomicFlag;
use crate::utils::ScopedTimer;
use crate::{FinishSolver, QualityUbSolver, SolverException, SolverSettings, StepLbSolver};

use std::vec::Vec;

#[derive(Clone)]
struct Solution {
    score: (SearchScore, u32),
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

#[derive(Debug, Clone, Copy)]
pub struct MacroSolverStats {
    pub finish_states: usize,
    pub search_queue_stats: SearchQueueStats,
    pub quality_ub_stats: QualityUbSolverStats,
    pub step_lb_stats: StepLbSolverStats,
}

pub struct MacroSolver<'a> {
    settings: SolverSettings,
    solution_callback: Box<SolutionCallback<'a>>,
    progress_callback: Box<ProgressCallback<'a>>,
    finish_solver: FinishSolver,
    quality_ub_solver: QualityUbSolver,
    step_lb_solver: StepLbSolver,
    search_queue_stats: SearchQueueStats, // stats of last solve
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
            finish_solver: FinishSolver::new(settings),
            quality_ub_solver: QualityUbSolver::new(settings, interrupt_signal.clone()),
            step_lb_solver: StepLbSolver::new(settings, interrupt_signal.clone()),
            search_queue_stats: SearchQueueStats::default(),
            interrupt_signal,
        }
    }

    pub fn solve(&mut self) -> Result<Vec<Action>, SolverException> {
        log::debug!(
            "rayon::current_num_threads() = {}",
            rayon::current_num_threads()
        );

        let _total_time = ScopedTimer::new("Total Time");

        let initial_state = SimulationState::new(&self.settings.simulator_settings);

        let timer = ScopedTimer::new("Finish Solver");
        if !self.finish_solver.can_finish(&initial_state) {
            return Err(SolverException::NoSolution);
        }
        drop(timer);

        let timer = ScopedTimer::new("Quality UB Solver");
        self.quality_ub_solver.precompute();
        drop(timer);

        let timer = ScopedTimer::new("Step Lb Solver");
        self.step_lb_solver.precompute();
        drop(timer);

        Ok(self.do_solve(initial_state)?.actions())
    }

    fn do_solve(&mut self, state: SimulationState) -> Result<Solution, SolverException> {
        let _timer = ScopedTimer::new("Search");
        let mut search_queue = SearchQueue::new(state);
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

            for action in FULL_SEARCH_ACTIONS {
                if let Ok(state) = use_action_combo(&self.settings, state, action) {
                    if !state.is_final(&self.settings.simulator_settings) {
                        if !self.finish_solver.can_finish(&state) {
                            // skip this state if it is impossible to max out Progress
                            continue;
                        }

                        search_queue.update_min_score(SearchScore {
                            quality_upper_bound: std::cmp::min(
                                state.quality,
                                self.settings.max_quality(),
                            ),
                            ..SearchScore::MIN
                        });

                        let quality_upper_bound = if state.quality >= self.settings.max_quality() {
                            self.settings.max_quality()
                        } else {
                            std::cmp::min(
                                score.quality_upper_bound,
                                self.quality_ub_solver.quality_upper_bound(state)?,
                            )
                        };

                        let step_lb_hint = score
                            .steps_lower_bound
                            .saturating_sub(score.current_steps + action.steps());
                        let steps_lower_bound =
                            match quality_upper_bound >= self.settings.max_quality() {
                                true => self
                                    .step_lb_solver
                                    .step_lower_bound(state, step_lb_hint)?
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
                            action,
                            backtrack_id,
                        );
                    } else if state.progress >= self.settings.max_progress() {
                        let solution_score = SearchScore {
                            quality_upper_bound: std::cmp::min(
                                state.quality,
                                self.settings.max_quality(),
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
                                    .chain(std::iter::once(action))
                                    .collect(),
                            });
                            (self.solution_callback)(&solution.as_ref().unwrap().actions());
                        }
                    }
                }
            }
        }

        self.search_queue_stats = search_queue.runtime_stats();
        solution.ok_or(SolverException::NoSolution)
    }

    pub fn runtime_stats(&self) -> MacroSolverStats {
        MacroSolverStats {
            finish_states: self.finish_solver.num_states(),
            search_queue_stats: self.search_queue_stats,
            quality_ub_stats: self.quality_ub_solver.runtime_stats(),
            step_lb_stats: self.step_lb_solver.runtime_stats(),
        }
    }
}
