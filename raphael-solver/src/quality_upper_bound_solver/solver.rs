use std::{
    collections::{VecDeque, hash_map::Entry},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    SolverException, SolverSettings,
    actions::FULL_SEARCH_ACTIONS,
    macros::internal_error,
    utils::{self, ParetoFrontBuilder, ParetoValue, ScopedTimer},
};

use bump_scope::{BumpPool, BumpPoolGuard};
use raphael_sim::*;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

use super::state::ReducedState;

type ParetoFront = nunny::Slice<ParetoValue>;
type SolvedStates<'alloc> = FxHashMap<ReducedState, &'alloc ParetoFront>;

#[derive(Default, Debug, Clone, Copy)]
pub struct QualityUbSolverStats {
    pub states_on_main: usize,
    pub states_on_shards: usize,
    pub values: usize,
}

#[derive(Clone)]
struct QualityUbSolverContext<'alloc> {
    allocator: &'alloc BumpPool,
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    iq_quality_lut: [u16; 11],
    durability_cost: u16,
    largest_progress_increase: u16,
}

pub struct QualityUbSolver<'alloc> {
    context: QualityUbSolverContext<'alloc>,
    solved_states: SolvedStates<'alloc>,
    num_states_solved_on_shards: usize,
}

pub struct QualityUbSolverShard<'main, 'alloc> {
    context: &'main QualityUbSolverContext<'alloc>,
    shared_states: &'main SolvedStates<'alloc>,
    local_states: SolvedStates<'alloc>,
}

impl<'alloc> QualityUbSolver<'alloc> {
    pub fn new(
        mut settings: SolverSettings,
        interrupt_signal: utils::AtomicFlag,
        allocator: &'alloc BumpPool,
    ) -> Self {
        let durability_cost = durability_cost(&settings.simulator_settings);
        settings.simulator_settings.max_cp = {
            let initial_state = SimulationState::new(&settings.simulator_settings);
            ReducedState::from_state(initial_state, &settings, durability_cost).cp
        };
        Self {
            context: QualityUbSolverContext {
                allocator,
                settings,
                interrupt_signal,
                iq_quality_lut: utils::compute_iq_quality_lut(&settings),
                durability_cost,
                largest_progress_increase: utils::maximum_muscle_memory_utilization(
                    &settings.simulator_settings,
                ),
            },
            solved_states: FxHashMap::default(),
            num_states_solved_on_shards: 0,
        }
    }

    pub fn extend_solved_states(&mut self, new_solved_states: SolvedStates<'alloc>) {
        let len_before = self.solved_states.len();
        self.solved_states.extend(new_solved_states);
        let len_after = self.solved_states.len();
        self.num_states_solved_on_shards += len_after - len_before;
    }

