use raphael_sim::*;
use rustc_hash::FxHashMap;

use crate::{
    SolverSettings,
    actions::{PROGRESS_ONLY_SEARCH_ACTIONS, use_action_combo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReducedState {
    durability: u16,
    cp: u16,
    effects: Effects,
}

impl ReducedState {
    fn from_state(state: &SimulationState) -> Self {
        Self {
            durability: state.durability,
            cp: state.cp,
            effects: state.effects.strip_quality_effects(),
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
        }
    }
}

pub struct FinishSolver {
    settings: SolverSettings,
    solved_states: FxHashMap<ReducedState, u32>,
}

pub struct FinishSolverShard<'a> {
    settings: &'a SolverSettings,
    shared_states: &'a FxHashMap<ReducedState, u32>,
    local_states: FxHashMap<ReducedState, u32>,
}

impl FinishSolver {
    pub fn new(settings: SolverSettings) -> Self {
        Self {
            settings,
            solved_states: FxHashMap::default(),
        }
    }

    pub fn extend_solved_states(&mut self, new_solved_states: Vec<(ReducedState, u32)>) {
        self.solved_states.extend(new_solved_states);
    }

    pub fn create_shard(&self) -> FinishSolverShard<'_> {
        FinishSolverShard {
            settings: &self.settings,
            shared_states: &self.solved_states,
            local_states: FxHashMap::default(),
        }
    }

    pub fn can_finish(&mut self, state: &SimulationState) -> bool {
        let max_progress = self.solve_max_progress(ReducedState::from_state(state));
        state.progress + max_progress >= self.settings.max_progress()
    }

    fn solve_max_progress(&mut self, state: ReducedState) -> u32 {
        match self.solved_states.get(&state) {
            Some(max_progress) => *max_progress,
            None => {
                let mut max_progress = 0;
                for action in PROGRESS_ONLY_SEARCH_ACTIONS {
                    if let Ok(new_state) =
                        use_action_combo(&self.settings, state.to_state(), action)
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
                    if max_progress >= self.settings.max_progress() {
                        // stop early if progress is already maxed out
                        // this optimization would work better with a better action ordering
                        max_progress = self.settings.max_progress();
                        break;
                    }
                }
                self.solved_states.insert(state, max_progress);
                max_progress
            }
        }
    }

    pub fn num_states(&self) -> usize {
        self.solved_states.len()
    }
}

impl<'a> FinishSolverShard<'a> {
    pub fn can_finish(&mut self, state: &SimulationState) -> bool {
        let max_progress = self.solve_max_progress(ReducedState::from_state(state));
        state.progress + max_progress >= self.settings.max_progress()
    }

    pub fn into_solved_states(self) -> impl Iterator<Item = (ReducedState, u32)> {
        self.local_states.into_iter()
    }

    fn solve_max_progress(&mut self, state: ReducedState) -> u32 {
        if let Some(max_progress) = self.shared_states.get(&state) {
            *max_progress
        } else if let Some(max_progress) = self.local_states.get(&state) {
            *max_progress
        } else {
            let mut max_progress = 0;
            for action in PROGRESS_ONLY_SEARCH_ACTIONS {
                if let Ok(new_state) = use_action_combo(&self.settings, state.to_state(), action) {
                    if new_state.is_final(&self.settings.simulator_settings) {
                        max_progress = std::cmp::max(max_progress, new_state.progress);
                    } else {
                        let child_progress =
                            self.solve_max_progress(ReducedState::from_state(&new_state));
                        max_progress =
                            std::cmp::max(max_progress, child_progress + new_state.progress);
                    }
                }
                if max_progress >= self.settings.max_progress() {
                    // stop early if progress is already maxed out
                    // this optimization would work better with a better action ordering
                    max_progress = self.settings.max_progress();
                    break;
                }
            }
            self.local_states.insert(state, max_progress);
            max_progress
        }
    }
}
