use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS},
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

pub struct QualityUpperBoundSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    // pre-computed branch pruning values
    waste_not_1_min_cp: i16,
    waste_not_2_min_cp: i16,
    durability_cost: i16,
}

impl QualityUpperBoundSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
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
            solved_states: SolvedStates::default(),
            interrupt_signal,
            durability_cost,
            waste_not_1_min_cp: waste_not_min_cp(56, 4, durability_cost),
            waste_not_2_min_cp: waste_not_min_cp(98, 8, durability_cost),
        }
    }

    /// Returns an upper-bound on the maximum Quality achievable from this state while also maxing out Progress.
    /// There is no guarantee on the tightness of the upper-bound.
    pub fn quality_upper_bound(&self, state: SimulationState) -> Result<u16, SolverException> {
        if state.combo != Combo::None {
            return Err(SolverException::InternalError(format!(
                "\"{:?}\" combo in quality upper bound solver",
                state.combo
            )));
        }

        let reduced_state =
            ReducedState::from_simulation_state(state, &self.settings, self.durability_cost);
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
        state: ReducedState,
    ) -> Result<(), SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        pareto_front_builder.push_empty();
        let search_actions = match state.progress_only {
            true => PROGRESS_ONLY_SEARCH_ACTIONS,
            false => FULL_SEARCH_ACTIONS,
        };
        for i in 0..search_actions.len() {
            let action = search_actions[(i + 1) * S % search_actions.len()];
            if !self.should_use_action(state, action) {
                continue;
            }
            self.build_child_front::<S>(pareto_front_builder, state, action)?;
            if pareto_front_builder.is_max() {
                // stop early if both Progress and Quality are maxed out
                // this optimization would work even better with better action ordering
                // (i.e. if better actions are visited first)
                break;
            }
        }
        let pareto_front = Box::from(pareto_front_builder.peek().unwrap());
        self.solved_states.pin().insert(state, pareto_front);
        Ok(())
    }

    #[inline(always)]
    fn build_child_front<const S: usize>(
        &self,
        pareto_front_builder: &mut ParetoFrontBuilder,
        state: ReducedState,
        action: ActionCombo,
    ) -> Result<(), SolverException> {
        if let Ok((new_state, progress, quality)) =
            state.use_action(action, &self.settings, self.durability_cost)
        {
            if new_state.cp >= self.durability_cost {
                if let Some(pareto_front) = self.solved_states.pin().get(&new_state) {
                    pareto_front_builder.push_slice(pareto_front);
                } else {
                    self.solve_state::<S>(pareto_front_builder, new_state)?;
                }
                pareto_front_builder
                    .peek_mut()
                    .unwrap()
                    .iter_mut()
                    .for_each(|value| {
                        value.first = value.first.saturating_add(progress);
                        value.second = value.second.saturating_add(quality);
                    });
                pareto_front_builder.merge();
            } else if new_state.cp >= -self.durability_cost && progress != 0 {
                // "durability" must not go lower than -5
                // last action must be a progress increase
                pareto_front_builder.push_slice(&[ParetoValue::new(progress, quality)]);
                pareto_front_builder.merge();
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
