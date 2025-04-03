use std::num::NonZeroU8;

use crate::{
    SolverException, SolverSettings,
    actions::{
        ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS, is_progress_only_state,
        use_action_combo,
    },
    utils,
};
use raphael_sim::*;

use super::state::ReducedState;

type ParetoValue = utils::ParetoValue<u16, u16>;
type ParetoFrontBuilder = utils::ParetoFrontBuilder<u16, u16>;
type SolvedStates = papaya::HashMap<
    ReducedState,
    Box<[ParetoValue]>,
    std::hash::BuildHasherDefault<rustc_hash::FxHasher>,
>;

pub struct StepLowerBoundSolver {
    settings: SolverSettings,
    solved_states: SolvedStates,
    interrupt_signal: utils::AtomicFlag,
}

impl StepLowerBoundSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        log::trace!(
            "ReducedState (StepLowerBoundSolver) - size: {}, align: {}",
            std::mem::size_of::<ReducedState>(),
            std::mem::align_of::<ReducedState>()
        );
        ReducedState::optimize_action_mask(&mut settings.simulator_settings);
        Self {
            settings,
            solved_states: SolvedStates::default(),
            interrupt_signal,
        }
    }

    pub fn step_lower_bound(
        &self,
        state: SimulationState,
        hint: u8,
    ) -> Result<u8, SolverException> {
        if self.settings.backload_progress
            && state.progress != 0
            && state.quality < self.settings.simulator_settings.max_quality
        {
            return Ok(u8::MAX);
        }
        let mut hint = NonZeroU8::try_from(std::cmp::max(hint, 1)).unwrap();
        while hint.get() != u8::MAX
            && self.quality_upper_bound(state, hint)? < self.settings.simulator_settings.max_quality
        {
            hint = hint.saturating_add(1);
        }
        Ok(hint.get())
    }

    fn quality_upper_bound(
        &self,
        state: SimulationState,
        step_budget: NonZeroU8,
    ) -> Result<u16, SolverException> {
        if state.combo != Combo::None {
            return Err(SolverException::InternalError(format!(
                "\"{:?}\" combo in step lower bound solver",
                state.combo
            )));
        }

        let progress_only = is_progress_only_state(&self.settings, &state);
        let reduced_state = ReducedState::from_state(state, step_budget, progress_only);
        let required_progress = self.settings.simulator_settings.max_progress - state.progress;

        if let Some(pareto_front) = self.solved_states.pin().get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality = pareto_front
                .get(index)
                .map_or(0, |value| state.quality.saturating_add(value.second));
            return Ok(std::cmp::min(
                self.settings.simulator_settings.max_quality,
                quality,
            ));
        }

        self.par_solve_state(reduced_state)?;

        if let Some(pareto_front) = self.solved_states.pin().get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality = pareto_front
                .get(index)
                .map_or(0, |value| state.quality.saturating_add(value.second));
            Ok(std::cmp::min(
                self.settings.simulator_settings.max_quality,
                quality,
            ))
        } else {
            unreachable!("State must be in memoization table after solver")
        }
    }

    fn par_solve_state(&self, state: ReducedState) -> Result<(), SolverException> {
        let (lhs_result, rhs_result) = rayon::join(
            || {
                let (lhs_result, rhs_result) = rayon::join(
                    || self.solve_state_begin::<59>(state),
                    || self.solve_state_begin::<67>(state),
                );
                lhs_result?;
                rhs_result?;
                Ok(())
            },
            || {
                let (lhs_result, rhs_result) = rayon::join(
                    || self.solve_state_begin::<73>(state),
                    || self.solve_state_begin::<97>(state),
                );
                lhs_result?;
                rhs_result?;
                Ok(())
            },
        );
        lhs_result?;
        rhs_result?;
        Ok(())
    }

    fn solve_state_begin<const S: usize>(
        &self,
        state: ReducedState,
    ) -> Result<(), SolverException> {
        let mut pareto_front_builder = ParetoFrontBuilder::new(
            self.settings.simulator_settings.max_progress,
            self.settings.simulator_settings.max_quality,
        );
        self.solve_state::<S>(&mut pareto_front_builder, state)
    }

    fn solve_state<const S: usize>(
        &self,
        pareto_front_builder: &mut ParetoFrontBuilder,
        reduced_state: ReducedState,
    ) -> Result<(), SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        pareto_front_builder.push_empty();

        // S must be co-prime to the action list length, otherwise we won't iterate over all actions.
        assert_eq!(gcd::euclid_usize(S, PROGRESS_ONLY_SEARCH_ACTIONS.len()), 1);
        assert_eq!(gcd::euclid_usize(S, FULL_SEARCH_ACTIONS.len()), 1);
        let search_actions = match reduced_state.progress_only {
            true => PROGRESS_ONLY_SEARCH_ACTIONS,
            false => FULL_SEARCH_ACTIONS,
        };
        for i in 0..search_actions.len() {
            let action = search_actions[(i + 1) * S % search_actions.len()];
            if action.steps() <= reduced_state.steps_budget.get() {
                self.build_child_front::<S>(pareto_front_builder, reduced_state, action)?;
                if pareto_front_builder.is_max() {
                    // stop early if both Progress and Quality are maxed out
                    // this optimization would work even better with better action ordering
                    // (i.e. if better actions are visited first)
                    break;
                }
            }
        }

        let pareto_front = Box::from(pareto_front_builder.peek().unwrap());
        self.solved_states.pin().insert(reduced_state, pareto_front);
        Ok(())
    }

    fn build_child_front<const S: usize>(
        &self,
        pareto_front_builder: &mut ParetoFrontBuilder,
        reduced_state: ReducedState,
        action: ActionCombo,
    ) -> Result<(), SolverException> {
        if let Ok(new_full_state) =
            use_action_combo(&self.settings, reduced_state.to_state(), action)
        {
            let action_progress = new_full_state.progress;
            let action_quality = new_full_state.quality;
            let progress_only = reduced_state.progress_only
                || is_progress_only_state(&self.settings, &new_full_state);
            let new_step_budget = reduced_state.steps_budget.get() - action.steps();
            match NonZeroU8::try_from(new_step_budget) {
                Ok(new_step_budget) if new_full_state.durability > 0 => {
                    // New state is not final
                    let new_reduced_state =
                        ReducedState::from_state(new_full_state, new_step_budget, progress_only);
                    if let Some(pareto_front) = self.solved_states.pin().get(&new_reduced_state) {
                        pareto_front_builder.push_slice(pareto_front);
                    } else {
                        self.solve_state::<S>(pareto_front_builder, new_reduced_state)?;
                    }
                    pareto_front_builder
                        .peek_mut()
                        .unwrap()
                        .iter_mut()
                        .for_each(|value| {
                            value.first = value.first.saturating_add(action_progress);
                            value.second = value.second.saturating_add(action_quality);
                        });
                    pareto_front_builder.merge();
                }
                _ if action_progress != 0 => {
                    // New state is final and last action increased Progress
                    pareto_front_builder
                        .push_slice(&[ParetoValue::new(action_progress, action_quality)]);
                    pareto_front_builder.merge();
                }
                _ => {
                    // New state is final but last action did not increase Progress
                    // Skip this state
                }
            }
        }
        Ok(())
    }
}
