use simulator::{Action, ActionMask, Combo, Effects, SimulationState, SingleUse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub steps_budget: u8,
    pub durability: i8,
    pub combo: Combo,
    pub effects: Effects,
}

impl ReducedState {
    pub fn optimize_action_mask(mut action_mask: ActionMask) -> ActionMask {
        // No CP cost so Observe is useless
        action_mask = action_mask.remove(Action::Observe);
        // Non-combo version is just as good as the combo version because there is no CP cost
        action_mask = action_mask
            .remove(Action::ComboStandardTouch)
            .remove(Action::ComboAdvancedTouch);
        // ImmaculateMend is always better than MasterMend because there is no CP cost
        if action_mask.has(Action::ImmaculateMend) {
            action_mask = action_mask.remove(Action::MasterMend);
        }
        // WasteNot2 is always better than WasteNot because there is no CP cost
        if action_mask.has(Action::WasteNot2) {
            action_mask = action_mask.remove(Action::WasteNot);
        }
        // CarefulSynthesis is always better than BasicSynthesis because there is no CP cost
        if action_mask.has(Action::CarefulSynthesis) {
            action_mask = action_mask.remove(Action::BasicSynthesis);
        }
        // AdvancedTouch (non-combo) is always better than StandardTouch (non-combo) because there is no CP cost
        if action_mask.has(Action::AdvancedTouch) {
            action_mask = action_mask.remove(Action::StandardTouch);
        }
        action_mask
    }

    pub fn from_state(state: SimulationState, steps_budget: u8) -> Self {
        let innovation = std::cmp::min(steps_budget, state.effects.innovation());
        let veneration = std::cmp::min(steps_budget, state.effects.veneration());
        let great_strides = if state.effects.great_strides() != 0 {
            3
        } else {
            0
        };
        let waste_not = if state.effects.waste_not() != 0 { 8 } else { 0 };
        let manipulation = if state.effects.manipulation() != 0 {
            8
        } else {
            0
        };
        let trained_perfection = match state.effects.trained_perfection() {
            SingleUse::Unavailable => SingleUse::Available,
            SingleUse::Available => SingleUse::Available,
            SingleUse::Active => SingleUse::Active,
        };
        Self {
            steps_budget,
            durability: state.durability,
            combo: match state.combo {
                Combo::None => Combo::None,
                Combo::SynthesisBegin => Combo::SynthesisBegin,
                // Can't optimize this combo away because there is no replacement for RefinedTouch
                Combo::BasicTouch => Combo::BasicTouch,
                // AdvancedTouch replaces ComboAdvancedTouch (no CP cost)
                Combo::StandardTouch => Combo::None,
            },
            effects: state
                .effects
                .with_innovation(innovation)
                .with_veneration(veneration)
                .with_great_strides(great_strides)
                .with_waste_not(waste_not)
                .with_manipulation(manipulation)
                .with_trained_perfection(trained_perfection)
                .with_guard(1),
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
            combo: self.combo,
        }
    }
}
