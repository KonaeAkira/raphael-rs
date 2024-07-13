use simulator::{state::InProgress, ComboAction, Effects, SimulationState, SingleUse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedEffects {
    pub inner_quiet: u8,
    pub innovation: u8,
    pub veneration: u8,
    pub great_strides: u8,
    pub muscle_memory: u8,
    pub trained_perfection: SingleUse,
    pub tricks: bool,
    pub guard: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: i16,
    pub unreliable_quality: u16,
    pub unreliable_diff: i16,
    pub prev_was_guarded: bool,
    pub combo: Option<ComboAction>,
    pub effects: ReducedEffects,
}

impl ReducedState {
    pub fn from_state(state: InProgress, base_durability_cost: i16, waste_not_cost: i16) -> Self {
        let state = *state.raw_state();
        let used_durability = (i8::MAX - state.durability) / 5;
        let durability_cost = std::cmp::min(
            used_durability as i16 * base_durability_cost,
            (used_durability as i16 + 1) / 2 * base_durability_cost + waste_not_cost,
        );
        Self {
            cp: state.cp - durability_cost,
            combo: state.combo,
            unreliable_quality: state.unreliable_quality,
            unreliable_diff: state.unreliable_diff,
            prev_was_guarded: state.prev_was_guarded,
            effects: ReducedEffects {
                inner_quiet: state.effects.inner_quiet(),
                innovation: state.effects.innovation(),
                veneration: state.effects.veneration(),
                great_strides: state.effects.great_strides(),
                muscle_memory: state.effects.muscle_memory(),
                trained_perfection: state.effects.trained_perfection(),
                tricks: state.effects.tricks(),
                guard: state.effects.guard(),
            },
        }
    }
}

impl std::convert::From<ReducedState> for InProgress {
    fn from(state: ReducedState) -> Self {
        SimulationState {
            durability: i8::MAX,
            cp: state.cp,
            missing_progress: u16::MAX,
            missing_quality: u16::MAX,
            unreliable_quality: state.unreliable_quality,
            unreliable_diff: state.unreliable_diff,
            prev_was_guarded: state.prev_was_guarded,
            effects: Effects::new()
                .with_inner_quiet(state.effects.inner_quiet)
                .with_innovation(state.effects.innovation)
                .with_veneration(state.effects.veneration)
                .with_great_strides(state.effects.great_strides)
                .with_muscle_memory(state.effects.muscle_memory)
                .with_trained_perfection(state.effects.trained_perfection)
                .with_tricks(state.effects.tricks)
                .with_guard(state.effects.guard),
            combo: state.combo,
        }
        .try_into()
        .unwrap()
    }
}
