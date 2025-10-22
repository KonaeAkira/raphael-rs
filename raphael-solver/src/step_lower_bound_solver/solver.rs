use std::{
    num::{NonZero, NonZeroU8},
    ops::Deref,
};

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
use rayon::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};

use super::state::ReducedState;

type NonEmptyParetoFront = nunny::Slice<ParetoValue>;
type SolvedStates = FxHashMap<ReducedState, Box<NonEmptyParetoFront>>;

#[derive(Debug, Clone, Copy)]
pub struct StepLbSolverStats {
    pub states_on_main: usize,
    pub states_on_shards: usize,
    pub values: usize,
}

#[derive(Debug, Clone)]
struct StepLbSolverContext {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    iq_quality_lut: [u32; 11],
    largest_progress_increase: u32,
}

pub struct StepLbSolver {
    context: StepLbSolverContext,
    solved_states: SolvedStates,
    num_states_solved_on_shards: usize,
}

pub struct StepLbSolverShard<'a> {
    context: &'a StepLbSolverContext,
    shared_states: &'a SolvedStates,
    local_states: SolvedStates,
    pf_builder: ParetoFrontBuilder,
}

impl StepLbSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        let iq_quality_lut = compute_iq_quality_lut(&settings);
        settings.simulator_settings.adversarial = false;
        ReducedState::optimize_action_mask(&mut settings.simulator_settings);
        Self {
            context: StepLbSolverContext {
                settings,
                interrupt_signal,
                iq_quality_lut,
                largest_progress_increase: largest_single_action_progress_increase(&settings),
            },
            solved_states: SolvedStates::default(),
            num_states_solved_on_shards: 0,
        }
    }

    pub fn create_shard(&self) -> StepLbSolverShard<'_> {
        StepLbSolverShard {
            context: &self.context,
            shared_states: &self.solved_states,
            local_states: SolvedStates::default(),
            pf_builder: ParetoFrontBuilder::new(),
        }
    }

    pub fn extend_solved_states(&mut self, new_solved_states: SolvedStates) {
        let len_before = self.solved_states.len();
        self.solved_states.extend(new_solved_states);
        let len_after = self.solved_states.len();
        self.num_states_solved_on_shards += len_after - len_before;
    }

    pub fn precompute(&mut self) -> Result<(), SolverException> {
        let initial_state = SimulationState::new(&self.context.settings.simulator_settings);
        self.step_lower_bound(initial_state, 0)?;
        Ok(())
    }

    pub fn step_lower_bound(
        &mut self,
        state: SimulationState,
        hint: u8,
    ) -> Result<u8, SolverException> {
        if self.context.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        if !state.effects.quality_actions_allowed()
            && state.quality < self.context.settings.max_quality()
        {
            return Ok(u8::MAX);
        }
        let mut hint = NonZeroU8::try_from(std::cmp::max(hint, 1)).unwrap();
        while self
            .quality_upper_bound(state, hint)?
            .is_none_or(|quality_ub| quality_ub < self.context.settings.max_quality())
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
        let mut required_progress = self.context.settings.max_progress() - state.progress;
        if state.effects.muscle_memory() != 0 {
            // Assume MuscleMemory can be used to its max potential and remove the effect
            // to reduce the number of states that need to be solved.
            required_progress =
                required_progress.saturating_sub(self.context.largest_progress_increase);
            state.effects.set_muscle_memory(0);
        }
        let reduced_state = ReducedState::from_state(state, step_budget);
        let pareto_front = if let Some(solution) = self.solved_states.get(&reduced_state) {
            solution
        } else {
            solve_state_parallel(reduced_state, &self.context, &mut self.solved_states)?
        };
        let idx = pareto_front.partition_point(|value| value.progress < required_progress);
        let quality_ub = pareto_front.get(idx).map(|v| state.quality + v.quality);
        Ok(quality_ub)
    }

    pub fn runtime_stats(&self) -> StepLbSolverStats {
        StepLbSolverStats {
            states_on_main: self.solved_states.len() - self.num_states_solved_on_shards,
            states_on_shards: self.num_states_solved_on_shards,
            values: self.solved_states.values().map(|value| value.len()).sum(),
        }
    }
}

impl<'a> StepLbSolverShard<'a> {
    pub fn solved_states(self) -> SolvedStates {
        self.local_states
    }

    pub fn step_lower_bound(
        &mut self,
        state: SimulationState,
        hint: u8,
    ) -> Result<u8, SolverException> {
        if self.context.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        if !state.effects.quality_actions_allowed()
            && state.quality < self.context.settings.max_quality()
        {
            return Ok(u8::MAX);
        }
        let mut hint = NonZeroU8::try_from(std::cmp::max(hint, 1)).unwrap();
        while self
            .quality_upper_bound(state, hint)?
            .is_none_or(|quality_ub| quality_ub < self.context.settings.max_quality())
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
        let mut required_progress = self.context.settings.max_progress() - state.progress;
        if state.effects.muscle_memory() != 0 {
            // Assume MuscleMemory can be used to its max potential and remove the effect
            // to reduce the number of states that need to be solved.
            required_progress =
                required_progress.saturating_sub(self.context.largest_progress_increase);
            state.effects.set_muscle_memory(0);
        }
        let reduced_state = ReducedState::from_state(state, step_budget);
        let pareto_front = if let Some(solution) = self.shared_states.get(&reduced_state) {
            solution
        } else if let Some(solution) = self.local_states.get(&reduced_state) {
            solution
        } else {
            solve_state_sequential(
                reduced_state,
                self.context,
                self.shared_states,
                &mut self.local_states,
                &mut self.pf_builder,
            )?
        };
        let idx = pareto_front.partition_point(|value| value.progress < required_progress);
        let quality_ub = pareto_front.get(idx).map(|v| state.quality + v.quality);
        Ok(quality_ub)
    }
}

