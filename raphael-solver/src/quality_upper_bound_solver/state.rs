use crate::{
    SolverSettings,
    actions::{ActionCombo, is_progress_only_state, use_action_combo},
};

use raphael_sim::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: i16,
    pub unreliable_quality: u8,
    pub progress_only: bool,
    pub effects: Effects,
}

impl ReducedState {
    pub fn from_simulation_state(
        mut state: SimulationState,
        settings: &SolverSettings,
        durability_cost: i16,
    ) -> Self {
        state.cp += state.effects.manipulation() as i16
            * (Manipulation::base_cp_cost(&state, &settings.simulator_settings) / 8);
        if state.effects.trained_perfection_active() || state.effects.trained_perfection_available()
        {
            state.cp += durability_cost * 4;
        }
        state.cp += state.durability as i16 / 5 * durability_cost;
        state.durability = settings.simulator_settings.max_durability;
        Self::from_simulation_state_inner(&state, settings, durability_cost)
    }

    fn from_simulation_state_inner(
        state: &SimulationState,
        settings: &SolverSettings,
        durability_cost: i16,
    ) -> Self {
        let progress_only = is_progress_only_state(settings, state);
        let used_durability = (settings.simulator_settings.max_durability - state.durability) / 5;
        let cp = state.cp - used_durability as i16 * durability_cost;
        let unreliable_quality = if progress_only {
            0
        } else {
            state
                .unreliable_quality
                .div_ceil(2 * settings.simulator_settings.base_quality) as u8
        };
        let effects = {
            let great_strides_active = state.effects.great_strides() != 0;
            state
                .effects
                .with_great_strides(if great_strides_active { 3 } else { 0 })
                .with_trained_perfection_available(false)
                .with_trained_perfection_active(false)
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

    pub fn has_no_quality_attributes(&self) -> bool {
        self.unreliable_quality == 0
            && self.effects.inner_quiet() == 0
            && self.effects.innovation() == 0
            && self.effects.great_strides() == 0
            && self.effects.guard() == 0
            && self.effects.quick_innovation_available() == false
    }

    fn strip_quality_attributes(&mut self) {
        self.unreliable_quality = 0;
        self.effects.set_inner_quiet(0);
        self.effects.set_innovation(0);
        self.effects.set_great_strides(0);
        self.effects.set_guard(0);
        self.effects.set_quick_innovation_available(false);
    }

    pub fn use_action(
        &self,
        action: ActionCombo,
        settings: &SolverSettings,
        durability_cost: i16,
    ) -> Result<(Self, u16, u16), &'static str> {
        match action {
            ActionCombo::Single(
                Action::MasterMend | Action::ImmaculateMend | Action::Manipulation,
            ) => Err("Action not supported"),
            _ => {
                let progress_only = self.progress_only;
                let state = self.to_simulation_state(&settings.simulator_settings);
                match use_action_combo(settings, state, action) {
                    Ok(state) => {
                        let mut solver_state =
                            Self::from_simulation_state_inner(&state, settings, durability_cost);
                        if progress_only || solver_state.progress_only {
                            solver_state.progress_only = true;
                            solver_state.strip_quality_attributes();
                        }
                        Ok((solver_state, state.progress, state.quality))
                    }
                    Err(err) => Err(err),
                }
            }
        }
    }
}