    pub fn create_shard<'main>(&'main self) -> QualityUbSolverShard<'main, 'alloc> {
        QualityUbSolverShard {
            context: &self.context,
            shared_states: &self.solved_states,
            local_states: SolvedStates::default(),
        }
    }

    pub fn precompute(&mut self) -> Result<(), SolverException> {
        let batches =
            generate_precompute_states(&self.context.settings, self.context.durability_cost);
        for batch in batches {
            let solved_states = batch
                .into_par_iter()
                .map_init(
                    || (ParetoFrontBuilder::new(), self.context.allocator.get()),
                    |(pf_builder, allocator), state| -> Result<_, SolverException> {
                        let pareto_front =
                            self.solve_precompute_state(pf_builder, state, allocator)?;
                        Ok((state, pareto_front))
                    },
                )
                .collect::<Result<Vec<_>, SolverException>>()?;
            self.solved_states.extend(solved_states);
        }
        Ok(())
    }

    fn solve_precompute_state(
        &self,
        pf_builder: &mut ParetoFrontBuilder,
        state: ReducedState,
        allocator: &BumpPoolGuard<'alloc>,
    ) -> Result<&'alloc ParetoFront, SolverException> {
        let cutoff = ParetoValue::new(
            self.context.settings.max_progress(),
            self.context.settings.max_quality().saturating_sub(
                self.context.iq_quality_lut[usize::from(state.effects.inner_quiet())],
            ),
        );

        // Check for a lesser state that has reached the cutoff value, in which case we
        // can use the solution of the lesser state.
        let lesser_state = ReducedState {
            cp: state.cp - 2,
            ..state
        };
        if let Some(pareto_front) = self.solved_states.get(&lesser_state) {
            let lesser_state_is_maximal = pareto_front.first().progress >= cutoff.progress
                && pareto_front.first().quality >= cutoff.quality;
            if lesser_state_is_maximal {
                return Ok(pareto_front);
            }
        }

        pf_builder.initialize_with_cutoff(cutoff);
        for action in FULL_SEARCH_ACTIONS {
            if let Some((new_state, progress, quality)) =
                state.use_action(action, &self.context.settings, self.context.durability_cost)
            {
                let action_value = ParetoValue::new(progress, quality);
                if !new_state.is_final(self.context.durability_cost) {
                    if let Some(pareto_front) = self.solved_states.get(&new_state).copied() {
                        pf_builder.push_slice(
                            pareto_front
                                .iter()
                                .map(|value| value.saturating_add(action_value)),
                        );
                    } else {
                        return Err(internal_error!(
                            "Required precompute state does not exist.",
                            self.context.settings,
                            action,
                            state,
                            new_state
                        ));
                    }
                } else if progress != 0 {
                    pf_builder.push(action_value);
                }
                if pf_builder.is_maximal(cutoff) {
                    break;
                }
            }
        }
        let pareto_front = allocator
            .alloc_slice_copy(pf_builder.result_as_slice())
            .into_ref();
        pareto_front.try_into().map_err(|_| {
            internal_error!(
                "Empty precompute Pareto front.",
                self.context.settings,
                state
            )
        })
    }

    pub fn runtime_stats(&self) -> QualityUbSolverStats {
        QualityUbSolverStats {
            states_on_main: self.solved_states.len() - self.num_states_solved_on_shards,
            states_on_shards: self.num_states_solved_on_shards,
            values: self.solved_states.values().map(|value| value.len()).sum(),
        }
    }
}

impl<'main, 'alloc> QualityUbSolverShard<'main, 'alloc> {
    pub fn solved_states(self) -> SolvedStates<'alloc> {
        self.local_states
    }

    pub fn quality_upper_bound(
        &mut self,
        mut state: SimulationState,
    ) -> Result<u16, SolverException> {
        let mut required_progress = self.context.settings.max_progress() - state.progress;
        if state.effects.muscle_memory() != 0 {
            // Assume MuscleMemory can be used to its max potential and remove the effect to reduce the number of states that need to be solved.
            required_progress =
                required_progress.saturating_sub(self.context.largest_progress_increase);
            state.effects.set_muscle_memory(0);
        }
        let reduced_state =
            ReducedState::from_state(state, &self.context.settings, self.context.durability_cost);
        let pareto_front =
            if let Some(pareto_front) = self.shared_states.get(&reduced_state).copied() {
                pareto_front
            } else if let Some(pareto_front) = self.local_states.get(&reduced_state).copied() {
                pareto_front
            } else {
                let allocator = self.context.allocator.get();
                self.solve_state(reduced_state, &allocator)?;
                if let Some(pareto_front) = self.local_states.get(&reduced_state).copied() {
                    pareto_front
                } else {
                    return Err(internal_error!(
                        "State not found in memoization table after solve.",
                        self.context.settings,
                        reduced_state
                    ));
                }
            };
        let i = pareto_front.partition_point(|value| value.progress < required_progress);
        let quality = pareto_front
            .get(i)
            .map_or(0, |value| state.quality.saturating_add(value.quality));
        Ok(std::cmp::min(self.context.settings.max_quality(), quality))
    }

    fn solve_state(
        &mut self,
        state: ReducedState,
        allocator: &BumpPoolGuard<'alloc>,
    ) -> Result<(), SolverException> {
        if self.context.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }

        let cutoff = ParetoValue::new(
            self.context.settings.max_progress(),
            self.context.settings.max_quality().saturating_sub(
                self.context.iq_quality_lut[usize::from(state.effects.inner_quiet())],
            ),
        );

        let mut pareto_front_builder = ParetoFrontBuilder::new();
        pareto_front_builder.initialize_with_cutoff(cutoff);

        for action in FULL_SEARCH_ACTIONS {
            if let Some((child_state, progress, quality)) =
                state.use_action(action, &self.context.settings, self.context.durability_cost)
            {
                let action_value = ParetoValue::new(progress, quality);
                if !child_state.is_final(self.context.durability_cost) {
                    let child_pareto_front = if let Some(child_pareto_front) =
                        self.shared_states.get(&child_state).copied()
                    {
                        child_pareto_front
                    } else if let Some(child_pareto_front) =
                        self.local_states.get(&child_state).copied()
                    {
                        child_pareto_front
                    } else {
                        self.solve_state(child_state, allocator)?;
                        self.local_states
                            .get(&child_state)
                            .copied()
                            .ok_or_else(|| {
                                internal_error!(
                                    "State not found in memoization table after solving.",
                                    self.context.settings,
                                    child_state
                                )
                            })?
                    };
                    pareto_front_builder.push_slice(
                        child_pareto_front
                            .iter()
                            .map(|value| value.saturating_add(action_value)),
                    );
                    if pareto_front_builder.is_maximal(cutoff) {
                        break;
                    }
                } else if action_value.progress != 0 {
                    pareto_front_builder.push(action_value);
                }
            }
        }
        let pareto_front = allocator
            .alloc_slice_copy(pareto_front_builder.result_as_slice())
            .into_ref();
        let pareto_front = pareto_front.try_into().map_err(|_| {
            internal_error!(
                "Solver produced empty Pareto front.",
                self.context.settings,
                state
            )
        })?;
        self.local_states.insert(state, pareto_front);
        Ok(())
    }
}

