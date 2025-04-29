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
    pareto_front_builder: ParetoFrontBuilder,
}

impl StepLbSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        ReducedState::optimize_action_mask(&mut settings.simulator_settings);
        Self {
            settings,
            interrupt_signal,
            solved_states: SolvedStates::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.max_progress(),
                settings.max_quality(),
            ),
        }
    }

    pub fn step_lower_bound(
        &mut self,
        state: SimulationState,
        hint: u8,
    ) -> Result<u8, SolverException> {
        if self.settings.backload_progress
            && state.progress != 0
            && state.quality < self.settings.max_quality()
        {
            return Ok(u8::MAX);
        }
        let mut hint = NonZeroU8::try_from(std::cmp::max(hint, 1)).unwrap();
        while hint.get() != u8::MAX
            && self.quality_upper_bound(state, hint)? < self.settings.max_quality()
        {
            hint = hint.saturating_add(1);
        }
        Ok(hint.get())
    }

    fn quality_upper_bound(
        &mut self,
        state: SimulationState,
        step_budget: NonZeroU8,
    ) -> Result<u32, SolverException> {
        if state.effects.combo() != Combo::None {
            return Err(SolverException::InternalError(format!(
                "\"{:?}\" combo in step lower bound solver",
                state.effects.combo()
            )));
        }

        let progress_only = is_progress_only_state(&self.settings, &state);
        let reduced_state = ReducedState::from_state(state, step_budget, progress_only);
        let required_progress = self.settings.max_progress() - state.progress;

        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality = pareto_front
                .get(index)
                .map_or(0, |value| state.quality + value.second);
            return Ok(std::cmp::min(self.settings.max_quality(), quality));
        }

        self.solve_state(reduced_state)?;

        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality = pareto_front
                .get(index)
                .map_or(0, |value| state.quality + value.second);
            Ok(std::cmp::min(self.settings.max_quality(), quality))
        } else {
            unreachable!("State must be in memoization table after solver")
        }
    }

    fn solve_state(&mut self, reduced_state: ReducedState) -> Result<(), SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        self.pareto_front_builder.push_empty();
        let search_actions = match reduced_state.progress_only {
            true => PROGRESS_ONLY_SEARCH_ACTIONS,
            false => FULL_SEARCH_ACTIONS,
        };
        for &action in search_actions {
            if action.steps() <= reduced_state.steps_budget.get() {
                self.build_child_front(reduced_state, action)?;
                if self.pareto_front_builder.is_max() {
                    // stop early if both Progress and Quality are maxed out
                    // this optimization would work even better with better action ordering
                    // (i.e. if better actions are visited first)
                    break;
                }
            }
        }
        let pareto_front = Box::from(self.pareto_front_builder.peek().unwrap());
        self.solved_states.insert(reduced_state, pareto_front);
        Ok(())
    }

    fn build_child_front(
        &mut self,
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
                    if let Some(pareto_front) = self.solved_states.get(&new_reduced_state) {
                        self.pareto_front_builder.push_slice(pareto_front);
                    } else {
                        self.solve_state(new_reduced_state)?;
                    }
                    self.pareto_front_builder
                        .peek_mut()
                        .unwrap()
                        .iter_mut()
                        .for_each(|value| {
                            value.first += action_progress;
                            value.second += action_quality;
                        });
                    self.pareto_front_builder.merge();
                }
                _ if action_progress != 0 => {
                    // New state is final and last action increased Progress
                    self.pareto_front_builder
                        .push_slice(&[ParetoValue::new(action_progress, action_quality)]);
                    self.pareto_front_builder.merge();
                }
                _ => {
                    // New state is final but last action did not increase Progress
                    // Skip this state
                }
            }
        }
        Ok(())
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
