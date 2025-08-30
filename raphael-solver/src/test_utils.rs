use rand::Rng;
use raphael_sim::*;

use crate::{
    SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
};

pub const REGULAR_ACTIONS: ActionMask = ActionMask::all()
    .remove(Action::TrainedEye)
    .remove(Action::HeartAndSoul)
    .remove(Action::QuickInnovation);
pub const NO_MANIPULATION: ActionMask = REGULAR_ACTIONS.remove(Action::Manipulation);
pub const WITH_SPECIALIST_ACTIONS: ActionMask = REGULAR_ACTIONS
    .add(Action::HeartAndSoul)
    .add(Action::QuickInnovation);

pub fn generate_random_states(
    settings: SolverSettings,
    min_num_states: usize,
) -> impl Iterator<Item = SimulationState> {
    let mut rng = rand::rng();
    let mut generated_states = vec![SimulationState::new(&settings.simulator_settings)];
    while generated_states.len() < min_num_states {
        let state = generated_states[rng.random_range(0..generated_states.len())];
        for _ in 0..5 {
            let action = FULL_SEARCH_ACTIONS[rng.random_range(0..FULL_SEARCH_ACTIONS.len())];
            if let Ok(new_state) = use_action_combo(&settings, state, action)
                && !new_state.is_final(&settings.simulator_settings)
            {
                generated_states.push(new_state);
            }
        }
    }
    generated_states.into_iter()
}
