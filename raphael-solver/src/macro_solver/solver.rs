use raphael_sim::*;
use rayon::prelude::*;

use super::search_queue::{SearchQueueStats, SearchScore};
use crate::actions::{ActionCombo, FULL_SEARCH_ACTIONS, use_action_combo};
use crate::finish_solver::FinishSolverShard;
use crate::macro_solver::search_queue::SearchQueue;
use crate::quality_upper_bound_solver::{QualityUbSolverShard, QualityUbSolverStats};
use crate::step_lower_bound_solver::{StepLbSolverShard, StepLbSolverStats};
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
        self.quality_ub_solver.precompute()?;
        drop(timer);

        let timer = ScopedTimer::new("Step LB Solver");
        self.step_lb_solver.precompute()?;
        drop(timer);

        let timer = ScopedTimer::new("Search");
        let actions = self.do_solve(initial_state)?.actions();
        drop(timer);

        log::debug!("{:?}", self.runtime_stats());

        Ok(actions)
    }

    fn do_solve(&mut self, state: SimulationState) -> Result<Solution, SolverException> {
        let mut search_queue = SearchQueue::new(state);
        let mut solution: Option<Solution> = None;

        while let Some(batch) = search_queue.pop_batch() {
            if self.interrupt_signal.is_set() {
                return Err(SolverException::Interrupted);
            }

            let create_thread_data = || ThreadData {
                settings: &self.settings,
                finish_solver_shard: self.finish_solver.create_shard(),
                quality_ub_solver_shard: self.quality_ub_solver.create_shard(),
                step_lb_solver_shard: self.step_lb_solver.create_shard(),
                score_lb: SearchScore::MIN,
                states: Vec::new(),
            };

            let thread_results = batch
                .into_par_iter()
                .try_fold(create_thread_data, thread_search_task)
                .collect::<Result<Vec<_>, SolverException>>()?;

            for thread_data in &thread_results {
                search_queue.update_min_score(thread_data.score_lb);
            }

            for thread_data in &thread_results {
                for &(state, score, action, parent_id) in &thread_data.states {
                    if state.progress >= self.settings.max_progress() {
                        if solution
                            .as_ref()
                            .is_none_or(|solution| solution.score < (score, state.quality))
                        {
                            solution = Some(Solution {
                                score: (score, state.quality),
                                solver_actions: search_queue
                                    .backtrack(parent_id)
                                    .chain(std::iter::once(action))
                                    .collect(),
                            });
                            (self.solution_callback)(&solution.as_ref().unwrap().actions());
                        }
                    } else {
                        search_queue.try_push(state, score, action, parent_id)?
                    }
                }
            }

            // Map each `ThreadData` instance to just the hashmaps containing all the newly solved states.
            // This drops all shared references to `self` which allows us to mutate the inner solvers.
            let solved_states_per_thread = thread_results
                .into_iter()
                .map(|thread_data| {
                    (
                        thread_data.finish_solver_shard.solved_states(),
                        thread_data.quality_ub_solver_shard.solved_states(),
                        thread_data.step_lb_solver_shard.solved_states(),
                    )
                })
                .collect::<Vec<_>>();
            for solved_states in solved_states_per_thread {
                self.finish_solver.extend_solved_states(solved_states.0);
                self.quality_ub_solver.extend_solved_states(solved_states.1);
                self.step_lb_solver.extend_solved_states(solved_states.2);
            }

            (self.progress_callback)(search_queue.runtime_stats().processed_nodes);
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

struct ThreadData<'a> {
    settings: &'a SolverSettings,
    finish_solver_shard: FinishSolverShard<'a>,
    quality_ub_solver_shard: QualityUbSolverShard<'a>,
    step_lb_solver_shard: StepLbSolverShard<'a>,
    score_lb: SearchScore,
    states: Vec<(SimulationState, SearchScore, ActionCombo, usize)>,
}

impl<'a> ThreadData<'a> {
    fn update_min_score(&mut self, score: SearchScore) {
        self.score_lb = std::cmp::max(self.score_lb, score);
    }
}

fn thread_search_task(
    mut thread_data: ThreadData<'_>,
    (state, score, backtrack_id): (SimulationState, SearchScore, usize),
) -> Result<ThreadData<'_>, SolverException> {
    for action in FULL_SEARCH_ACTIONS {
        if let Ok(state) = use_action_combo(thread_data.settings, state, action) {
            if !state.is_final(&thread_data.settings.simulator_settings) {
                if !thread_data.finish_solver_shard.can_finish(&state) {
                    continue;
                }

                thread_data.update_min_score(SearchScore {
                    quality_upper_bound: std::cmp::min(
                        state.quality,
                        thread_data.settings.max_quality(),
                    ),
                    ..SearchScore::MIN
                });

                let quality_upper_bound = if state.quality >= thread_data.settings.max_quality() {
                    thread_data.settings.max_quality()
                } else {
                    std::cmp::min(
                        score.quality_upper_bound,
                        thread_data
                            .quality_ub_solver_shard
                            .quality_upper_bound(state)?,
                    )
                };

                if !thread_data.settings.allow_non_max_quality_solutions
                    && quality_upper_bound < thread_data.settings.max_quality()
                {
                    continue;
                }

                let step_lb_hint = score
                    .steps_lower_bound
                    .saturating_sub(score.current_steps + action.steps());
                let steps_lower_bound =
                    match quality_upper_bound >= thread_data.settings.max_quality() {
                        true => thread_data
                            .step_lb_solver_shard
                            .step_lower_bound(state, step_lb_hint)?
                            .saturating_add(score.current_steps + action.steps()),
                        false => score.current_steps + action.steps(),
                    };

                thread_data.states.push((
                    state,
                    SearchScore {
                        quality_upper_bound,
                        steps_lower_bound,
                        duration_lower_bound: score.current_duration + action.duration() + 3,
                        current_steps: score.current_steps + action.steps(),
                        current_duration: score.current_duration + action.duration(),
                    },
                    action,
                    backtrack_id,
                ));
            } else if state.progress >= thread_data.settings.max_progress() {
                let solution_score = SearchScore {
                    quality_upper_bound: std::cmp::min(
                        state.quality,
                        thread_data.settings.max_quality(),
                    ),
                    steps_lower_bound: score.current_steps + action.steps(),
                    duration_lower_bound: score.current_duration + action.duration(),
                    current_steps: score.current_steps + action.steps(),
                    current_duration: score.current_duration + action.duration(),
                };
                thread_data.update_min_score(solution_score);
                thread_data
                    .states
                    .push((state, solution_score, action, backtrack_id));
            }
        }
    }
    Ok(thread_data)
}
