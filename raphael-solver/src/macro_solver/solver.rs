use bump_scope::BumpPool;
use raphael_sim::*;
use rayon::prelude::*;

use super::search_queue::{SearchQueueStats, SearchScore};
use crate::actions::{
    ActionCombo, FULL_SEARCH_ACTIONS, LIVE_FIRST_ACTIONS, use_action_combo,
    use_action_combo_with_condition,
};
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
        self.solve_from_state(SimulationState::new(&self.settings.simulator_settings))
    }

    /// Solves the remainder of a synthesis from an arbitrary live state.
    ///
    /// The supplied state is treated as the root of the returned action list. Future
    /// crafting conditions are simulated according to the solver settings, exactly as
    /// they are for a fresh synthesis.
    pub fn solve_from_state(
        &mut self,
        initial_state: SimulationState,
    ) -> Result<Vec<Action>, SolverException> {
        log::debug!(
            "rayon::current_num_threads() = {}",
            rayon::current_num_threads()
        );

        if initial_state.is_final(&self.settings.simulator_settings)
            || initial_state.cp > self.settings.max_cp()
            || initial_state.durability > self.settings.max_durability()
        {
            return Err(SolverException::NoSolution);
        }

        self.last_solve_runtime_stats = MacroSolverStats::default();
        let allocator = BumpPool::default();
        let mut quality_ub_solver =
            QualityUbSolver::new(self.settings, self.interrupt_signal.clone(), &allocator);
        let mut step_lb_solver =
            StepLbSolver::new(self.settings, self.interrupt_signal.clone(), &allocator);

        let _total_time = ScopedTimer::new("Total Time");

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

    /// Solves from a live synthesis state, applying `condition` to the first
    /// synthesis step and assuming Normal for later, not-yet-observed steps.
    ///
    /// Calling this method again after every in-game action produces an online
    /// policy: Good/Excellent/Poor can change the selected next action while the
    /// remainder is still optimized by the regular macro solver.
    pub fn solve_from_state_with_condition(
        &mut self,
        initial_state: SimulationState,
        condition: Condition,
    ) -> Result<Vec<Action>, SolverException> {
        if condition == Condition::Normal {
            return self.solve_from_state(initial_state);
        }
        if initial_state.is_final(&self.settings.simulator_settings)
            || initial_state.cp > self.settings.max_cp()
            || initial_state.durability > self.settings.max_durability()
        {
            return Err(SolverException::NoSolution);
        }

        let mut best: Option<(u16, usize, u16, Vec<Action>, MacroSolverStats)> = None;
        for action_combo in LIVE_FIRST_ACTIONS {
            if self.interrupt_signal.is_set() {
                return Err(SolverException::Interrupted);
            }
            let Ok(child_state) = use_action_combo_with_condition(
                &self.settings,
                initial_state,
                action_combo,
                condition,
            ) else {
                continue;
            };

            let (remaining_actions, child_stats) =
                if child_state.is_final(&self.settings.simulator_settings) {
                    if child_state.progress < self.settings.max_progress() {
                        continue;
                    }
                    (Vec::new(), MacroSolverStats::default())
                } else {
                    let mut child_solver = Self::new(
                        self.settings,
                        Box::new(|_| {}),
                        Box::new(|_| {}),
                        self.interrupt_signal.clone(),
                    );
                    let condition_was_consumed = action_combo.actions().iter().any(|action| {
                        !matches!(action, Action::HeartAndSoul | Action::QuickInnovation)
                    });
                    let child_result = if condition_was_consumed {
                        child_solver.solve_from_state(child_state)
                    } else {
                        child_solver.solve_from_state_with_condition(child_state, condition)
                    };
                    match child_result {
                        Ok(actions) => (actions, child_solver.runtime_stats()),
                        Err(SolverException::NoSolution) => continue,
                        Err(error) => return Err(error),
                    }
                };

            let mut actions = action_combo.actions().to_vec();
            actions.extend(remaining_actions);

            let mut final_state = child_state;
            let mut valid = true;
            let mut condition_was_consumed = action_combo
                .actions()
                .iter()
                .any(|action| !matches!(action, Action::HeartAndSoul | Action::QuickInnovation));
            for action in &actions[action_combo.actions().len()..] {
                match final_state.use_action(
                    *action,
                    if condition_was_consumed {
                        Condition::Normal
                    } else {
                        condition
                    },
                    &self.settings.simulator_settings,
                ) {
                    Ok(state) => {
                        final_state = state;
                        condition_was_consumed |=
                            !matches!(action, Action::HeartAndSoul | Action::QuickInnovation);
                    }
                    Err(_) => {
                        valid = false;
                        break;
                    }
                }
            }
            if !valid || final_state.progress < self.settings.max_progress() {
                continue;
            }

            let quality = final_state.quality.min(self.settings.max_quality());
            let steps = actions.len();
            let duration = actions
                .iter()
                .map(|action| u16::from(action.time_cost()))
                .sum();
            let is_better =
                best.as_ref()
                    .is_none_or(|(best_quality, best_steps, best_duration, _, _)| {
                        quality > *best_quality
                            || (quality == *best_quality && steps < *best_steps)
                            || (quality == *best_quality
                                && steps == *best_steps
                                && duration < *best_duration)
                    });
            if is_better {
                best = Some((quality, steps, duration, actions, child_stats));
            }
        }

        let (_, _, _, actions, stats) = best.ok_or(SolverException::NoSolution)?;
        self.last_solve_runtime_stats = stats;
        (self.solution_callback)(&actions);
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

        if let Some(solution) = &solution
            && solution.score.0.quality_upper_bound < self.settings.max_quality()
            && !self.settings.allow_non_max_quality_solutions
        {
            return Err(SolverException::NoSolution);
        }

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
