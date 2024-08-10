use simulator::{Combo, Effects, SimulationState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub cp: i16,
    pub combo: Combo,
    pub effects: Effects,
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
            effects: state
                .effects
                .with_waste_not(0)
                .with_manipulation(0)
                .with_guard(1),
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
            effects: state.effects,
            combo: state.combo,
        }
    }
}
