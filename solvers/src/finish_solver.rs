use simulator::*;

use rustc_hash::FxHashMap as HashMap;

use crate::{
    SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    durability: i8,
    cp: i16,
    effects: Effects,
    combo: Combo,
}

impl ReducedState {
    fn from_state(state: &SimulationState) -> Self {
        Self {
            durability: state.durability,
            cp: state.cp,
            effects: state
                .effects
                .with_inner_quiet(0)
                .with_innovation(0)
                .with_great_strides(0)
                .with_guard(0)
                .with_quick_innovation_available(false),
            combo: state.combo,
        }
    }

    fn to_state(self) -> SimulationState {
        SimulationState {
            durability: self.durability,
            cp: self.cp,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: self.effects,
            combo: self.combo,
        }
    }
}

pub struct FinishSolver {
    settings: SolverSettings,
    // maximum attainable progress for each state
    max_progress: HashMap<ReducedState, u16>,
}

impl FinishSolver {
    pub fn new(settings: SolverSettings) -> Self {
        log::trace!(
            "ReducedState (FinishSolver) - size: {}, align: {}",
            std::mem::size_of::<ReducedState>(),
            std::mem::align_of::<ReducedState>()
        );
        Self {
            settings,
            max_progress: HashMap::default(),
        }
    }

    pub fn can_finish(&mut self, state: &SimulationState) -> bool {
        let max_progress = self.solve_max_progress(ReducedState::from_state(state));
        state.progress + max_progress >= self.settings.simulator_settings.max_progress
    }

    fn solve_max_progress(&mut self, state: ReducedState) -> u16 {
        match self.max_progress.get(&state) {
            Some(max_progress) => *max_progress,
            None => {
                let mut max_progress = 0;
                for action in FULL_SEARCH_ACTIONS {
                    if let Ok(new_state) =
                        use_action_combo(&self.settings, state.to_state(), *action)
                    {
                        if new_state.is_final(&self.settings.simulator_settings) {
                            max_progress = std::cmp::max(max_progress, new_state.progress);
                        } else {
                            let child_progress =
                                self.solve_max_progress(ReducedState::from_state(&new_state));
                            max_progress =
                                std::cmp::max(max_progress, child_progress + new_state.progress);
                        }
                    }
                    if max_progress >= self.settings.simulator_settings.max_progress {
                        // stop early if progress is already maxed out
                        // this optimization would work better with a better action ordering
                        max_progress = self.settings.simulator_settings.max_progress;
                        break;
                    }
                }
                self.max_progress.insert(state, max_progress);
                max_progress
            }
        }
    }
}

impl Drop for FinishSolver {
    fn drop(&mut self) {
        log::debug!("FinishSolver - states: {}", self.max_progress.len());
    }
}
