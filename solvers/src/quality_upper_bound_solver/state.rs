use simulator::{Combo, Effects, SimulationState, SingleUse};

pub trait ReducedState: Clone + Copy + PartialEq + Eq + std::hash::Hash {
    fn from_state(state: SimulationState, durability_cost: i16, waste_not_cost: i16) -> Self;
    fn to_state(self) -> SimulationState;
    fn cp(&self) -> i16;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedStateFast {
    pub cp: i16,
    pub combo: Combo,
    pub effects: Effects,
}

impl ReducedState for ReducedStateFast {
    fn cp(&self) -> i16 {
        self.cp
    }

    fn from_state(state: SimulationState, durability_cost: i16, waste_not_cost: i16) -> Self {
        let used_durability = (i8::MAX - state.durability) / 5;
        let used_durability_cost = std::cmp::min(
            used_durability as i16 * durability_cost,
            (used_durability as i16 + 1) / 2 * durability_cost + waste_not_cost,
        );
        let cp =
            state.cp - used_durability_cost + state.effects.waste_not() as i16 * waste_not_cost;
        Self {
            cp,
            combo: state.combo,
            effects: state
                .effects
                .with_trained_perfection(SingleUse::Unavailable)
                .with_waste_not(0)
                .with_manipulation(0)
                .with_guard(1),
        }
    }

    fn to_state(self) -> SimulationState {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedStateSlow {
    pub cp: i16,
    pub combo: Combo,
    pub effects: Effects,
}

impl ReducedState for ReducedStateSlow {
    fn cp(&self) -> i16 {
        self.cp
    }

    fn from_state(state: SimulationState, durability_cost: i16, _waste_not_cost: i16) -> Self {
        let used_durability = (i8::MAX - state.durability) / 5;
        let cp = state.cp - used_durability as i16 * durability_cost;
        Self {
            cp,
            combo: state.combo,
            effects: state
                .effects
                .with_trained_perfection(SingleUse::Unavailable)
                .with_manipulation(0)
                .with_guard(1),
        }
    }

    fn to_state(self) -> SimulationState {
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
