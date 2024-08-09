use simulator::{Action, ActionMask, Combo, Effects, SimulationState, SingleUse};

use crate::actions::DURABILITY_ACTIONS;

pub trait ReducedState: Clone + Copy + PartialEq + Eq + std::hash::Hash {
    fn optimize_action_mask(action_mask: ActionMask) -> ActionMask;
    fn steps_budget(&self) -> u8;
    fn from_state(state: SimulationState, steps_budget: u8) -> Self;
    fn to_state(self) -> SimulationState;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedStateWithDurability {
    steps_budget: u8,
    durability: i8,
    combo: Combo,
    inner_quiet: u8,
    innovation: u8,
    veneration: u8,
    great_strides: bool,
    muscle_memory: u8,
    waste_not: bool,
    manipulation: bool,
    trained_perfection: SingleUse,
    heart_and_soul: SingleUse,
    quick_innovation_used: bool,
}

impl ReducedState for ReducedStateWithDurability {
    fn optimize_action_mask(mut action_mask: ActionMask) -> ActionMask {
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

    fn steps_budget(&self) -> u8 {
        self.steps_budget
    }

    fn from_state(state: SimulationState, steps_budget: u8) -> Self {
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
            inner_quiet: state.effects.inner_quiet(),
            innovation: state.effects.innovation(),
            veneration: state.effects.veneration(),
            // Mapping GreatStrides to a bool makes the state-space smaller.
            // The trade-off is a slightly less tight step lower-bound
            great_strides: state.effects.great_strides() != 0,
            muscle_memory: state.effects.muscle_memory(),
            // Mapping WasteNot and Manipulation to bools makes the state-space smaller.
            // The trade-off is a slightly less tight step lower-bound
            waste_not: state.effects.waste_not() != 0,
            manipulation: state.effects.manipulation() != 0,
            trained_perfection: match state.effects.trained_perfection() {
                // Mapping Unavailable to Available makes the state-space smaller.
                // The trade-off is a slightly less tight step lower-bound
                SingleUse::Unavailable => SingleUse::Available,
                SingleUse::Available => SingleUse::Available,
                SingleUse::Active => SingleUse::Active,
            },
            heart_and_soul: state.effects.heart_and_soul(),
            quick_innovation_used: state.effects.quick_innovation_used(),
        }
    }

    fn to_state(self) -> SimulationState {
        SimulationState {
            durability: self.durability,
            cp: 1000,
            progress: 0,
            unreliable_quality: [0, 0],
            effects: Effects::new()
                .with_inner_quiet(self.inner_quiet)
                .with_innovation(self.innovation)
                .with_veneration(self.veneration)
                .with_great_strides(if self.great_strides { 3 } else { 0 })
                .with_muscle_memory(self.muscle_memory)
                .with_waste_not(if self.waste_not { 8 } else { 0 })
                .with_manipulation(if self.manipulation { 8 } else { 0 })
                .with_trained_perfection(self.trained_perfection)
                .with_heart_and_soul(self.heart_and_soul)
                .with_quick_innovation_used(self.quick_innovation_used)
                .with_guard(1),
            combo: self.combo,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedStateWithoutDurability {
    steps_budget: u8,
    combo: Combo,
    inner_quiet: u8,
    innovation: u8,
    veneration: u8,
    great_strides: bool,
    muscle_memory: u8,
    heart_and_soul: SingleUse,
    quick_innovation_used: bool,
}

impl ReducedState for ReducedStateWithoutDurability {
    fn optimize_action_mask(action_mask: ActionMask) -> ActionMask {
        // There are a lot more actions that can be optimized out, but the performance gain is probably not worth the effort because the StepLowerBoundSolver is already so fast for this ReducedState variant.
        action_mask.minus(DURABILITY_ACTIONS)
    }

    fn steps_budget(&self) -> u8 {
        self.steps_budget
    }

    fn from_state(state: SimulationState, steps_budget: u8) -> Self {
        Self {
            steps_budget,
            combo: match state.combo {
                Combo::None => Combo::None,
                Combo::SynthesisBegin => Combo::SynthesisBegin,
                // StandardTouch replaces ComboStandardTouch (no CP cost, next combo not needed)
                // PreparatoryTouch replaces RefinedTouch (no CP cost, no durability cost)
                Combo::BasicTouch => Combo::None,
                // AdvancedTouch replaces ComboAdvancedTouch (no CP cost)
                Combo::StandardTouch => Combo::None,
            },
            inner_quiet: state.effects.inner_quiet(),
            innovation: state.effects.innovation(),
            veneration: state.effects.veneration(),
            great_strides: state.effects.great_strides() != 0,
            muscle_memory: state.effects.muscle_memory(),
            heart_and_soul: state.effects.heart_and_soul(),
            quick_innovation_used: state.effects.quick_innovation_used(),
        }
    }

    fn to_state(self) -> SimulationState {
        SimulationState {
            durability: i8::MAX,
            cp: 1000,
            progress: 0,
            unreliable_quality: [0, 0],
            effects: Effects::new()
                .with_inner_quiet(self.inner_quiet)
                .with_innovation(self.innovation)
                .with_veneration(self.veneration)
                .with_great_strides(if self.great_strides { 3 } else { 0 })
                .with_muscle_memory(self.muscle_memory)
                .with_heart_and_soul(self.heart_and_soul)
                .with_quick_innovation_used(self.quick_innovation_used)
                .with_guard(1),
            combo: self.combo,
        }
    }
}
