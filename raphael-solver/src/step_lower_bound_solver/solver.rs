use std::{collections::VecDeque, num::NonZeroU8};

use crate::{
    SolverException, SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
    macros::internal_error,
    utils::{
        self, ParetoFrontBuilder, ParetoValue, compute_iq_quality_lut,
        largest_single_action_progress_increase,
    },
};

use raphael_sim::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rustc_hash::FxHashSet;

use super::state::ReducedState;

type SolvedStates = rustc_hash::FxHashMap<ReducedState, Box<nunny::Slice<ParetoValue>>>;

#[derive(Debug, Clone, Copy)]
pub struct StepLbSolverStats {
    pub states: usize,
    pub pareto_values: usize,
}

pub struct StepLbSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    iq_quality_lut: [u32; 11],
    largest_progress_increase: u32,
}

impl StepLbSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        let iq_quality_lut = compute_iq_quality_lut(&settings);
        settings.simulator_settings.adversarial = false;
        ReducedState::optimize_action_mask(&mut settings.simulator_settings);
        Self {
            settings,
            interrupt_signal,
            solved_states: SolvedStates::default(),
            iq_quality_lut,
            largest_progress_increase: largest_single_action_progress_increase(&settings),
        }
    }

    pub fn step_lower_bound(
        &mut self,
        state: SimulationState,
        hint: u8,
    ) -> Result<u8, SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        if !state.effects.quality_actions_allowed() && state.quality < self.settings.max_quality() {
            return Ok(u8::MAX);
        }
        let mut hint = NonZeroU8::try_from(std::cmp::max(hint, 1)).unwrap();
        while self
            .quality_upper_bound(state, hint)?
            .is_none_or(|quality_ub| quality_ub < self.settings.max_quality())
        {
            hint = hint.checked_add(1).unwrap();
        }
        Ok(hint.get())
    }

    fn quality_upper_bound(
        &mut self,
        mut state: SimulationState,
        step_budget: NonZeroU8,
    ) -> Result<Option<u32>, SolverException> {
        let mut required_progress = self.settings.max_progress() - state.progress;
        if state.effects.muscle_memory() != 0 {
            // Assume MuscleMemory can be used to its max potential and remove the effect to reduce the number of states that need to be solved.
            required_progress = required_progress.saturating_sub(self.largest_progress_increase);
            state.effects.set_muscle_memory(0);
        }

        let reduced_state = ReducedState::from_state(state, step_budget);
        let pareto_front = match self.solved_states.get(&reduced_state) {
            Some(pareto_front) => pareto_front,
            None => solve_state_parallel(
                reduced_state,
                &self.settings,
                &self.iq_quality_lut,
                &mut self.solved_states,
            )?,
        };
        let index = pareto_front.partition_point(|value| value.progress < required_progress);
        let quality_ub = pareto_front
            .get(index)
            .map(|value| state.quality + value.quality);
        Ok(quality_ub)
    }

    pub fn runtime_stats(&self) -> StepLbSolverStats {
        StepLbSolverStats {
            states: self.solved_states.len(),
            pareto_values: self.solved_states.values().map(|value| value.len()).sum(),
        }
    }
}

impl Drop for StepLbSolver {
    fn drop(&mut self) {
        let runtime_stats = self.runtime_stats();
        log::debug!(
            "StepLbSolver - states: {}, values: {}",
            runtime_stats.states,
            runtime_stats.pareto_values
        );
    }
}

