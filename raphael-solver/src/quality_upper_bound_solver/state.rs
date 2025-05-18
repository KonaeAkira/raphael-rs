use crate::{
    SolverSettings,
    actions::{ActionCombo, use_action_combo},
};

use raphael_sim::*;

/// A high enough value to make sure that no action combo fails due to missing durability.
const MAX_DURABILITY: u16 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: u16,
    pub compressed_unreliable_quality: u8,
    pub effects: Effects,
}

impl ReducedState {
    pub fn from_state(
        mut state: SimulationState,
        settings: &SolverSettings,
        durability_cost: u16,
    ) -> Self {
        // Turn all current durability +5 extra durability into CP.
        // The +5 extra durability is to account for the possibility of a 10-durability action being used at the end while at 5 durability, resulting in -5 durability.
        // A final value of -10 durability isn't considered because the last action has to be a Progress-increasing action, and the only Progress action with more than 10 durability cost is Groundwork.
        // However, Groundwork has its efficiency halved when there is not enough durability to cover the full cost, which makes CarefulSynthesis a better option.
        let mut refunded_durability = state.durability / 5 + 1;

        // Assume Manipulation effect can be used to its full potential (each tick restores 5 durability).
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
        state.durability = MAX_DURABILITY;
        Self::from_state_inner(&state, settings, durability_cost).unwrap()
    }

    fn from_state_inner(
        state: &SimulationState,
        settings: &SolverSettings,
        durability_cost: u16,
    ) -> Option<Self> {
        let used_durability_cost = (MAX_DURABILITY - state.durability) / 5 * durability_cost;
        if used_durability_cost > state.cp {
            return None;
        }
        let compressed_unreliable_quality = state
            .unreliable_quality
            .div_ceil(2 * settings.base_quality())
            as u8;
        let effects = {
            let great_strides_active = state.effects.great_strides() != 0;
            state
                .effects
                .with_great_strides(if great_strides_active { 3 } else { 0 })
        };
        Some(Self {
            cp: state.cp - used_durability_cost,
            compressed_unreliable_quality,
            effects,
        })
    }

    fn to_simulation_state(self, settings: &SolverSettings) -> SimulationState {
        SimulationState {
            durability: MAX_DURABILITY,
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
                let state = self.to_simulation_state(settings);
                match use_action_combo(settings, state, action) {
                    Ok(state) => {
                        let solver_state =
                            Self::from_state_inner(&state, settings, durability_cost)?;
                        Some((solver_state, state.progress, state.quality))
                    }
                    Err(_) => None,
                }
            }
        }
    }
}
