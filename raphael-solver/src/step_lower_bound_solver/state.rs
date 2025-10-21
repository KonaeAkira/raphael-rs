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
        // Observe is only useful in the Observe > AdvancedTouch combo to reduce the CP cost of AdvancedTouch.
        // But because there is no CP cost, it's better to just use AdvancedTouch directly.
        settings.allowed_actions = settings.allowed_actions.remove(Action::Observe);
        // TricksOfTheTrade restores CP, but the StepLbSolver has no CP cost, so the action is useless.
        settings.allowed_actions = settings.allowed_actions.remove(Action::TricksOfTheTrade);
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

    pub fn from_state(state: SimulationState, steps_budget: NonZeroU8) -> Self {
        let mut effects = state.effects;

        // Make it so that TrainedPerfection can be used an arbitrary amount of times instead of just once.
        // This decreases the number of possible states, as now there are only Active/Inactive states for TrainedPerfection instead of the usual Available/Active/Unavailable.
        // This also technically loosens the step-lb, but testing shows that rarely has any impact on the number of pruned nodes.
        effects.set_trained_perfection_available(true);
        // Same thing for QuickInnovation. Just set it to always available.
        effects.set_quick_innovation_available(true);

        // Make the effects of GreatStrides and WasteNot last forever.
        // This decreases the number of unique states as now each effect only has 2 possible states
        // instead of [0,3] for GreatStrides and [0,8] for Waste Not.
        // Turning GreatStrides into a binary state only has a small impact on the quality of the step lower bound,
        // whereas turning WasteNot into a binary state significantly reduces the quality of the step lower bound.
        // However, having WasteNot as a binary state also significantly reduces the number of unique states.
        // In the future we might need to reconsider if the WasteNot modification is worth trade-off.
        if effects.great_strides() != 0 {
            effects.set_great_strides(3);
        }
        if effects.waste_not() != 0 {
            effects.set_waste_not(8);
        }

        // If Innovation and Veneration are both active, set both effects to the same value.
        // This greatly decreases the number of unique states and in practice does not decrease the lower bound
        // tightness much.
        if effects.innovation() != 0 && effects.veneration() != 0 {
            let innovation_veneration = std::cmp::max(effects.innovation(), effects.veneration());
            effects.set_innovation(innovation_veneration);
            effects.set_veneration(innovation_veneration);
        }

        // Clamp all effects down to the steps budget to reduce the number of unique states.
        if effects.manipulation() > steps_budget.get() - 1 {
            effects.set_manipulation(steps_budget.get() - 1);
        }
        if effects.veneration() > steps_budget.get() {
            effects.set_veneration(steps_budget.get());
        }
        if effects.innovation() > steps_budget.get() {
            effects.set_innovation(steps_budget.get());
        }

        // The StepLbSolver does not implement adversarial mode.
        if effects.adversarial_guard_active() {
            effects.set_special_quality_state(SpecialQualityState::Normal);
        }

        // Sometimes a state has more durability than it is possible to use within the given steps budget,
        // in which case we should scale back the state's durability to reduce the number of unique states.
        let durability = {
            let mut max_usable_durability = u16::from(steps_budget.get()) * 20;
            max_usable_durability -= u16::from(effects.manipulation()) * 5;
            let max_usable_waste_not = std::cmp::min(effects.waste_not(), steps_budget.get());
            max_usable_durability -= u16::from(max_usable_waste_not) * 10;
            std::cmp::min(max_usable_durability, state.durability)
        };

        Self {
            steps_budget,
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
