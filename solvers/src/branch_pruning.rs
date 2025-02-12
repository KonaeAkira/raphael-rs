use simulator::{Combo, SimulationState};

pub fn is_progress_only_state(
    state: &SimulationState,
    backload_progress: bool,
    allow_unsound: bool,
) -> bool {
    let mut progress_only = false;
    progress_only |= backload_progress && state.progress != 0;
    if allow_unsound {
        progress_only |= backload_progress && state.effects.veneration() != 0;
        // only allow increasing Progress after using Byregot's Blessing
        progress_only |= state.quality != 0 && state.effects.inner_quiet() == 0;
    }
    progress_only
}

pub fn strip_quality_effects(state: SimulationState) -> SimulationState {
    SimulationState {
        unreliable_quality: 0,
        effects: state
            .effects
            .with_inner_quiet(0)
            .with_innovation(0)
            .with_great_strides(0)
            .with_guard(0)
            .with_quick_innovation_available(false),
        combo: match state.combo {
            Combo::None => Combo::None,
            Combo::SynthesisBegin => Combo::SynthesisBegin,
            Combo::BasicTouch => Combo::None,
            Combo::StandardTouch => Combo::None,
        },
        ..state
    }
}
