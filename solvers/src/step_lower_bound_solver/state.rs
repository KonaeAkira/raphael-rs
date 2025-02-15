use simulator::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub steps_budget: u8,
    pub progress_only: bool,
    pub durability: i8,
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

    pub fn from_state(state: SimulationState, steps_budget: u8, progress_only: bool) -> Self {
        Self {
            steps_budget,
            progress_only,
            durability: state.durability,
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
            combo: Combo::None,
        }
    }

    fn optimize_effects(mut effects: Effects, step_budget: u8, progress_only: bool) -> Effects {
        if effects.manipulation() > step_budget.saturating_sub(1) {
            effects.set_manipulation(step_budget.saturating_sub(1));
        }
        if effects.waste_not() != 0 {
            // make waste not last forever
            // this gives a looser bound but decreases the number of states
            effects.set_waste_not(8);
        }
        if effects.trained_perfection() == SingleUse::Available {
            effects.set_trained_perfection(SingleUse::Unavailable);
        }
        if effects.veneration() > step_budget {
            effects.set_veneration(step_budget);
        }
        if effects.innovation() > step_budget {
            effects.set_innovation(step_budget);
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
