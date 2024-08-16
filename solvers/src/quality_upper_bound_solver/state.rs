use simulator::{Combo, Effects, SimulationState, SingleUse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: i16,
    pub combo: Combo,
    pub effects: Effects,
}

impl ReducedState {
    pub fn to_non_combo(self) -> Self {
        Self {
            combo: Combo::None,
            ..self
        }
    }

    pub fn from_state(state: SimulationState, durability_cost: i16) -> Self {
        let used_durability = (i8::MAX - state.durability) / 5;
        let cp = state.cp - used_durability as i16 * durability_cost;
        let great_strides_active = state.effects.great_strides() != 0;
        Self {
            cp,
            combo: state.combo,
            effects: state
                .effects
                .with_great_strides(if great_strides_active { 3 } else { 0 })
                .with_trained_perfection(SingleUse::Unavailable)
                .with_manipulation(0)
                .with_guard(1),
        }
    }

    pub fn to_state(self) -> SimulationState {
        SimulationState {
            durability: i8::MAX,
            cp: self.cp,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: self.effects,
            combo: self.combo,
        }
    }
}
