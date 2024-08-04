use simulator::{Combo, Effects, SimulationState, SingleUse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedEffects {
    pub inner_quiet: u8,
    pub innovation: u8,
    pub veneration: u8,
    pub great_strides: u8,
    pub muscle_memory: u8,
    pub heart_and_soul: SingleUse,
    pub quick_innovation_used: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: i16,
    pub combo: Combo,
    pub effects: ReducedEffects,
}

impl ReducedState {
    pub fn from_state(
        state: SimulationState,
        base_durability_cost: i16,
        waste_not_cost: i16,
    ) -> Self {
        let used_durability = (i8::MAX - state.durability) / 5;
        let durability_cost = std::cmp::min(
            used_durability as i16 * base_durability_cost,
            (used_durability as i16 + 1) / 2 * base_durability_cost + waste_not_cost,
        );
        Self {
            cp: state.cp - durability_cost,
            combo: state.combo,
            effects: ReducedEffects {
                inner_quiet: state.effects.inner_quiet(),
                innovation: state.effects.innovation(),
                veneration: state.effects.veneration(),
                great_strides: state.effects.great_strides(),
                muscle_memory: state.effects.muscle_memory(),
                heart_and_soul: state.effects.heart_and_soul(),
                quick_innovation_used: state.effects.quick_innovation_used(),
            },
        }
    }
}

impl std::convert::From<ReducedState> for SimulationState {
    fn from(state: ReducedState) -> Self {
        SimulationState {
            durability: i8::MAX,
            cp: state.cp,
            progress: 0,
            unreliable_quality: [0, 0],
            effects: Effects::new()
                .with_inner_quiet(state.effects.inner_quiet)
                .with_innovation(state.effects.innovation)
                .with_veneration(state.effects.veneration)
                .with_great_strides(state.effects.great_strides)
                .with_muscle_memory(state.effects.muscle_memory)
                .with_trained_perfection(SingleUse::Unavailable)
                .with_heart_and_soul(state.effects.heart_and_soul)
                .with_quick_innovation_used(state.effects.quick_innovation_used)
                .with_guard(1),
            combo: state.combo,
        }
        .try_into()
        .unwrap()
    }
}
