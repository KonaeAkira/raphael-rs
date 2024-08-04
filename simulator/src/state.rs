use crate::{effects::SingleUse, Action, Combo, Condition, Effects, Settings};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimulationState {
    pub cp: i16,
    pub durability: i8,
    pub progress: u16,
    pub unreliable_quality: [u16; 2],
    // This value represents the minimum additional quality achievable by the simulator
    // 1 while allowing the previous un-Guarded action to be Poor
    // 0 while forcing the previous un-Guarded action to be Normal
    pub effects: Effects,
    pub combo: Combo,
}

impl SimulationState {
    pub fn new(settings: &Settings) -> Self {
        Self {
            cp: settings.max_cp,
            durability: settings.max_durability,
            progress: 0,
            unreliable_quality: [0; 2],
            effects: Effects::default().with_guard(if settings.adversarial { 2 } else { 0 }),
            combo: Combo::SynthesisBegin,
        }
    }

    pub fn from_macro(settings: &Settings, actions: &[Action]) -> Result<Self, &'static str> {
        let mut state = Self::new(settings);
        for action in actions {
            state = state.use_action(*action, Condition::Normal, settings)?;
        }
        Ok(state)
    }

    pub fn from_macro_continue_on_error(
        settings: &Settings,
        actions: &[Action],
    ) -> (Self, Vec<Result<(), &'static str>>) {
        let mut state = Self::new(settings);
        let mut errors = Vec::new();
        for action in actions {
            state = match state.use_action(*action, Condition::Normal, settings) {
                Ok(new_state) => {
                    errors.push(Ok(()));
                    new_state
                }
                Err(err) => {
                    errors.push(Err(err));
                    state
                }
            };
        }
        (state, errors)
    }

    pub fn get_quality(&self) -> u16 {
        #[cfg(test)]
        assert!(self.unreliable_quality[0] >= self.unreliable_quality[1]);
        self.unreliable_quality[1]
    }

    pub fn is_final(&self, settings: &Settings) -> bool {
        self.durability <= 0 || self.progress >= settings.max_progress
    }

    pub fn can_use_action(
        &self,
        action: Action,
        condition: Condition,
        settings: &Settings,
    ) -> Result<(), &'static str> {
        if self.is_final(settings) {
            return Err("State is final");
        }
        if !settings.allowed_actions.has(action) {
            return Err("Action not enabled");
        }
        if action.cp_cost() > self.cp {
            return Err("Not enough CP");
        }
        if !action.combo_fulfilled(self.combo) {
            return Err("Combo requirement not fulfilled");
        }
        match action {
            Action::ByregotsBlessing if self.effects.inner_quiet() == 0 => {
                Err("Need Inner Quiet to use Byregot's Blessing")
            }
            Action::PrudentSynthesis | Action::PrudentTouch if self.effects.waste_not() != 0 => {
                Err("Action cannot be used during Waste Not")
            }
            Action::IntensiveSynthesis | Action::PreciseTouch
                if self.effects.heart_and_soul() != SingleUse::Active
                    && condition != Condition::Good
                    && condition != Condition::Excellent =>
            {
                Err("Requires condition to be Good or Excellent")
            }
            Action::Groundwork if self.durability < action.durability_cost(&self.effects) => {
                Err("Not enough durability")
            }
            Action::TrainedFinesse if self.effects.inner_quiet() < 10 => {
                Err("Requires 10 Inner Quiet")
            }
            Action::TrainedPerfection
                if !matches!(self.effects.trained_perfection(), SingleUse::Available) =>
            {
                Err("Action can only be used once per synthesis")
            }
            Action::HeartAndSoul if self.effects.heart_and_soul() != SingleUse::Available => {
                Err("Action can only be used once per synthesis")
            }
            Action::QuickInnovation if self.effects.quick_innovation_used() => {
                Err("Action can only be used once per synthesis")
            }
            Action::QuickInnovation if self.effects.innovation() != 0 => {
                Err("Action cannot be used when Innovation is active")
            }
            _ => Ok(()),
        }
    }

    pub fn use_action(
        self,
        action: Action,
        condition: Condition,
        settings: &Settings,
    ) -> Result<SimulationState, &'static str> {
        self.can_use_action(action, condition, settings)?;
        let mut state = self;

        let cp_cost = action.cp_cost();
        let durability_cost = action.durability_cost(&state.effects);
        let progress_increase = action.progress_increase(settings, &state.effects);
        let quality_increase = if settings.adversarial && state.effects.guard() == 0 {
            action.quality_increase(settings, &state.effects, Condition::Poor)
        } else {
            action.quality_increase(settings, &state.effects, condition)
        };
        let quality_delta = if settings.adversarial && state.effects.guard() == 0 {
            action.quality_increase(settings, &state.effects, condition)
                - action.quality_increase(settings, &state.effects, Condition::Poor)
        } else {
            0
        };

        state.cp -= cp_cost;
        state.durability -= durability_cost;

        if action.base_durability_cost() != 0
            && state.effects.trained_perfection() == SingleUse::Active
        {
            state.effects.set_trained_perfection(SingleUse::Unavailable);
        }

        // reset muscle memory if progress increased
        if progress_increase != 0 {
            state.progress += progress_increase;
            state.effects.set_muscle_memory(0);
        }

        // reset great strides and increase inner quiet if quality increased
        if quality_increase != 0 {
            state.unreliable_quality[0] += quality_increase;
            state.unreliable_quality[1] += quality_increase;
            state.effects.set_great_strides(0);
            if settings.job_level >= 11 {
                let inner_quiet_bonus = match action {
                    Action::Reflect => 2,
                    Action::PreciseTouch => 2,
                    Action::PreparatoryTouch => 2,
                    Action::ComboRefinedTouch => 2,
                    _ => 1,
                };
                state.effects.set_inner_quiet(std::cmp::min(
                    10,
                    state.effects.inner_quiet() + inner_quiet_bonus,
                ));
            }
        }

        // calculate guard effects
        if settings.adversarial {
            if (state.effects.guard() == 0 && quality_increase == 0)
                || (state.effects.guard() != 0 && quality_increase != 0)
            {
                // commit the current value
                state.unreliable_quality = [state.get_quality(); 2];
            } else if quality_increase != 0 {
                // append new info
                let saved = state.unreliable_quality[0];
                state.unreliable_quality[0] =
                    std::cmp::min(state.unreliable_quality[1], state.unreliable_quality[0])
                        + quality_delta;
                state.unreliable_quality[1] =
                    std::cmp::min(saved, state.unreliable_quality[1] + quality_delta);
            }
        }

        if state.is_final(settings) {
            return Ok(state);
        }

        state.combo = action.to_combo();

        // skip processing effects for actions that do not increase turn count
        if !matches!(action, Action::HeartAndSoul | Action::QuickInnovation) {
            if action == Action::Manipulation {
                state.effects.set_manipulation(0);
            }
            if state.effects.manipulation() > 0 {
                state.durability = std::cmp::min(state.durability + 5, settings.max_durability);
            }
            state.effects.tick_down();
        }

        if quality_increase != 0 {
            state.effects.set_guard(1);
        }

        // trigger special action effects
        match action {
            Action::MuscleMemory => state.effects.set_muscle_memory(5),
            Action::GreatStrides => state.effects.set_great_strides(3),
            Action::Veneration => state.effects.set_veneration(4),
            Action::Innovation => state.effects.set_innovation(4),
            Action::WasteNot => state.effects.set_waste_not(4),
            Action::WasteNot2 => state.effects.set_waste_not(8),
            Action::Manipulation => state.effects.set_manipulation(8),
            Action::MasterMend => {
                state.durability = std::cmp::min(settings.max_durability, state.durability + 30)
            }
            Action::ByregotsBlessing => state.effects.set_inner_quiet(0),
            Action::ImmaculateMend => state.durability = settings.max_durability,
            Action::TrainedPerfection => state.effects.set_trained_perfection(SingleUse::Active),
            Action::HeartAndSoul => state.effects.set_heart_and_soul(SingleUse::Active),
            Action::QuickInnovation => {
                state.effects.set_innovation(1);
                state.effects.set_quick_innovation_used(true);
            }
            Action::IntensiveSynthesis | Action::PreciseTouch
                if condition != Condition::Good && condition != Condition::Excellent =>
            {
                state.effects.set_heart_and_soul(SingleUse::Unavailable)
            }
            _ => (),
        }

        Ok(state)
    }
}