/// Calculates the CP cost to "magically" restore 5 durability
fn durability_cost(settings: &Settings) -> u16 {
    let mut cost = 100;
    if settings.is_action_allowed::<MasterMend>() {
        let cost_per_five = MasterMend::CP_COST / std::cmp::min(6, settings.max_durability / 5 - 1);
        cost = std::cmp::min(cost, cost_per_five);
    }
    if settings.is_action_allowed::<Manipulation>() {
        let cost_per_five = Manipulation::CP_COST / 8;
        cost = std::cmp::min(cost, cost_per_five);
    }
    if settings.is_action_allowed::<ImmaculateMend>() {
        let cost_per_five = ImmaculateMend::CP_COST / (settings.max_durability / 5 - 1);
        cost = std::cmp::min(cost, cost_per_five);
    }
    cost
}

fn generate_precompute_states(
    settings: &SolverSettings,
    durability_cost: u16,
) -> Vec<Vec<ReducedState>> {
    let mut queue = VecDeque::default();

    let seed_state = ReducedState::from_state(
        SimulationState::new(&settings.simulator_settings),
        settings,
        durability_cost,
    );
    queue.push_back(seed_state);

    let timer = ScopedTimer::new("Discovery");
    // Discover all states and keep track of their in degrees.
    let mut in_degree = FxHashMap::<ReducedState, AtomicUsize>::default();
    while let Some(state) = queue.pop_front() {
        for action in FULL_SEARCH_ACTIONS {
            let Some((next_state, _action_progress, _action_quality)) =
                state.use_action(action, settings, durability_cost)
            else {
                continue;
            };
            if next_state.is_final(durability_cost) {
                continue;
            }
            match in_degree.entry(next_state) {
                Entry::Occupied(entry) => {
                    entry.into_mut().fetch_add(1, Ordering::Relaxed);
                }
                Entry::Vacant(entry) => {
                    entry.insert(AtomicUsize::new(1));
                    queue.push_back(next_state);
                }
            }
        }
    }
    drop(timer);

    let timer = ScopedTimer::new("Sorting");
    // Topological sorting of the states into batches that can be processed in parallel later.
    let mut batches = Vec::new();
    let mut next_batch = boxcar::vec![seed_state];
    while !next_batch.is_empty() {
        let current_batch = std::mem::take(&mut next_batch)
            .into_iter()
            .collect::<Vec<_>>();
        current_batch.par_iter().for_each(|state| {
            for action in FULL_SEARCH_ACTIONS {
                let Some((next_state, _action_progress, _action_quality)) =
                    state.use_action(action, settings, durability_cost)
                else {
                    continue;
                };
                if next_state.is_final(durability_cost) {
                    continue;
                }
                let in_degree = in_degree.get(&next_state).unwrap();
                if in_degree.fetch_sub(1, Ordering::Relaxed) == 1 {
                    next_batch.push(next_state);
                }
            }
        });
        batches.push(current_batch);
    }
    batches.reverse();
    drop(timer);

    batches
}
