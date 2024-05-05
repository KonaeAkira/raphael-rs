use crate::{
    game::{state::InProgress, units::*, Action, Condition, Effects, Settings, State},
    solvers::action_sequences::{ALL_PROGRESS_ACTIONS, DURABILITY_ACTIONS},
};

use constcat::concat_slices;
use rustc_hash::FxHashMap as HashMap;

use super::action_sequences::ActionSequence;

const ACTION_SEQUENCES: &[ActionSequence] =
    concat_slices!([ActionSequence]: ALL_PROGRESS_ACTIONS, DURABILITY_ACTIONS);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedEffects {
    pub waste_not: u8,
    pub veneration: u8,
    pub manipulation: u8,
}

impl ReducedEffects {
    pub fn from_effects(effects: &Effects) -> ReducedEffects {
        ReducedEffects {
            waste_not: effects.waste_not,
            veneration: effects.veneration,
            manipulation: effects.manipulation,
        }
    }

    pub fn to_effects(self) -> Effects {
        Effects {
            inner_quiet: 0,
            waste_not: self.waste_not,
            innovation: 0,
            veneration: self.veneration,
            great_strides: 0,
            muscle_memory: 0,
            manipulation: self.manipulation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    durability: Durability,
    cp: CP,
    effects: ReducedEffects,
}

impl ReducedState {
    pub const INF_PROGRESS: Progress = Progress::new(100_000);

    pub fn from_state(state: &InProgress) -> ReducedState {
        ReducedState {
            durability: state.durability,
            cp: state.cp,
            effects: ReducedEffects::from_effects(&state.effects),
        }
    }

    pub fn to_state(self) -> InProgress {
        InProgress {
            durability: self.durability,
            cp: self.cp,
            missing_progress: Self::INF_PROGRESS,
            missing_quality: Quality::new(0),
            effects: self.effects.to_effects(),
            combo: None,
        }
    }
}

#[derive(Debug)]
pub struct FinishSolver {
    settings: Settings,
    // maps ReducedState to maximum additional progress that can be gained from that state
    memoization: HashMap<ReducedState, Progress>,
}

impl FinishSolver {
    pub fn new(settings: Settings) -> FinishSolver {
        FinishSolver {
            settings,
            memoization: HashMap::default(),
        }
    }

    pub fn get_finish_sequence(&self, state: &InProgress) -> Option<Vec<Action>> {
        let reduced_state = ReducedState::from_state(state);
        match self.memoization.get(&reduced_state) {
            Some(progress) => {
                if state.missing_progress <= *progress {
                    let mut result: Vec<Action> = Vec::new();
                    self.do_trace(&mut result, reduced_state, *progress);
                    Some(self.truncate(*state, &result))
                } else {
                    None
                }
            }
            None => None,
        }
    }

    fn truncate(&self, mut state: InProgress, actions: &[Action]) -> Vec<Action> {
        let mut result: Vec<Action> = Vec::new();
        for action in actions {
            match state.use_action(*action, Condition::Normal, &self.settings) {
                State::InProgress(in_progress) => {
                    result.push(*action);
                    state = in_progress;
                }
                State::Completed { quality: _ } => {
                    result.push(*action);
                    break;
                }
                _ => panic!("Invalid finish sequence"),
            }
        }
        result
    }

    fn do_trace(&self, result: &mut Vec<Action>, state: ReducedState, target_progress: Progress) {
        if target_progress == Progress::new(0) {
            return;
        }
        for sequence in ACTION_SEQUENCES {
            match State::InProgress(state.to_state()).use_actions(
                sequence,
                Condition::Normal,
                &self.settings,
            ) {
                State::InProgress(new_state) => {
                    let gained_progress =
                        ReducedState::INF_PROGRESS.saturating_sub(new_state.missing_progress);
                    let new_state = ReducedState::from_state(&new_state);
                    let new_state_potential = *self.memoization.get(&new_state).unwrap();
                    if gained_progress.saturating_add(new_state_potential) == target_progress {
                        result.extend_from_slice(sequence);
                        self.do_trace(result, new_state, new_state_potential);
                        return;
                    }
                }
                State::Failed { missing_progress } => {
                    let gained_progress =
                        ReducedState::INF_PROGRESS.saturating_sub(missing_progress);
                    if gained_progress == target_progress {
                        result.extend_from_slice(sequence);
                        return;
                    }
                }
                State::Completed { .. } => {
                    panic!("This state should never be reached. INF_PROGRESS probably isn't high enough.")
                }
                State::Invalid => (),
            }
        }
        panic!("Unable to find a trace.")
    }

    pub fn can_finish(&mut self, state: &InProgress) -> bool {
        state.missing_progress <= self.do_solve(ReducedState::from_state(state))
    }

    fn do_solve(&mut self, state: ReducedState) -> Progress {
        match self.memoization.get(&state) {
            Some(progress) => *progress,
            None => {
                let mut max_progress = Progress::new(0);
                for sequence in ACTION_SEQUENCES {
                    match State::InProgress(state.to_state()).use_actions(
                        sequence,
                        Condition::Normal,
                        &self.settings,
                    ) {
                        State::InProgress(new_state) => {
                            let gained_progress = ReducedState::INF_PROGRESS
                                .saturating_sub(new_state.missing_progress);
                            let new_state_potential =
                                self.do_solve(ReducedState::from_state(&new_state));
                            max_progress = std::cmp::max(
                                max_progress,
                                gained_progress.saturating_add(new_state_potential),
                            );
                        }
                        State::Failed { missing_progress } => {
                            let gained_progress =
                                ReducedState::INF_PROGRESS.saturating_sub(missing_progress);
                            max_progress = std::cmp::max(max_progress, gained_progress);
                        }
                        State::Completed { .. } => {
                            panic!("This state should never be reached. INF_PROGRESS probably isn't high enough.")
                        }
                        State::Invalid => (),
                    }
                    if max_progress >= self.settings.max_progress {
                        break;
                    }
                }
                self.memoization.insert(state, max_progress);
                max_progress
            }
        }
    }
}

impl Drop for FinishSolver {
    fn drop(&mut self) {
        let finish_solver_states = self.memoization.len();
        dbg!(finish_solver_states);
    }
}
