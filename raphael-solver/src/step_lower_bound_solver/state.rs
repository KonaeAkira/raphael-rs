use std::num::NonZeroU8;

use raphael_sim::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub steps_budget: NonZeroU8,
    pub durability: u16,
    pub effects: Effects,
}

impl ReducedState {
    pub fn optimize_action_mask(settings: &mut Settings) {
        settings.allowed_actions = settings
            .allowed_actions
            .remove(Action::Observe)
            .remove(Action::TricksOfTheTrade);
        // WasteNot2 is always better than WasteNot because there is no CP cost
        if settings.is_action_allowed::<WasteNot2>() {
            settings.allowed_actions = settings.allowed_actions.remove(Action::WasteNot);
        }
        // CarefulSynthesis is always better than BasicSynthesis because there is no CP cost
        if settings.is_action_allowed::<CarefulSynthesis>() {
            settings.allowed_actions = settings.allowed_actions.remove(Action::BasicSynthesis);
        }
        // AdvancedTouch is always better than StandardTouch because there is no CP cost
        if settings.is_action_allowed::<AdvancedTouch>() {
            settings.allowed_actions = settings.allowed_actions.remove(Action::StandardTouch);
        }
        // ImmaculateMend is always better than MasterMend because there is no CP cost
        if settings.is_action_allowed::<ImmaculateMend>() {
            settings.allowed_actions = settings.allowed_actions.remove(Action::MasterMend);
        }
    }

    pub fn from_state(state: SimulationState, step_budget: NonZeroU8) -> Self {
        // Optimize effects
        let mut effects = state.effects;
        // Make it so that TrainedPerfection can be used an arbitrary amount of times instead of just once.
        // This decreases the number of possible states, as now there are only Active/Inactive states for TrainedPerfection instead of the usual Available/Active/Unavailable.
        // This also technically loosens the step-lb, but testing shows that rarely has any impact on the number of pruned nodes.
        effects.set_trained_perfection_available(true);
        if effects.manipulation() > step_budget.get() - 1 {
            effects.set_manipulation(step_budget.get() - 1);
        }
        if effects.waste_not() != 0 {
            // make waste not last forever
            // this gives a looser bound but decreases the number of states
            effects.set_waste_not(8);
        }
        if effects.veneration() > step_budget.get() {
            effects.set_veneration(step_budget.get());
        }
        if effects.innovation() > step_budget.get() {
            effects.set_innovation(step_budget.get());
        }
        if effects.great_strides() != 0 {
            // make great strides last forever (until used)
            // this gives a looser bound but decreases the number of states
            effects.set_great_strides(3);
        }
        effects.set_adversarial_guard(false);
        // Optimize durability
        let durability = {
            let mut usable_durability = u16::from(step_budget.get()) * 20;
            let usable_manipulation = std::cmp::min(effects.manipulation(), step_budget.get() - 1);
            usable_durability -= u16::from(usable_manipulation) * 5;
            let usable_waste_not = std::cmp::min(effects.waste_not(), step_budget.get());
            usable_durability -= u16::from(usable_waste_not) * 10;
            std::cmp::min(usable_durability, state.durability)
        };
        Self {
            steps_budget: step_budget,
            durability,
            effects,
        }
    }

    pub fn to_state(self) -> SimulationState {
        SimulationState {
            durability: self.durability,
            cp: 1000,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: self.effects,
        }
    }
}
