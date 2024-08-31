use simulator::{Combo, Effects, SimulationState, SingleUse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: i16,
    pub unreliable_quality: u8,
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

    pub fn from_state(state: SimulationState, durability_cost: i16, base_quality: u16) -> Self {
        let used_durability = (i8::MAX - state.durability) / 5;
        let cp = state.cp - used_durability as i16 * durability_cost;
        let great_strides_active = state.effects.great_strides() != 0;
        Self {
            cp,
            // compress into units of base_quality, rounded up
            unreliable_quality: ((state.unreliable_quality + base_quality - 1) / base_quality)
                as u8,
            combo: state.combo,
            effects: state
                .effects
                .with_great_strides(if great_strides_active { 3 } else { 0 })
                .with_trained_perfection(SingleUse::Unavailable)
                .with_manipulation(0),
        }
    }

    pub fn to_state(self, base_quality: u16) -> SimulationState {
        SimulationState {
            durability: i8::MAX,
            cp: self.cp,
            progress: 0,
            quality: 0,
            unreliable_quality: self.unreliable_quality as u16 * base_quality,
            effects: self.effects,
            combo: self.combo,
        }
    }
}
