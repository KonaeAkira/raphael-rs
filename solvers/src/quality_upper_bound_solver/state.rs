use crate::{
    actions::{use_action_combo, ActionCombo},
    branch_pruning::is_progress_only_state,
};

use super::solver::SolverSettings;
use simulator::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: i16,
    pub unreliable_quality: u8,
    pub progress_only: bool,
    pub effects: Effects,
}

impl ReducedState {
    pub fn from_simulation_state(
        mut state: SimulationState,
        simulator_settings: &Settings,
        solver_settings: &SolverSettings,
    ) -> Self {
        state.cp += state.effects.manipulation() as i16
            * (Manipulation::base_cp_cost(&state, simulator_settings) / 8);
        if state.effects.trained_perfection() != SingleUse::Unavailable
            && simulator_settings.is_action_allowed::<TrainedPerfection>()
        {
            state.cp += solver_settings.durability_cost * 4;
        }
        state.cp += state.durability as i16 / 5 * solver_settings.durability_cost;
        state.durability = simulator_settings.max_durability;
        Self::from_simulation_state_inner(&state, simulator_settings, solver_settings)
    }

    fn from_simulation_state_inner(
        state: &SimulationState,
        simulator_settings: &Settings,
        solver_settings: &SolverSettings,
    ) -> Self {
        let progress_only = is_progress_only_state(
            state,
            solver_settings.backload_progress,
            solver_settings.unsound_branch_pruning,
        );
        let used_durability = (simulator_settings.max_durability - state.durability) / 5;
        let cp = state.cp - used_durability as i16 * solver_settings.durability_cost;
        let unreliable_quality = if progress_only {
            0
        } else {
            state
                .unreliable_quality
                .div_ceil(2 * simulator_settings.base_quality) as u8
        };
        let effects = if progress_only {
            state
                .effects
                .with_inner_quiet(0)
                .with_innovation(0)
                .with_great_strides(0)
                .with_guard(0)
                .with_quick_innovation_used(true)
                .with_trained_perfection(SingleUse::Unavailable)
                .with_manipulation(0)
        } else {
            let great_strides_active = state.effects.great_strides() != 0;
            state
                .effects
                .with_great_strides(if great_strides_active { 3 } else { 0 })
                .with_trained_perfection(SingleUse::Unavailable)
                .with_manipulation(0)
        };
        Self {
            cp,
            unreliable_quality,
            progress_only,
            effects,
        }
    }

    fn to_simulation_state(self, settings: &Settings) -> SimulationState {
        SimulationState {
            durability: settings.max_durability,
            cp: self.cp,
            progress: 0,
            quality: 0,
            unreliable_quality: self.unreliable_quality as u16 * settings.base_quality * 2,
            effects: self.effects,
            combo: Combo::None,
        }
    }

    pub fn use_action(
        &self,
        action: ActionCombo,
        simulator_settings: &Settings,
        solver_settings: &SolverSettings,
    ) -> Result<(Self, u16, u16), &'static str> {
        match action {
            ActionCombo::Single(
                Action::MasterMend | Action::ImmaculateMend | Action::Manipulation,
            ) => Err("Action not supported"),
            _ => {
                let progress_only = self.progress_only;
                let state = self.to_simulation_state(simulator_settings);
                match use_action_combo(simulator_settings, state, action) {
                    Ok(state) => {
                        let mut solver_state = Self::from_simulation_state_inner(
                            &state,
                            simulator_settings,
                            solver_settings,
                        );
                        if progress_only {
                            solver_state.progress_only = true;
                        }
                        Ok((solver_state, state.progress, state.quality))
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }
}
