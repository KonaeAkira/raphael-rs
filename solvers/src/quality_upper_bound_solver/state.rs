use crate::branch_pruning::is_progress_only_state;

use super::solver::SolverSettings;
use simulator::*;

#[bitfield_struct::bitfield(u32)]
#[derive(PartialEq, Eq, Hash)]
pub struct ReducedStateData {
    pub cp: i16,
    pub unreliable_quality: u8,
    #[bits(2, default=Combo::None)]
    pub combo: Combo,
    #[bits(1)]
    pub progress_only: bool,
    #[bits(5)]
    _padding: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub data: ReducedStateData,
    pub effects: Effects,
}

impl ReducedState {
    pub fn from_simulation_state(
        mut state: SimulationState,
        simulator_settings: &Settings,
        solver_settings: &SolverSettings,
    ) -> Self {
        state.cp += state.effects.manipulation() as i16
            * (Manipulation::base_cp_cost(&state, &simulator_settings) / 8);
        if state.effects.trained_perfection() != SingleUse::Unavailable
            && simulator_settings.is_action_allowed::<TrainedPerfection>()
        {
            state.cp += solver_settings.durability_cost * 4;
        }
        if state.effects.heart_and_soul() == SingleUse::Available {
            state.effects.set_heart_and_soul(SingleUse::Active);
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
            data: ReducedStateData::new()
                .with_cp(cp)
                .with_unreliable_quality(unreliable_quality)
                .with_combo(state.combo)
                .with_progress_only(progress_only),
            effects,
        }
    }

    pub fn drop_combo(self) -> Self {
        Self {
            data: self.data.with_combo(Combo::None),
            effects: self.effects,
        }
    }

    fn to_simulation_state(&self, settings: &Settings) -> SimulationState {
        SimulationState {
            durability: settings.max_durability,
            cp: self.data.cp(),
            progress: 0,
            quality: 0,
            unreliable_quality: self.data.unreliable_quality() as u16 * settings.base_quality * 2,
            effects: self.effects,
            combo: self.data.combo(),
        }
    }

    pub fn use_action(
        &self,
        action: Action,
        simulator_settings: &Settings,
        solver_settings: &SolverSettings,
    ) -> Result<(Self, u16, u16), &'static str> {
        if matches!(
            action,
            Action::MasterMend | Action::ImmaculateMend | Action::Manipulation
        ) {
            panic!("Action not supported.")
        }
        let progress_only = self.data.progress_only();
        let state = self.to_simulation_state(simulator_settings);
        match state.use_action(action, Condition::Normal, simulator_settings) {
            Ok(state) => {
                let mut solver_state =
                    Self::from_simulation_state_inner(&state, simulator_settings, solver_settings);
                if progress_only {
                    solver_state.data.set_progress_only(true);
                }
                Ok((solver_state, state.progress, state.quality))
            }
            Err(err) => Err(err),
        }
    }
}
