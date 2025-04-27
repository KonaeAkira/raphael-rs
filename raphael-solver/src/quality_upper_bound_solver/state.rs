use crate::{
    SolverSettings,
    actions::{ActionCombo, is_progress_only_state, use_action_combo},
};

use raphael_sim::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: u16,
    pub compressed_unreliable_quality: u8,
    pub progress_only: bool,
    pub effects: Effects,
}

impl ReducedState {
    pub fn from_simulation_state(
        mut state: SimulationState,
        settings: &SolverSettings,
        durability_cost: u16,
    ) -> Self {
        let mut refunded_durability = state.durability / 5 + 1;
        // Assume Manipulation effect can be used to its full potential
        refunded_durability += u16::from(state.effects.manipulation());
        state.effects.set_manipulation(0);
        // Assume TrainedPerfection can be used to its full potential (saving 20 durability)
        if state.effects.trained_perfection_active() || state.effects.trained_perfection_available()
        {
            refunded_durability += 4;
            state.effects.set_trained_perfection_active(false);
            state.effects.set_trained_perfection_available(false);
        }
        state.cp += refunded_durability * durability_cost;
        state.durability = settings.max_durability();
        Self::from_simulation_state_inner(&state, settings, durability_cost).unwrap()
    }

    fn from_simulation_state_inner(
        state: &SimulationState,
        settings: &SolverSettings,
        durability_cost: u16,
    ) -> Option<Self> {
        let progress_only = is_progress_only_state(settings, state);
        let used_durability_cost =
            (settings.max_durability() - state.durability) / 5 * durability_cost;
        if used_durability_cost > state.cp {
            return None;
        }
        let compressed_unreliable_quality = if progress_only {
            0
        } else {
            state
                .unreliable_quality
                .div_ceil(2 * settings.base_quality()) as u8
        };
        let effects = {
            let great_strides_active = state.effects.great_strides() != 0;
            state
                .effects
                .with_great_strides(if great_strides_active { 3 } else { 0 })
        };
        Some(Self {
            cp: state.cp - used_durability_cost,
            compressed_unreliable_quality,
            progress_only,
            effects,
        })
    }

    fn to_simulation_state(self, settings: &SolverSettings) -> SimulationState {
        SimulationState {
            durability: settings.max_durability(),
            cp: self.cp,
            progress: 0,
            quality: 0,
            unreliable_quality: u32::from(self.compressed_unreliable_quality)
                * (2 * settings.base_quality()),
            effects: self.effects,
        }
    }

    pub fn is_final(&self, durability_cost: u16) -> bool {
        self.cp < 2 * durability_cost
    }

    fn strip_quality_attributes(&mut self) {
        self.compressed_unreliable_quality = 0;
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
        durability_cost: u16,
    ) -> Option<(Self, u32, u32)> {
        match action {
            ActionCombo::Single(
                Action::MasterMend | Action::ImmaculateMend | Action::Manipulation,
            ) => None,
            _ => {
                let progress_only = self.progress_only;
                let state = self.to_simulation_state(settings);
                match use_action_combo(settings, state, action) {
                    Ok(state) => {
                        let mut solver_state =
                            Self::from_simulation_state_inner(&state, settings, durability_cost)?;
                        if progress_only || solver_state.progress_only {
                            solver_state.progress_only = true;
                            solver_state.strip_quality_attributes();
                        }
                        Some((solver_state, state.progress, state.quality))
                    }
                    Err(_) => None,
                }
            }
        }
    }
}
