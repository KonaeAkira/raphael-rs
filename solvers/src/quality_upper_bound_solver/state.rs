use simulator::{Combo, Effects, SimulationState, SingleUse};

#[bitfield_struct::bitfield(u32)]
#[derive(PartialEq, Eq, Hash)]
pub struct ReducedStateData {
    pub cp: i16,
    pub unreliable_quality: u8,
    #[bits(2, default=Combo::None)]
    pub combo: Combo,
    #[bits(1)]
    pub progress_only: bool,
    #[bits(5)]
    _padding: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    pub data: ReducedStateData,
    pub effects: Effects,
}

impl ReducedState {
    pub fn to_non_combo(self) -> Self {
        Self {
            data: self.data.with_combo(Combo::None),
            effects: self.effects,
        }
    }

    pub fn from_state(
        state: SimulationState,
        progress_only: bool,
        durability_cost: i16,
        base_quality: u16,
    ) -> Self {
        let used_durability = (i8::MAX - state.durability) / 5;
        let cp = state.cp - used_durability as i16 * durability_cost;
        let unreliable_quality = if progress_only {
            0
        } else {
            ((state.unreliable_quality + 2 * base_quality - 1) / (2 * base_quality)) as u8
        };
        let effects = if progress_only {
            state
                .effects
                .with_inner_quiet(0)
                .with_innovation(0)
                .with_great_strides(0)
                .with_guard(0)
                .with_quick_innovation_used(true)
                .with_trained_perfection(SingleUse::Unavailable)
                .with_manipulation(0)
        } else {
            let great_strides_active = state.effects.great_strides() != 0;
            state
                .effects
                .with_great_strides(if great_strides_active { 3 } else { 0 })
                .with_trained_perfection(SingleUse::Unavailable)
                .with_manipulation(0)
        };
        Self {
            data: ReducedStateData::new()
                .with_cp(cp)
                .with_unreliable_quality(unreliable_quality)
                .with_combo(state.combo)
                .with_progress_only(progress_only),
            effects,
        }
    }

    pub fn to_state(self, base_quality: u16) -> SimulationState {
        SimulationState {
            durability: i8::MAX,
            cp: self.data.cp(),
            progress: 0,
            quality: 0,
            unreliable_quality: self.data.unreliable_quality() as u16 * base_quality * 2,
            effects: self.effects,
            combo: self.data.combo(),
        }
    }
}
