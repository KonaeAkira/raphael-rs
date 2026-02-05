mod atomic_flag;
mod pareto_front_builder;

pub use atomic_flag::AtomicFlag;
pub use pareto_front_builder::{ParetoFrontBuilder, ParetoValue};
use raphael_sim::*;

use crate::{
    SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
};

pub struct ScopedTimer {
    name: &'static str,
    timer: web_time::Instant,
}

impl ScopedTimer {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            timer: web_time::Instant::now(),
        }
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        log::info!(
            "Timer \"{}\" elapsed: {} seconds",
            self.name,
            self.timer.elapsed().as_secs_f32()
        );
    }
}

/// The only way to increase the InnerQuiet effect is to use Quality-increasing actions,
/// which means that all states with InnerQuiet must have some amount of Quality.
/// This function finds a lower-bound on the minimum amount of Quality a state with `n` InnerQuiet can have.
pub fn compute_iq_quality_lut(settings: &SolverSettings) -> [u32; 11] {
    if settings.simulator_settings.adversarial {
        // TODO: implement this for adversarial mode
        return [0; 11];
    }
    if settings
        .simulator_settings
        .is_action_allowed::<HeartAndSoul>()
    {
        // TODO: implement this for heart and soul
        return [0; 11];
    }
    let mut result = [u32::MAX; 11];
    result[0] = 0;
    for iq in 0..10 {
        let state = SimulationState {
            cp: 500,
            durability: 100,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: Effects::new()
                .with_special_quality_state(SpecialQualityState::AdversarialGuard)
                .with_inner_quiet(iq),
        };
        for action in FULL_SEARCH_ACTIONS {
            if let Ok(new_state) = use_action_combo(settings, state, action) {
                let new_iq = new_state.effects.inner_quiet();
                if new_iq > iq {
                    let action_quality = new_state.quality;
                    result[usize::from(new_iq)] = std::cmp::min(
                        result[usize::from(new_iq)],
                        result[usize::from(iq)] + action_quality,
                    );
                }
            }
        }
    }
    result
}

/// Returns the maximum additional Progress gained by having the Muscle Memory effect.
pub fn maximum_muscle_memory_utilization(settings: &Settings) -> u32 {
    let mut state = SimulationState::new(settings);
    let mut result = 0;
    if settings.is_action_allowed::<BasicSynthesis>() {
        result = result.max(BasicSynthesis::progress_increase(&state, settings));
    }
    if settings.is_action_allowed::<CarefulSynthesis>() {
        result = result.max(CarefulSynthesis::progress_increase(&state, settings));
    }
    if settings.is_action_allowed::<Groundwork>() {
        // Prevent Groundwork efficiency from halving in very-low durability settings.
        state.effects.set_trained_perfection_active(true);
        result = result.max(Groundwork::progress_increase(&state, settings));
    }
    if settings.is_action_allowed::<PrudentSynthesis>() {
        result = result.max(PrudentSynthesis::progress_increase(&state, settings));
    }
    if settings.is_action_allowed::<DelicateSynthesis>() {
        result = result.max(DelicateSynthesis::progress_increase(&state, settings));
    }
    if settings.is_action_allowed::<HeartAndSoul>()
        && settings.is_action_allowed::<IntensiveSynthesis>()
    {
        result = result.max(IntensiveSynthesis::progress_increase(&state, settings));
    }
    if settings.stellar_steady_hand_charges != 0
        && settings.is_action_allowed::<StellarSteadyHand>()
        && settings.is_action_allowed::<RapidSynthesis>()
    {
        result = result.max(RapidSynthesis::progress_increase(&state, settings));
    }
    result
}
