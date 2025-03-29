use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS},
    utils::{AtomicFlag, ParetoFrontBuilder, ParetoFrontId, ParetoValue},
};
use raphael_sim::*;

use rustc_hash::FxHashMap as HashMap;

use super::state::ReducedState;

pub struct QualityUpperBoundSolver {
    settings: SolverSettings,
    solved_states: HashMap<ReducedState, ParetoFrontId>,
    pareto_front_builder: ParetoFrontBuilder<u16, u16>,
    interrupt_signal: AtomicFlag,
    // pre-computed branch pruning values
    waste_not_1_min_cp: i16,
    waste_not_2_min_cp: i16,
    durability_cost: i16,
}

impl QualityUpperBoundSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: AtomicFlag) -> Self {
        log::trace!(
            "ReducedState (QualityUpperBoundSolver) - size: {}, align: {}",
            std::mem::size_of::<ReducedState>(),
            std::mem::align_of::<ReducedState>()
        );

        settings.simulator_settings.max_cp = i16::MAX;

        let initial_state = SimulationState::new(&settings.simulator_settings);

        let mut durability_cost = 100;
        if settings
            .simulator_settings
            .is_action_allowed::<MasterMend>()
        {
            let master_mend_cost =
                MasterMend::base_cp_cost(&initial_state, &settings.simulator_settings);
            durability_cost = std::cmp::min(durability_cost, master_mend_cost / 6);
        }
        if settings
            .simulator_settings
            .is_action_allowed::<Manipulation>()
        {
            let manipulation_cost =
                Manipulation::base_cp_cost(&initial_state, &settings.simulator_settings);
            durability_cost = std::cmp::min(durability_cost, manipulation_cost / 8);
        }
        if settings
            .simulator_settings
            .is_action_allowed::<ImmaculateMend>()
        {
            let immaculate_mend_cost =
                ImmaculateMend::base_cp_cost(&initial_state, &settings.simulator_settings);
            let max_restored = settings.simulator_settings.max_durability as i16 / 5 - 1;
            durability_cost = std::cmp::min(durability_cost, immaculate_mend_cost / max_restored);
        }

        Self {
            settings,
            solved_states: HashMap::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.simulator_settings.max_progress,
                settings.simulator_settings.max_quality,
            ),
            interrupt_signal,
            durability_cost,
            waste_not_1_min_cp: waste_not_min_cp(56, 4, durability_cost),
            waste_not_2_min_cp: waste_not_min_cp(98, 8, durability_cost),
        }
    }

    /// Returns an upper-bound on the maximum Quality achievable from this state while also maxing out Progress.
    /// There is no guarantee on the tightness of the upper-bound.
    pub fn quality_upper_bound(&mut self, state: SimulationState) -> Result<u16, SolverException> {
        if state.combo == Combo::SynthesisBegin {
            return Ok(self.settings.simulator_settings.max_quality);
        }
        if state.combo != Combo::None {
            return Err(SolverException::InternalError(format!(
                "\"{:?}\" combo in quality upper bound solver",
                state.combo
            )));
        }
        let reduced_state =
            ReducedState::from_simulation_state(state, &self.settings, self.durability_cost);
        let pareto_front = match self.solved_states.get(&reduced_state) {
            Some(id) => self.pareto_front_builder.retrieve(*id),
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

    fn solve_state(&mut self, state: ReducedState) -> Result<(), SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        self.pareto_front_builder.push_empty();
        let search_actions = match state.progress_only {
            true => PROGRESS_ONLY_SEARCH_ACTIONS,
            false => FULL_SEARCH_ACTIONS,
        };
        for action in search_actions {
            if !self.should_use_action(state, *action) {
                continue;
            }
            self.build_child_front(state, *action)?;
            if self.pareto_front_builder.is_max() {
                // stop early if both Progress and Quality are maxed out
                // this optimization would work even better with better action ordering
                // (i.e. if better actions are visited first)
                break;
            }
        }
        let id = self.pareto_front_builder.save().unwrap();
        self.solved_states.insert(state, id);
        Ok(())
    }

    fn build_child_front(
        &mut self,
        state: ReducedState,
        action: ActionCombo,
    ) -> Result<(), SolverException> {
        if let Ok((new_state, action_progress, action_quality)) =
            state.use_action(action, &self.settings, self.durability_cost)
        {
            if new_state.cp >= self.durability_cost {
                match self.solved_states.get(&new_state) {
                    Some(id) => self.pareto_front_builder.push_id(*id),
                    None => self.solve_state(new_state)?,
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
            } else if new_state.cp >= -self.durability_cost && action_progress != 0 {
                // "durability" must not go lower than -5
                // last action must be a progress increase
                self.pareto_front_builder
                    .push_slice(&[ParetoValue::new(action_progress, action_quality)]);
                self.pareto_front_builder.merge();
            }
        }

        Ok(())
    }

    fn should_use_action(&self, state: ReducedState, action: ActionCombo) -> bool {
        match action {
            ActionCombo::Single(Action::WasteNot) => state.cp >= self.waste_not_1_min_cp,
            ActionCombo::Single(Action::WasteNot2) => state.cp >= self.waste_not_2_min_cp,
            _ => true,
        }
    }
}

/// Calculates the minimum CP a state must have so that using WasteNot is not worse than just restoring durability via CP
fn waste_not_min_cp(
    waste_not_action_cp_cost: i16,
    effect_duration: i16,
    durability_cost: i16,
) -> i16 {
    const BASIC_SYNTH_CP: i16 = 0;
    const GROUNDWORK_CP: i16 = 18;
    // how many units of 5-durability does WasteNot have to save to be worth using over magically restoring durability?
    let min_durability_save = (waste_not_action_cp_cost - 1) / durability_cost + 1;
    if min_durability_save > effect_duration * 2 {
        return i16::MAX;
    }
    // how many 20-durability actions and how many 10-durability actions are needed?
    let double_dur_count = min_durability_save.saturating_sub(effect_duration);
    let single_dur_count = min_durability_save.abs_diff(effect_duration) as i16;
    // minimum CP required to execute those actions
    let double_dur_cost = double_dur_count * (GROUNDWORK_CP + durability_cost * 2);
    let single_dur_cost = single_dur_count * (BASIC_SYNTH_CP + durability_cost);
    waste_not_action_cp_cost + double_dur_cost + single_dur_cost - durability_cost
}
