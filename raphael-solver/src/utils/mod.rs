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

struct Entry<T> {
    item: T,
    depth: u8,
    parent_index: usize,
}

pub struct Backtracking<T: Copy> {
    entries: Vec<Entry<T>>,
}

impl<T: Copy> Backtracking<T> {
    pub const SENTINEL: usize = usize::MAX;

    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn get_items(&self, mut index: usize) -> impl Iterator<Item = T> {
        let mut items = Vec::new();
        while index != Self::SENTINEL {
            items.push(self.entries[index].item);
            index = self.entries[index].parent_index;
        }
        items.into_iter().rev()
    }

    pub fn push(&mut self, item: T, parent_index: usize) -> usize {
        let depth = if parent_index == Self::SENTINEL {
            1
        } else {
            self.entries[parent_index].depth + 1
        };
        self.entries.push(Entry {
            item,
            depth,
            parent_index,
        });
        self.entries.len() - 1
    }
}

impl<T: Copy> Drop for Backtracking<T> {
    fn drop(&mut self) {
        log::debug!("Backtracking - nodes: {}", self.entries.len());
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
                .with_allow_quality_actions(true)
                .with_adversarial_guard(true)
                .with_inner_quiet(iq),
        };
        for &action in FULL_SEARCH_ACTIONS {
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

pub fn largest_single_action_progress_increase(settings: &SolverSettings) -> u32 {
    let state = SimulationState::new(&settings.simulator_settings);
    assert_eq!(state.progress, 0);
    FULL_SEARCH_ACTIONS
        .iter()
        .filter_map(|&action| {
            use_action_combo(settings, state, action)
                .ok()
                .map(|state| state.progress)
        })
        .max()
        .unwrap()
}
