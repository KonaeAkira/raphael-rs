use bump_scope::BumpPool;
use raphael_sim::*;
use rayon::prelude::*;

use super::search_queue::{SearchQueueStats, SearchScore};
use crate::actions::{
    ActionCombo, FULL_SEARCH_ACTIONS, LIVE_FIRST_ACTIONS, SearchAction, enabled_search_actions,
    use_action_combo, use_action_combo_with_condition,
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

use rustc_hash::FxHashMap;
use self_cell::self_cell;
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

struct ReusableSolvers<'alloc> {
    quality_ub_solver: QualityUbSolver<'alloc>,
    step_lb_solver: StepLbSolver<'alloc>,
    quality_precomputed: bool,
    step_precomputed: bool,
}

self_cell!(
    struct ReusableSolverCell {
        owner: BumpPool,

        #[not_covariant]
        dependent: ReusableSolvers,
    }
);

pub struct MacroSolver<'a> {
    settings: SolverSettings,
    solution_callback: Box<SolutionCallback<'a>>,
    progress_callback: Box<ProgressCallback<'a>>,
    finish_solver: FinishSolver,
    finish_precomputed: bool,
    reusable_solvers: ReusableSolverCell,
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
        let reusable_interrupt_signal = interrupt_signal.clone();
        let reusable_solvers =
            ReusableSolverCell::new(BumpPool::default(), |allocator| ReusableSolvers {
                quality_ub_solver: QualityUbSolver::new(
                    settings,
                    reusable_interrupt_signal.clone(),
                    allocator,
                ),
                step_lb_solver: StepLbSolver::new(
                    settings,
                    reusable_interrupt_signal.clone(),
                    allocator,
                ),
                quality_precomputed: false,
                step_precomputed: false,
            });
        Self {
            settings,
            solution_callback,
            progress_callback,
            finish_solver: FinishSolver::new(settings),
            finish_precomputed: false,
            reusable_solvers,
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
        let _total_time = ScopedTimer::new("Total Time");

        if !self.finish_precomputed {
            let timer = ScopedTimer::new("Finish Solver");
            self.finish_solver.precompute()?;
            self.finish_precomputed = true;
            drop(timer);
        }
        if !self.finish_solver.can_finish(&initial_state)? {
            self.last_solve_runtime_stats.finish_solver_stats = self.finish_solver.runtime_stats();
            return Err(SolverException::NoSolution);
        }

        let settings = self.settings;
        let search_actions = enabled_search_actions(&settings, FULL_SEARCH_ACTIONS);
        let solution_callback = &self.solution_callback;
        let progress_callback = &self.progress_callback;
        let finish_solver = &self.finish_solver;
        let interrupt_signal = &self.interrupt_signal;
        let last_stats = &mut self.last_solve_runtime_stats;
        let actions = self.reusable_solvers.with_dependent_mut(|_, solvers| {
            if !solvers.quality_precomputed {
                let timer = ScopedTimer::new("Quality UB Solver");
                solvers.quality_ub_solver.precompute()?;
                solvers.quality_precomputed = true;
                drop(timer);
            }

            let mut shard = solvers.quality_ub_solver.create_shard();
            let initial_state_quality_ub = shard.quality_upper_bound(initial_state)?;
            solvers
                .quality_ub_solver
                .extend_solved_states(shard.solved_states());
            if initial_state_quality_ub >= settings.max_quality() && !solvers.step_precomputed {
                let _timer = ScopedTimer::new("Step LB Solver");
                solvers.step_lb_solver.precompute()?;
                solvers.step_precomputed = true;
            }

            let timer = ScopedTimer::new("Search");
            let solution = Self::do_solve_impl(
                settings,
                solution_callback,
                progress_callback,
                finish_solver,
                interrupt_signal,
                last_stats,
                &mut solvers.quality_ub_solver,
                &mut solvers.step_lb_solver,
                vec![(SearchScore::MAX, initial_state, Vec::new())],
                &search_actions,
            )?;
            drop(timer);
            Ok::<_, SolverException>(solution.actions())
        })?;

        log::debug!("{:?}", self.runtime_stats());
        Ok(actions)
    }

    /// Solves from a live synthesis state, applying `condition` to the first
    /// synthesis step and honoring forced normal-recipe transitions. In particular,
    /// Excellent is followed by Poor. Random, not-yet-observed rolls are Normal.
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

        let conditioned_roots = self.conditioned_roots(initial_state, condition)?;
        self.solve_from_roots(conditioned_roots)
    }

    fn conditioned_roots(
        &self,
        initial_state: SimulationState,
        initial_condition: Condition,
    ) -> Result<Vec<(SimulationState, Vec<ActionCombo>, u8, u8)>, SolverException> {
        let mut pending = vec![(initial_state, initial_condition, Vec::new(), 0_u8, 0_u8)];
        let mut roots: FxHashMap<SimulationState, (Vec<ActionCombo>, u8, u8)> =
            FxHashMap::default();

        while let Some((state, condition, prefix, steps, duration)) = pending.pop() {
            if self.interrupt_signal.is_set() {
                return Err(SolverException::Interrupted);
            }
            if condition == Condition::Normal || state.is_final(&self.settings.simulator_settings) {
                let candidate = (prefix, steps, duration);
                let entry = roots.entry(state).or_insert_with(|| candidate.clone());
                if (candidate.1, candidate.2) < (entry.1, entry.2) {
                    *entry = candidate;
                }
                continue;
            }

            for action_combo in LIVE_FIRST_ACTIONS {
                let Ok(child_state) =
                    use_action_combo_with_condition(&self.settings, state, action_combo, condition)
                else {
                    continue;
                };
                let mut child_prefix = prefix.clone();
                child_prefix.push(action_combo);
                let next_condition = action_combo.actions().iter().fold(
                    condition,
                    |condition, action| match action {
                        Action::HeartAndSoul | Action::QuickInnovation => condition,
                        _ => condition.next_after_step(),
                    },
                );
                pending.push((
                    child_state,
                    next_condition,
                    child_prefix,
                    steps.saturating_add(action_combo.steps()),
                    duration.saturating_add(action_combo.duration()),
                ));
            }
        }

        log::debug!("generated {} unique conditioned search roots", roots.len());
        Ok(roots
            .into_iter()
            .map(|(state, (prefix, steps, duration))| (state, prefix, steps, duration))
            .collect())
    }

    fn solve_from_roots(
        &mut self,
        roots: Vec<(SimulationState, Vec<ActionCombo>, u8, u8)>,
    ) -> Result<Vec<Action>, SolverException> {
        self.last_solve_runtime_stats = MacroSolverStats::default();
        let _total_time = ScopedTimer::new("Total Time");
        if !self.finish_precomputed {
            let timer = ScopedTimer::new("Finish Solver");
            self.finish_solver.precompute()?;
            self.finish_precomputed = true;
            drop(timer);
        }

        let settings = self.settings;
        let search_actions = enabled_search_actions(&settings, FULL_SEARCH_ACTIONS);
        let solution_callback = &self.solution_callback;
        let progress_callback = &self.progress_callback;
        let finish_solver = &self.finish_solver;
        let interrupt_signal = &self.interrupt_signal;
        let last_stats = &mut self.last_solve_runtime_stats;
        let actions = self.reusable_solvers.with_dependent_mut(|_, solvers| {
            if !solvers.quality_precomputed {
                let timer = ScopedTimer::new("Quality UB Solver");
                solvers.quality_ub_solver.precompute()?;
                solvers.quality_precomputed = true;
                drop(timer);
            }

            let mut prepared_roots = Vec::new();
            let mut needs_step_lb = false;
            for (state, prefix, current_steps, current_duration) in roots {
                if state.is_final(&settings.simulator_settings) {
                    if state.progress >= settings.max_progress() {
                        prepared_roots.push((
                            SearchScore {
                                quality_upper_bound: state.quality.min(settings.max_quality()),
                                steps_lower_bound: current_steps,
                                duration_lower_bound: current_duration,
                                current_steps,
                                current_duration,
                            },
                            state,
                            prefix,
                        ));
                    }
                    continue;
                }
                if !finish_solver.can_finish(&state)? {
                    continue;
                }
                let mut shard = solvers.quality_ub_solver.create_shard();
                let quality_upper_bound = shard.quality_upper_bound(state)?;
                solvers
                    .quality_ub_solver
                    .extend_solved_states(shard.solved_states());
                if !settings.allow_non_max_quality_solutions
                    && quality_upper_bound < settings.max_quality()
                {
                    continue;
                }
                needs_step_lb |= quality_upper_bound >= settings.max_quality();
                prepared_roots.push((
                    SearchScore {
                        quality_upper_bound,
                        steps_lower_bound: current_steps,
                        duration_lower_bound: current_duration.saturating_add(3),
                        current_steps,
                        current_duration,
                    },
                    state,
                    prefix,
                ));
            }
            if prepared_roots.is_empty() {
                return Err(SolverException::NoSolution);
            }
            if needs_step_lb {
                if !solvers.step_precomputed {
                    let _timer = ScopedTimer::new("Step LB Solver");
                    solvers.step_lb_solver.precompute()?;
                    solvers.step_precomputed = true;
                }
                for (score, state, _) in &mut prepared_roots {
                    if score.quality_upper_bound >= settings.max_quality()
                        && !state.is_final(&settings.simulator_settings)
                    {
                        score.steps_lower_bound = solvers
                            .step_lb_solver
                            .step_lower_bound(*state, 0)?
                            .saturating_add(score.current_steps);
                    }
                }
            }

            let timer = ScopedTimer::new("Search");
            let solution = Self::do_solve_impl(
                settings,
                solution_callback,
                progress_callback,
                finish_solver,
                interrupt_signal,
                last_stats,
                &mut solvers.quality_ub_solver,
                &mut solvers.step_lb_solver,
                prepared_roots,
                &search_actions,
            )?;
            drop(timer);
            Ok::<_, SolverException>(solution.actions())
        })?;
        log::debug!("{:?}", self.runtime_stats());
        Ok(actions)
    }

    #[allow(clippy::too_many_arguments)]
    fn do_solve_impl<'alloc>(
        settings: SolverSettings,
        solution_callback: &SolutionCallback<'_>,
        progress_callback: &ProgressCallback<'_>,
        finish_solver: &FinishSolver,
        interrupt_signal: &AtomicFlag,
        last_solve_runtime_stats: &mut MacroSolverStats,
        quality_ub_solver: &mut QualityUbSolver<'alloc>,
        step_lb_solver: &mut StepLbSolver<'alloc>,
        roots: Vec<(SearchScore, SimulationState, Vec<ActionCombo>)>,
        search_actions: &[SearchAction],
    ) -> Result<Solution, SolverException> {
        let mut solution = roots
            .iter()
            .filter(|(_, state, _)| state.progress >= settings.max_progress())
            .map(|(score, state, prefix)| Solution {
                score: (*score, state.quality),
                solver_actions: prefix.clone(),
            })
            .max_by_key(|solution| solution.score);
        let mut min_accepted_score = solution
            .as_ref()
            .map_or(SearchScore::MIN, |solution| solution.score.0);
        if let Some(solution) = &solution {
            solution_callback(&solution.actions());
        }
        let mut search_queue = SearchQueue::new(settings, roots);

        while let Some(Batch {
            score,
            nodes: batch,
        }) = search_queue.pop_batch()
            && score >= min_accepted_score
        {
            if interrupt_signal.is_set() {
                return Err(SolverException::Interrupted);
            }

            let create_worker_data = || WorkerData {
                settings: &settings,
                finish_solver,
                quality_ub_solver_shard: quality_ub_solver.create_shard(),
                step_lb_solver_shard: step_lb_solver.create_shard(),
                search_queue: &search_queue,
                search_actions,
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
                    solution_callback(&solution.as_ref().unwrap().actions());
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

            progress_callback(search_queue.runtime_stats().processed_nodes);
        }

        *last_solve_runtime_stats = MacroSolverStats {
            search_queue_stats: search_queue.runtime_stats(),
            finish_solver_stats: finish_solver.runtime_stats(),
            quality_ub_stats: quality_ub_solver.runtime_stats(),
            step_lb_stats: step_lb_solver.runtime_stats(),
        };

        if let Some(solution) = &solution
            && solution.score.0.quality_upper_bound < settings.max_quality()
            && !settings.allow_non_max_quality_solutions
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
    search_actions: &'main [SearchAction],
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
        for &SearchAction {
            combo: action,
            steps: action_steps,
            duration: action_duration,
        } in self.search_actions
        {
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

                    let current_steps = score.current_steps + action_steps;
                    let current_duration = score.current_duration + action_duration;
                    let step_lb_hint = score.steps_lower_bound.saturating_sub(current_steps);
                    let steps_lower_bound = match quality_upper_bound >= self.settings.max_quality()
                    {
                        true => self
                            .step_lb_solver_shard
                            .step_lower_bound(state, step_lb_hint)?
                            .saturating_add(current_steps),
                        false => current_steps,
                    };

                    let child_score = SearchScore {
                        quality_upper_bound,
                        steps_lower_bound,
                        duration_lower_bound: current_duration + 3,
                        current_steps,
                        current_duration,
                    };
                    self.add_candidate_state(state, child_score, action, backtrack_id);
                } else if state.progress >= self.settings.max_progress() {
                    let solution_score = SearchScore {
                        quality_upper_bound: std::cmp::min(
                            state.quality,
                            self.settings.max_quality(),
                        ),
                        steps_lower_bound: score.current_steps + action_steps,
                        duration_lower_bound: score.current_duration + action_duration,
                        current_steps: score.current_steps + action_steps,
                        current_duration: score.current_duration + action_duration,
                    };
                    self.update_min_score(solution_score);
                    self.add_candidate_state(state, solution_score, action, backtrack_id);
                }
            }
        }
        Ok(())
    }
}
