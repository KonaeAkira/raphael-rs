use bump_scope::BumpPool;
use raphael_sim::*;
use rayon::prelude::*;

use super::search_queue::{SearchQueueStats, SearchScore};
use crate::actions::{ActionCombo, FULL_SEARCH_ACTIONS, use_action_combo};
use crate::finish_solver::FinishSolverStats;
use crate::macro_solver::search_queue::{Batch, SearchQueue};
use crate::quality_upper_bound_solver::{
    QualityUbSolverShard, QualityUbSolverStats, QualityUbStates,
};
use crate::step_lower_bound_solver::{StepLbSolverShard, StepLbSolverStats, StepLbStates};
use crate::utils::AtomicFlag;
use crate::utils::ScopedTimer;
use crate::{FinishSolver, QualityUbSolver, SolverException, SolverSettings, StepLbSolver};

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

#[derive(Debug, Default, Clone, Copy)]
pub struct MacroSolverStats {
    pub search_queue_stats: SearchQueueStats,
    pub finish_solver_stats: FinishSolverStats,
    pub quality_ub_stats: QualityUbSolverStats,
    pub step_lb_stats: StepLbSolverStats,
}

pub struct MacroSolver<'a> {
    settings: SolverSettings,
    solution_callback: Box<SolutionCallback<'a>>,
    progress_callback: Box<ProgressCallback<'a>>,
    finish_solver: FinishSolver,
    interrupt_signal: AtomicFlag,
    last_solve_runtime_stats: MacroSolverStats,
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
            interrupt_signal,
            last_solve_runtime_stats: MacroSolverStats::default(),
        }
    }

    pub fn solve(&mut self) -> Result<Vec<Action>, SolverException> {
        log::debug!(
            "rayon::current_num_threads() = {}",
            rayon::current_num_threads()
        );

        self.last_solve_runtime_stats = MacroSolverStats::default();
        let allocator = BumpPool::default();
        let mut quality_ub_solver =
            QualityUbSolver::new(self.settings, self.interrupt_signal.clone(), &allocator);
        let mut step_lb_solver =
            StepLbSolver::new(self.settings, self.interrupt_signal.clone(), &allocator);

        let _total_time = ScopedTimer::new("Total Time");

        let initial_state = SimulationState::new(&self.settings.simulator_settings);

        let timer = ScopedTimer::new("Finish Solver");
        self.finish_solver.precompute()?;
        if !self.finish_solver.can_finish(&initial_state)? {
            self.last_solve_runtime_stats.finish_solver_stats = self.finish_solver.runtime_stats();
            return Err(SolverException::NoSolution);
        }
        drop(timer);

        let timer = ScopedTimer::new("Quality UB Solver");
        quality_ub_solver.precompute()?;
        drop(timer);

        // The StepLbSolver is only queried when a state has the potential to reach max_quality.
        // If the quality upper-bound of the initial state is less than max_quality, then no
        // subsequent state can reach max_quality, which in turn means the StepLbSolver is not needed.
        let mut quality_ub_solver_shard = quality_ub_solver.create_shard();
        let initial_state_quality_ub =
            quality_ub_solver_shard.quality_upper_bound(initial_state)?;
        quality_ub_solver.extend_solved_states(quality_ub_solver_shard.solved_states());
        if initial_state_quality_ub >= self.settings.max_quality() {
            let _timer = ScopedTimer::new("Step LB Solver");
            step_lb_solver.precompute()?;
        }

        let timer = ScopedTimer::new("Search");
        let actions = self
            .do_solve(&mut quality_ub_solver, &mut step_lb_solver, initial_state)?
            .actions();
        drop(timer);

        log::debug!("{:?}", self.runtime_stats());

        Ok(actions)
    }

    fn do_solve<'alloc>(
        &mut self,
        quality_ub_solver: &mut QualityUbSolver<'alloc>,
        step_lb_solver: &mut StepLbSolver<'alloc>,
        state: SimulationState,
    ) -> Result<Solution, SolverException> {
        let mut search_queue = SearchQueue::new(self.settings, state);
        let mut solution: Option<Solution> = None;
        let mut min_accepted_score = SearchScore::MIN;

        while let Some(Batch {
            score,
            nodes: batch,
        }) = search_queue.pop_batch()
            && score >= min_accepted_score
        {
            if self.interrupt_signal.is_set() {
                return Err(SolverException::Interrupted);
            }

            let create_worker_data = || WorkerData {
                settings: &self.settings,
                finish_solver: &self.finish_solver,
                quality_ub_solver_shard: quality_ub_solver.create_shard(),
                step_lb_solver_shard: step_lb_solver.create_shard(),
                search_queue: &search_queue,
                min_accepted_score,
                candidate_states: Vec::new(),
                best_intermediate_solution: None,
            };

            let worker_results = batch
                .into_par_iter()
                .try_fold(
                    create_worker_data,
                    |mut worker_data, (state, backtrack_id)| {
                        worker_data.process_state(state, score, backtrack_id)?;
                        Ok(worker_data)
                    },
                )
                .collect::<Result<Vec<_>, SolverException>>()?;

            // Finalize the workers to drop all shared references to `self` to satisfy the borrow checker.
            let worker_results = worker_results
                .into_iter()
                .map(WorkerData::finalize)
                .collect::<Vec<_>>();

            // Update the current best intermediate solution.
            for worker_data in &worker_results {
                if let Some(worker_solution) = worker_data.best_intermediate_solution.as_ref()
                    && Some(worker_solution.score) > solution.as_ref().map(|s| s.score)
                {
                    solution = Some(worker_solution.clone());
                    (self.solution_callback)(&solution.as_ref().unwrap().actions());
                }
            }

            min_accepted_score = worker_results
                .iter()
                .map(|result| result.min_accepted_score)
                .max()
                .unwrap_or(min_accepted_score);
            search_queue.drop_nodes_below_score(min_accepted_score);

            // Add all eligible candidate states to the search queue.
            for worker_data in &worker_results {
                for &(score, action, parent_id) in &worker_data.candidate_states {
                    if score >= min_accepted_score {
                        search_queue.push(score, action, parent_id)?;
                    }
                }
            }

            // Extend inner solvers with local states from all workers.
            for worker_result in worker_results {
                quality_ub_solver.extend_solved_states(worker_result.quality_ub_states);
                step_lb_solver.extend_solved_states(worker_result.step_lb_states);
            }

            (self.progress_callback)(search_queue.runtime_stats().processed_nodes);
        }

        self.last_solve_runtime_stats = MacroSolverStats {
            search_queue_stats: search_queue.runtime_stats(),
            finish_solver_stats: self.finish_solver.runtime_stats(),
            quality_ub_stats: quality_ub_solver.runtime_stats(),
            step_lb_stats: step_lb_solver.runtime_stats(),
        };

        solution.ok_or(SolverException::NoSolution)
    }

    pub fn runtime_stats(&self) -> MacroSolverStats {
        self.last_solve_runtime_stats
    }
}

