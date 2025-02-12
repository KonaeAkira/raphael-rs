use crate::{
    actions::{use_action_combo, ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS},
    branch_pruning::is_progress_only_state,
    utils::{AtomicFlag, ParetoFrontBuilder, ParetoFrontId, ParetoValue},
    SolverException,
};
use simulator::*;

use rustc_hash::FxHashMap as HashMap;

use super::state::ReducedState;

pub struct StepLowerBoundSolver {
    settings: Settings,
    backload_progress: bool,
    unsound_branch_pruning: bool,
    solved_states: HashMap<ReducedState, ParetoFrontId>,
    pareto_front_builder: ParetoFrontBuilder<u16, u16>,
    interrupt_signal: AtomicFlag,
}

impl StepLowerBoundSolver {
    pub fn new(
        mut settings: Settings,
        backload_progress: bool,
        unsound_branch_pruning: bool,
        interrupt_signal: AtomicFlag,
    ) -> Self {
        log::trace!(
            "ReducedState (StepLowerBoundSolver) - size: {}, align: {}",
            std::mem::size_of::<ReducedState>(),
            std::mem::align_of::<ReducedState>()
        );
        ReducedState::optimize_action_mask(&mut settings);
        Self {
            settings,
            backload_progress,
            unsound_branch_pruning,
            solved_states: HashMap::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.max_progress,
                settings.max_quality,
            ),
            interrupt_signal,
        }
    }

    pub fn step_lower_bound_with_hint(
        &mut self,
        state: SimulationState,
        mut hint: u8,
    ) -> Result<u8, SolverException> {
        if self.backload_progress
            && state.progress != 0
            && state.quality < self.settings.max_quality
        {
            return Ok(u8::MAX);
        }
        hint = std::cmp::max(1, hint);
        while hint != u8::MAX && self.quality_upper_bound(state, hint)? < self.settings.max_quality
        {
            hint += 1;
        }
        Ok(hint)
    }

    fn quality_upper_bound(
        &mut self,
        state: SimulationState,
        step_budget: u8,
    ) -> Result<u16, SolverException> {
        if state.combo == Combo::SynthesisBegin {
            return Ok(self.settings.max_quality);
        }
        if state.combo != Combo::None {
            return Err(SolverException::InternalError(
                "Combo state in internal solver",
            ));
        }

        let current_quality = state.quality;
        let missing_progress = self.settings.max_progress.saturating_sub(state.progress);

        let progress_only =
            is_progress_only_state(&state, self.backload_progress, self.unsound_branch_pruning);
        let reduced_state = ReducedState::from_state(state, step_budget, progress_only);

        let pareto_front = match self.solved_states.get(&reduced_state) {
            Some(id) => self.pareto_front_builder.retrieve(*id),
            None => {
                self.pareto_front_builder.clear();
                self.solve_state(reduced_state)?;
                self.pareto_front_builder.peek().unwrap()
            }
        };

        match pareto_front.last() {
            Some(element) => {
                if element.first < missing_progress {
                    return Ok(0);
                }
            }
            None => return Ok(0),
        }

        let index = match pareto_front.binary_search_by_key(&missing_progress, |value| value.first)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        Ok(std::cmp::min(
            self.settings.max_quality.saturating_mul(2),
            pareto_front[index].second.saturating_add(current_quality),
        ))
    }

    fn solve_state(&mut self, reduced_state: ReducedState) -> Result<(), SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }

        self.pareto_front_builder.push_empty();
        let search_actions = match reduced_state.progress_only {
            false => FULL_SEARCH_ACTIONS,
            true => PROGRESS_ONLY_SEARCH_ACTIONS,
        };
        for action in search_actions.iter() {
            self.build_child_front(reduced_state, *action)?;
            if self.pareto_front_builder.is_max() {
                // stop early if both Progress and Quality are maxed out
                // this optimization would work even better with better action ordering
                // (i.e. if better actions are visited first)
                break;
            }
        }
        let id = self.pareto_front_builder.save().unwrap();
        self.solved_states.insert(reduced_state, id);

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
            if reduced_state.steps_budget < action.steps() {
                return Ok(());
            }
            let action_progress = new_full_state.progress;
            let action_quality = new_full_state.quality;
            let progress_only = reduced_state.progress_only
                || is_progress_only_state(
                    &new_full_state,
                    self.backload_progress,
                    self.unsound_branch_pruning,
                );
            let new_reduced_state = ReducedState::from_state(
                new_full_state,
                reduced_state.steps_budget - action.steps(),
                progress_only,
            );
            if new_reduced_state.steps_budget != 0 && new_reduced_state.durability > 0 {
                match self.solved_states.get(&new_reduced_state) {
                    Some(id) => self.pareto_front_builder.push_from_id(*id),
                    None => self.solve_state(new_reduced_state)?,
                }
                self.pareto_front_builder.map(move |value| {
                    value.first = value.first.saturating_add(action_progress);
                    value.second = value.second.saturating_add(action_quality);
                });
                self.pareto_front_builder.merge();
            } else if action_progress != 0 {
                // last action must be a progress increase
                self.pareto_front_builder
                    .push_from_slice(&[ParetoValue::new(action_progress, action_quality)]);
                self.pareto_front_builder.merge();
            }
        }

        Ok(())
    }
}
