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

use rustc_hash::FxHashMap as HashMap;

use super::state::ReducedState;

type ParetoValue = utils::ParetoValue<u16, u16>;
type ParetoFrontBuilder = utils::ParetoFrontBuilder<u16, u16>;

pub struct StepLowerBoundSolver {
    settings: SolverSettings,
    solved_states: HashMap<ReducedState, Box<[ParetoValue]>>,
    pareto_front_builder: ParetoFrontBuilder,
    interrupt_signal: utils::AtomicFlag,
    single_step_states: usize,
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
            solved_states: HashMap::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.simulator_settings.max_progress,
                settings.simulator_settings.max_quality,
            ),
            interrupt_signal,
            single_step_states: 0,
        }
    }

    pub fn step_lower_bound_with_hint(
        &mut self,
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
        &mut self,
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

        let pareto_front = match self.solved_states.get(&reduced_state) {
            Some(pareto_front) => pareto_front,
            None => {
                self.pareto_front_builder.clear();
                self.solve_state(reduced_state)?;
                self.pareto_front_builder.peek().unwrap()
            }
        };
        let required_progress = self.settings.simulator_settings.max_progress - state.progress;
        let index = pareto_front.partition_point(|value| value.first < required_progress);
        let quality_upper_bound = pareto_front.get(index).map_or(0, |value| {
            std::cmp::min(
                self.settings.simulator_settings.max_quality,
                state.quality.saturating_add(value.second),
            )
        });
        Ok(quality_upper_bound)
    }

    fn solve_state(&mut self, reduced_state: ReducedState) -> Result<(), SolverException> {
        if reduced_state.steps_budget.get() == 1 {
            self.single_step_states += 1;
        }
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        self.pareto_front_builder.push_empty();
        let search_actions = match reduced_state.progress_only {
            false => FULL_SEARCH_ACTIONS,
            true => PROGRESS_ONLY_SEARCH_ACTIONS,
        };
        for action in search_actions {
            if action.steps() <= reduced_state.steps_budget.get() {
                self.build_child_front(reduced_state, *action)?;
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
                    match self.solved_states.get(&new_reduced_state) {
                        Some(pareto_front) => self.pareto_front_builder.push_slice(pareto_front),
                        None => self.solve_state(new_reduced_state)?,
                    }
                    self.pareto_front_builder
                        .peek_mut()
                        .unwrap()
                        .iter_mut()
                        .for_each(|value| {
                            value.first = value.first.saturating_add(action_progress);
                            value.second = value.second.saturating_add(action_quality);
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
}
