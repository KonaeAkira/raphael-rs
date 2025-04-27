use std::num::NonZeroU8;

use raphael_sim::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub steps_budget: NonZeroU8,
    pub progress_only: bool,
    pub durability: u16,
    pub effects: Effects,
}

impl ReducedState {
    pub fn optimize_action_mask(settings: &mut Settings) {
        settings.allowed_actions = settings
            .allowed_actions
            .remove(Action::Observe)
            .remove(Action::TricksOfTheTrade)
            .remove(Action::TrainedPerfection);
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

    pub fn from_state(
        state: SimulationState,
        steps_budget: NonZeroU8,
        progress_only: bool,
    ) -> Self {
        Self {
            steps_budget,
            progress_only,
            durability: Self::optimize_durability(state.effects, state.durability, steps_budget),
            effects: Self::optimize_effects(state.effects, steps_budget, progress_only),
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

    fn optimize_durability(effects: Effects, durability: u16, step_budget: NonZeroU8) -> u16 {
        let mut usable_durability = u16::from(step_budget.get()) * 20;
        let usable_manipulation = std::cmp::min(effects.manipulation(), step_budget.get() - 1);
        usable_durability -= u16::from(usable_manipulation) * 5;
        let usable_waste_not = std::cmp::min(effects.waste_not(), step_budget.get());
        usable_durability -= u16::from(usable_waste_not) * 10;
        std::cmp::min(usable_durability, durability)
    }

    fn optimize_effects(
        mut effects: Effects,
        step_budget: NonZeroU8,
        progress_only: bool,
    ) -> Effects {
        if effects.manipulation() > step_budget.get() - 1 {
            effects.set_manipulation(step_budget.get() - 1);
        }
        if effects.waste_not() != 0 {
            // make waste not last forever
            // this gives a looser bound but decreases the number of states
            effects.set_waste_not(8);
        }
        effects.set_trained_perfection_available(false);
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

        if progress_only {
            effects
                .with_inner_quiet(0)
                .with_innovation(0)
                .with_great_strides(0)
                .with_quick_innovation_available(false)
                .with_guard(1)
        } else {
            effects.with_guard(1)
        }
    }
}