fn solve_state_parallel<'a>(
    seed_state: ReducedState,
    settings: &SolverSettings,
    iq_quality_lut: &[u32; 11],
    solved_states: &'a mut SolvedStates,
) -> Result<&'a nunny::Slice<ParetoValue>, SolverException> {
    // Find all transitive children that still need solving and group them by step budget.
    // This is done with a simple BFS, skipping all states that have already been solved.
    let mut unvisited_states_by_steps: VecDeque<FxHashSet<ReducedState>> = VecDeque::new();
    for _ in 0..seed_state.steps_budget.get() {
        unvisited_states_by_steps.push_back(FxHashSet::default());
    }
    unvisited_states_by_steps[0].insert(seed_state);
    let mut unsolved_state_by_steps: Vec<Vec<ReducedState>> = Vec::new();
    while let Some(unvisited_states) = unvisited_states_by_steps.pop_front() {
        if unvisited_states.is_empty() {
            continue;
        }
        let currently_visited_states = unvisited_states.into_iter().collect::<Vec<_>>();
        for parent_state in &currently_visited_states {
            let full_parent_state = parent_state.to_state();
            let parent_steps_budget = parent_state.steps_budget.get();
            for action in FULL_SEARCH_ACTIONS
                .into_iter()
                .filter(|action| action.steps() < parent_steps_budget)
            {
                let child_steps_budget =
                    NonZeroU8::try_from(parent_steps_budget - action.steps()).unwrap();
                if let Ok(full_child_state) = use_action_combo(settings, full_parent_state, action)
                    && !full_child_state.is_final(&settings.simulator_settings)
                {
                    let child_state =
                        ReducedState::from_state(full_child_state, child_steps_budget);
                    if !solved_states.contains_key(&child_state) {
                        unvisited_states_by_steps[usize::from(action.steps() - 1)]
                            .insert(child_state);
                    }
                }
            }
        }
        unsolved_state_by_steps.push(currently_visited_states);
    }

    // Solve unsolved states in parallel, batched by step budget to ensure the children
    // of the current batch have already been solved in one of the previous batches.
    // This is the wavefront technique for parallelizing dynamic programming.
    for unsolved_states in unsolved_state_by_steps.into_iter().rev() {
        let new_solved_states = unsolved_states
            .into_par_iter()
            .map_init(
                ParetoFrontBuilder::new,
                |pf_builder, reduced_state| -> Result<_, SolverException> {
                    let pareto_front = merge_child_solutions(
                        reduced_state,
                        settings,
                        iq_quality_lut,
                        solved_states,
                        pf_builder,
                    )?;
                    Ok((reduced_state, pareto_front))
                },
            )
            .collect::<Result<Vec<_>, SolverException>>()?;
        solved_states.extend(new_solved_states);
    }

    match solved_states.get(&seed_state) {
        Some(pareto_front) => Ok(pareto_front),
        None => Err(internal_error!(
            "State not found in memoization after solving",
            settings,
            seed_state
        )),
    }
}

fn merge_child_solutions(
    parent_state: ReducedState,
    settings: &SolverSettings,
    iq_quality_lut: &[u32; 11],
    solved_states: &SolvedStates,
    pf_builder: &mut ParetoFrontBuilder,
) -> Result<Box<nunny::Slice<ParetoValue>>, SolverException> {
    let cutoff = ParetoValue::new(
        settings.max_progress(),
        settings
            .max_quality()
            .saturating_sub(iq_quality_lut[usize::from(parent_state.effects.inner_quiet())]),
    );
    pf_builder.initialize_with_cutoff(cutoff);
    for action in FULL_SEARCH_ACTIONS {
        if parent_state.steps_budget.get() < action.steps() {
            continue;
        }
        let new_step_budget = parent_state.steps_budget.get() - action.steps();
        if let Ok(child_state) = use_action_combo(settings, parent_state.to_state(), action) {
            let progress = child_state.progress;
            let quality = child_state.quality;
            if let Ok(new_step_budget) = NonZeroU8::try_from(new_step_budget)
                && !child_state.is_final(&settings.simulator_settings)
            {
                let child_state = ReducedState::from_state(child_state, new_step_budget);
                if let Some(pareto_front) = solved_states.get(&child_state) {
                    pf_builder.push_slice(pareto_front.iter().map(|value| {
                        ParetoValue::new(value.progress + progress, value.quality + quality)
                    }));
                } else {
                    return Err(internal_error!(
                        "Required precompute state does not exist.",
                        settings,
                        action,
                        parent_state,
                        child_state
                    ));
                }
            } else if progress != 0 {
                pf_builder.push(ParetoValue::new(progress, quality));
            }
        }
    }
    pf_builder.result().try_into().map_err(|_| {
        internal_error!(
            "Solver produced empty Pareto front.",
            settings,
            parent_state
        )
    })
}
