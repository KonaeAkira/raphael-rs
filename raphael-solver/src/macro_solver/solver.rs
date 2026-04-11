use bump_scope::BumpPool;
use raphael_sim::*;
use rayon::prelude::*;

use super::search_queue::{SearchQueueStats, SearchScore};
use crate::actions::{ActionCombo, FULL_SEARCH_ACTIONS, use_action_combo};
use crate::finish_solver::FinishSolverStats;
use crate::macro_solver::search_queue::{Batch, SearchQueue};
use crate::utils::AtomicFlag;
use crate::utils::ScopedTimer;
use crate::{FinishSolver, ScoreUbSolver, SolverException, SolverSettings};

use std::sync::Mutex;
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
        let score_ub_solver = Mutex::new(ScoreUbSolver::new(
            self.settings,
            self.interrupt_signal.clone(),
            allocator.get(),
        ));

        let _total_time = ScopedTimer::new("Total Time");

        let initial_state = SimulationState::new(&self.settings.simulator_settings);

        let timer = ScopedTimer::new("Finish Solver");
        self.finish_solver.precompute()?;
        if !self.finish_solver.can_finish(&initial_state)? {
            self.last_solve_runtime_stats.finish_solver_stats = self.finish_solver.runtime_stats();
            return Err(SolverException::NoSolution);
        }
        drop(timer);

        let timer = ScopedTimer::new("Search");
        let actions = self.do_solve(&score_ub_solver, initial_state)?.actions();
        drop(timer);

        log::debug!("{:?}", self.runtime_stats());

        Ok(actions)
    }

    fn do_solve<'alloc>(
        &mut self,
        score_ub_solver: &Mutex<ScoreUbSolver<'alloc>>,
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
                score_ub_solver,
                min_accepted_score,
                candidate_states: Vec::new(),
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

            min_accepted_score = worker_results
                .iter()
                .map(|result| result.min_accepted_score)
                .max()
                .unwrap_or(min_accepted_score);
            search_queue.drop_nodes_below_score(min_accepted_score);

            for worker_data in &worker_results {
                for &(state, score, action, parent_id) in &worker_data.candidate_states {
                    if state.progress >= self.settings.max_progress() {
                        if solution
                            .as_ref()
                            .is_none_or(|solution| solution.score < (score, state.quality))
                        {
                            let mut actions = search_queue.get_actions_from_node_idx(parent_id);
                            actions.push(action);
                            solution = Some(Solution {
                                score: (score, state.quality),
                                solver_actions: actions.into_vec(),
                            });
                            (self.solution_callback)(&solution.as_ref().unwrap().actions());
                        }
                    } else if score >= min_accepted_score {
                        search_queue.push(score, action, parent_id)?;
                    }
                }
            }

            (self.progress_callback)(search_queue.runtime_stats().processed_nodes);
        }

        self.last_solve_runtime_stats = MacroSolverStats {
            search_queue_stats: search_queue.runtime_stats(),
            finish_solver_stats: self.finish_solver.runtime_stats(),
        };

        solution.ok_or(SolverException::NoSolution)
    }

    pub fn runtime_stats(&self) -> MacroSolverStats {
        self.last_solve_runtime_stats
    }
}

struct WorkerData<'main, 'alloc> {
    settings: &'main SolverSettings,
    finish_solver: &'main FinishSolver,
    score_ub_solver: &'main Mutex<ScoreUbSolver<'alloc>>,
    min_accepted_score: SearchScore,
    candidate_states: Vec<(SimulationState, SearchScore, ActionCombo, usize)>,
}

impl<'main, 'alloc> WorkerData<'main, 'alloc> {
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
        if score >= self.min_accepted_score {
            self.candidate_states
                .push((state, score, action, parent_id));
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

                    let score_ub = self
                        .score_ub_solver
                        .lock()
                        .unwrap()
                        .score_upper_bound(state, score.current_steps)?;

                    let child_score = SearchScore {
                        quality_upper_bound: score_ub.quality,
                        steps_lower_bound: score_ub.step_count,
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
