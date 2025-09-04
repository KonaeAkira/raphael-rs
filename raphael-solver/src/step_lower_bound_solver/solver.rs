use std::{collections::VecDeque, num::NonZeroU8};

use crate::{
    SolverException, SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
    macros::internal_error,
    utils::{self, largest_single_action_progress_increase},
};
use raphael_sim::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rustc_hash::FxHashSet;

use super::state::ReducedState;

type ParetoValue = utils::ParetoValue<u32, u32>;
type ParetoFrontBuilder = utils::ParetoFrontBuilder<u32, u32>;
type SolvedStates = rustc_hash::FxHashMap<ReducedState, Box<[ParetoValue]>>;

#[derive(Debug, Clone, Copy)]
pub struct StepLbSolverStats {
    pub states: usize,
    pub pareto_values: usize,
}

pub struct StepLbSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    largest_progress_increase: u32,
}

impl StepLbSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        settings.simulator_settings.adversarial = false;
        ReducedState::optimize_action_mask(&mut settings.simulator_settings);
        Self {
            settings,
            interrupt_signal,
            solved_states: SolvedStates::default(),
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
        if !state.effects.allow_quality_actions() && state.quality < self.settings.max_quality() {
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
        if state.effects.combo() != Combo::None {
            return Err(internal_error!(
                "Unexpected combo state.",
                self.settings,
                state
            ));
        }

        let mut required_progress = self.settings.max_progress() - state.progress;
        if state.effects.muscle_memory() != 0 {
            // Assume MuscleMemory can be used to its max potential and remove the effect to reduce the number of states that need to be solved.
            required_progress = required_progress.saturating_sub(self.largest_progress_increase);
            state.effects.set_muscle_memory(0);
        }

        let reduced_state = ReducedState::from_state(state, step_budget);
        let pareto_front = match self.solved_states.get(&reduced_state) {
            Some(pareto_front) => pareto_front,
            None => self.solve_state(reduced_state)?,
        };
        let index = pareto_front.partition_point(|value| value.first < required_progress);
        let quality_ub = pareto_front
            .get(index)
            .map(|value| state.quality + value.second);
        Ok(quality_ub)
    }

    fn solve_state(&mut self, seed_state: ReducedState) -> Result<&[ParetoValue], SolverException> {
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
                    if let Ok(full_child_state) =
                        use_action_combo(&self.settings, full_parent_state, action)
                        && !full_child_state.is_final(&self.settings.simulator_settings)
                    {
                        let child_state =
                            ReducedState::from_state(full_child_state, child_steps_budget);
                        if !self.solved_states.contains_key(&child_state) {
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
            let solved_states = unsolved_states
                .into_par_iter()
                .map_init(
                    || {
                        ParetoFrontBuilder::new(
                            self.settings.max_progress(),
                            self.settings.max_quality(),
                        )
                    },
                    |pf_builder, reduced_state| -> Result<_, SolverException> {
                        let pareto_front = self.do_solve_state(pf_builder, reduced_state)?;
                        Ok((reduced_state, pareto_front))
                    },
                )
                .collect::<Result<Vec<_>, SolverException>>()?;
            self.solved_states.extend(solved_states);
        }

        match self.solved_states.get(&seed_state) {
            Some(pareto_front) => Ok(pareto_front),
            None => Err(internal_error!(
                "State not found in memoization after solving",
                self.settings,
                seed_state
            )),
        }
    }

    fn do_solve_state(
        &self,
        pareto_front_builder: &mut ParetoFrontBuilder,
        state: ReducedState,
    ) -> Result<Box<[ParetoValue]>, SolverException> {
        pareto_front_builder.clear();
        pareto_front_builder.push_empty();
        for action in FULL_SEARCH_ACTIONS {
            if state.steps_budget.get() < action.steps() {
                continue;
            }
            let new_step_budget = state.steps_budget.get() - action.steps();
            if let Ok(new_state) = use_action_combo(&self.settings, state.to_state(), action) {
                let progress = new_state.progress;
                let quality = new_state.quality;
                if let Ok(new_step_budget) = NonZeroU8::try_from(new_step_budget)
                    && new_state.durability > 0
                {
                    let new_state = ReducedState::from_state(new_state, new_step_budget);
                    if let Some(pareto_front) = self.solved_states.get(&new_state) {
                        pareto_front_builder.push_slice(pareto_front);
                    } else {
                        return Err(internal_error!(
                            "Required precompute state does not exist.",
                            self.settings,
                            action,
                            state,
                            new_state
                        ));
                    }
                    pareto_front_builder
                        .peek_mut()
                        .unwrap()
                        .iter_mut()
                        .for_each(|value| {
                            value.first += progress;
                            value.second += quality;
                        });
                    pareto_front_builder.merge();
                } else if progress != 0 {
                    pareto_front_builder.push_slice(&[ParetoValue::new(progress, quality)]);
                    pareto_front_builder.merge();
                }
            }
        }
        Ok(Box::from(pareto_front_builder.peek().unwrap()))
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