struct WorkerResult<'alloc> {
    quality_ub_states: QualityUbStates<'alloc>,
    step_lb_states: StepLbStates<'alloc>,
    min_accepted_score: SearchScore,
    candidate_states: Vec<(SearchScore, ActionCombo, usize)>,
    best_intermediate_solution: Option<Solution>,
}

struct WorkerData<'main, 'alloc> {
    settings: &'main SolverSettings,
    finish_solver: &'main FinishSolver,
    quality_ub_solver_shard: QualityUbSolverShard<'main, 'alloc>,
    step_lb_solver_shard: StepLbSolverShard<'main, 'alloc>,
    search_queue: &'main SearchQueue,
    min_accepted_score: SearchScore,
    candidate_states: Vec<(SearchScore, ActionCombo, usize)>,
    best_intermediate_solution: Option<Solution>,
}

impl<'main, 'alloc> WorkerData<'main, 'alloc> {
    fn finalize(self) -> WorkerResult<'alloc> {
        WorkerResult {
            quality_ub_states: self.quality_ub_solver_shard.solved_states(),
            step_lb_states: self.step_lb_solver_shard.solved_states(),
            min_accepted_score: self.min_accepted_score,
            candidate_states: self.candidate_states,
            best_intermediate_solution: self.best_intermediate_solution,
        }
    }

    fn update_min_score(&mut self, score: SearchScore) {
        self.min_accepted_score = std::cmp::max(self.min_accepted_score, score);
    }

    fn add_candidate_state(
        &mut self,
        state: SimulationState,
        score: SearchScore,
        action: ActionCombo,
        parent_id: usize,
    ) {
        if state.progress >= self.settings.max_progress() {
            if self
                .best_intermediate_solution
                .as_ref()
                .is_none_or(|solution| solution.score < (score, state.quality))
            {
                let mut actions = self.search_queue.get_actions_from_node_idx(parent_id);
                actions.push(action);
                self.best_intermediate_solution = Some(Solution {
                    score: (score, state.quality),
                    solver_actions: actions.into_vec(),
                });
            }
        } else if score >= self.min_accepted_score {
            self.candidate_states.push((score, action, parent_id));
        }
    }

    fn process_state(
        &mut self,
        state: SimulationState,
        score: SearchScore,
        backtrack_id: usize,
    ) -> Result<(), SolverException> {
        for action in FULL_SEARCH_ACTIONS {
            if let Ok(state) = use_action_combo(self.settings, state, action) {
                if !state.is_final(&self.settings.simulator_settings) {
                    if !self.finish_solver.can_finish(&state)? {
                        continue;
                    }

                    self.update_min_score(SearchScore {
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
                            self.quality_ub_solver_shard.quality_upper_bound(state)?,
                        )
                    };

                    if !self.settings.allow_non_max_quality_solutions
                        && quality_upper_bound < self.settings.max_quality()
                    {
                        continue;
                    }

                    let step_lb_hint = score
                        .steps_lower_bound
                        .saturating_sub(score.current_steps + action.steps());
                    let steps_lower_bound = match quality_upper_bound >= self.settings.max_quality()
                    {
                        true => self
                            .step_lb_solver_shard
                            .step_lower_bound(state, step_lb_hint)?
                            .saturating_add(score.current_steps + action.steps()),
                        false => score.current_steps + action.steps(),
                    };

                    let child_score = SearchScore {
                        quality_upper_bound,
                        steps_lower_bound,
                        duration_lower_bound: score.current_duration + action.duration() + 3,
                        current_steps: score.current_steps + action.steps(),
                        current_duration: score.current_duration + action.duration(),
                    };
                    self.add_candidate_state(state, child_score, action, backtrack_id);
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
                    self.update_min_score(solution_score);
                    self.add_candidate_state(state, solution_score, action, backtrack_id);
                }
            }
        }
        Ok(())
    }
}
