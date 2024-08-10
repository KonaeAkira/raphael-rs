use simulator::{Combo, Effects, SimulationState, SingleUse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: i16,
    pub combo: Combo,
    pub effects: Effects,
}

impl ReducedState {
    pub fn from_state(state: SimulationState, base_durability_cost: i16) -> Self {
        let used_durability = (i8::MAX - state.durability) / 5;
        let durability_cost = used_durability as i16 * base_durability_cost;
        Self {
            cp: state.cp - durability_cost,
            combo: state.combo,
            effects: state
                .effects
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
            unreliable_quality: [0, 0],
            effects: self.effects,
            combo: self.combo,
        }
    }
}