fn discover_unsolved_states(
    seed_state: ReducedState,
    settings: &SolverSettings,
    has_solution: impl Fn(ReducedState) -> bool,
) -> Vec<ReducedState> {
    let mut unsolved_states = vec![seed_state];
    let mut unique_check = FxHashSet::default();
    let mut idx = 0;
    while idx < unsolved_states.len() {
        let parent = unsolved_states[idx];
        let full_parent = parent.to_state();
        for action in FULL_SEARCH_ACTIONS {
            if let Ok(step_budget) =
                NonZero::try_from(parent.steps_budget.get().saturating_sub(action.steps()))
                && let Ok(full_child) = use_action_combo(settings, full_parent, action)
                && !full_child.is_final(&settings.simulator_settings)
            {
                let child = ReducedState::from_state(full_child, step_budget);
                if !unique_check.contains(&child) && !has_solution(child) {
                    unsolved_states.push(child);
                    unique_check.insert(child);
                }
            }
        }
        idx += 1;
    }
    unsolved_states
}

fn construct_solution<'a>(
    state: ReducedState,
    context: &StepLbSolverContext,
    pf_builder: &mut ParetoFrontBuilder,
    get_solution: impl Fn(ReducedState) -> Option<&'a Box<NonEmptyParetoFront>>,
) -> Result<Box<nunny::Slice<ParetoValue>>, SolverException> {
    let min_quality = context.iq_quality_lut[usize::from(state.effects.inner_quiet())];
    let cutoff = ParetoValue::new(
        context.settings.max_progress(),
        context.settings.max_quality().saturating_sub(min_quality),
    );
    pf_builder.initialize_with_cutoff(cutoff);
    for action in FULL_SEARCH_ACTIONS {
        if state.steps_budget.get() < action.steps() {
            continue;
        }
        let new_step_budget = state.steps_budget.get() - action.steps();
        if let Ok(child_state) = use_action_combo(&context.settings, state.to_state(), action) {
            let progress = child_state.progress;
            let quality = child_state.quality;
            if let Ok(new_step_budget) = NonZeroU8::try_from(new_step_budget)
                && !child_state.is_final(&context.settings.simulator_settings)
            {
                let child_state = ReducedState::from_state(child_state, new_step_budget);
                if let Some(pareto_front) = get_solution(child_state) {
                    pf_builder.push_slice(pareto_front.iter().map(|value| {
                        ParetoValue::new(value.progress + progress, value.quality + quality)
                    }));
                } else {
                    return Err(internal_error!(
                        "Required child state does not exist.",
                        context.settings,
                        action,
                        state,
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
            context.settings,
            state
        )
    })
}

fn solve_state_sequential<'a>(
    seed_state: ReducedState,
    context: &StepLbSolverContext,
    shared_states: &'a SolvedStates,
    local_states: &'a mut SolvedStates,
    pf_builder: &mut ParetoFrontBuilder,
) -> Result<&'a nunny::Slice<ParetoValue>, SolverException> {
    let mut unsolved_states = {
        let has_solution =
            |state| shared_states.contains_key(&state) || local_states.contains_key(&state);
        discover_unsolved_states(seed_state, &context.settings, has_solution)
    };
    unsolved_states.sort_unstable_by_key(|state| state.steps_budget);
    for state in unsolved_states {
        let solution = {
            let get_solution = |state| {
                shared_states
                    .get(&state)
                    .or_else(|| local_states.get(&state))
            };
            construct_solution(state, context, pf_builder, get_solution)?
        };
        local_states.insert(state, solution);
    }
    local_states
        .get(&seed_state)
        .map(Box::deref)
        .ok_or_else(|| {
            internal_error!(
                "State not found in memoization after solving",
                context.settings,
                seed_state
            )
        })
}

fn solve_state_parallel<'a>(
    seed_state: ReducedState,
    context: &StepLbSolverContext,
    solved_states: &'a mut SolvedStates,
) -> Result<&'a nunny::Slice<ParetoValue>, SolverException> {
    let mut unsolved_states = {
        let has_solution = |state| solved_states.contains_key(&state);
        discover_unsolved_states(seed_state, &context.settings, has_solution)
    };
    unsolved_states.par_sort_unstable_by_key(|state| state.steps_budget);
    let mut idx_begin = 0;
    let mut idx_end = 0;
    while idx_begin < unsolved_states.len() {
        let current_step_budget = unsolved_states[idx_begin].steps_budget;
        while idx_end < unsolved_states.len()
            && unsolved_states[idx_end].steps_budget == current_step_budget
        {
            idx_end += 1;
        }
        let current_batch = &unsolved_states[idx_begin..idx_end];
        let current_batch_solutions = {
            let get_solution = |state| solved_states.get(&state);
            current_batch
                .par_iter()
                .map_init(
                    ParetoFrontBuilder::new,
                    |pf_builder, state| -> Result<_, SolverException> {
                        let solution =
                            construct_solution(*state, context, pf_builder, get_solution)?;
                        Ok((*state, solution))
                    },
                )
                .collect::<Result<Vec<_>, SolverException>>()?
        };
        solved_states.extend(current_batch_solutions);
        idx_begin = idx_end;
    }
    solved_states
        .get(&seed_state)
        .map(Box::deref)
        .ok_or_else(|| {
            internal_error!(
                "State not found in memoization after solving",
                context.settings,
                seed_state
            )
        })
}
