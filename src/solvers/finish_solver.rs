use crate::{
    game::{state::InProgress, units::*, Action, ComboAction, Condition, Effects, Settings, State},
    solvers::action_sequences::{DURABILITY_ACTIONS, PROGRESS_ACTIONS},
};

use constcat::concat_slices;
use rustc_hash::FxHashMap as HashMap;

use super::action_sequences::ActionSequence;

const ACTION_SEQUENCES: &[ActionSequence] =
    concat_slices!([ActionSequence]: PROGRESS_ACTIONS, DURABILITY_ACTIONS);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedEffects {
    pub muscle_memory: u8,
    pub waste_not: u8,
    pub veneration: u8,
    pub manipulation: u8,
}

impl ReducedEffects {
    pub fn from_effects(effects: &Effects) -> ReducedEffects {
        ReducedEffects {
            muscle_memory: effects.muscle_memory,
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
            muscle_memory: self.muscle_memory,
            manipulation: self.manipulation,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    durability: Durability,
    cp: CP,
    effects: ReducedEffects,
    combo: Option<ComboAction>,
}

impl ReducedState {
    pub const INF_PROGRESS: Progress = Progress::new(100_000);

    pub fn from_state(state: &InProgress) -> ReducedState {
        ReducedState {
            durability: state.durability,
            cp: state.cp,
            effects: ReducedEffects::from_effects(&state.effects),
            combo: state.combo,
        }
    }

    pub fn to_state(self) -> InProgress {
        InProgress {
            durability: self.durability,
            cp: self.cp,
            missing_progress: Self::INF_PROGRESS,
            missing_quality: Quality::new(0),
            effects: self.effects.to_effects(),
            combo: self.combo,
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

    pub fn get_finish_sequence(&mut self, mut state: InProgress) -> Option<Vec<Action>> {
        if !self.can_finish(&state) {
            return None;
        }
        let mut finish_sequence: Vec<Action> = Vec::new();
        loop {
            for actions in ACTION_SEQUENCES {
                match State::InProgress(state).use_actions(
                    actions,
                    Condition::Normal,
                    &self.settings,
                ) {
                    State::InProgress(new_state) => {
                        if self.can_finish(&new_state) {
                            finish_sequence.extend_from_slice(actions);
                            state = State::InProgress(state)
                                .use_actions(actions, Condition::Normal, &self.settings)
                                .as_in_progress()
                                .unwrap();
                        }
                    }
                    State::Completed { missing_quality: _ } => {
                        finish_sequence.extend_from_slice(actions);
                        return Some(finish_sequence);
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn can_finish(&mut self, state: &InProgress) -> bool {
        state.missing_progress <= self._do_solve(ReducedState::from_state(state))
    }

    fn _do_solve(&mut self, state: ReducedState) -> Progress {
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
                            let gained_progress =
                                ReducedState::INF_PROGRESS.sub(new_state.missing_progress);
                            let new_state_potential =
                                self._do_solve(ReducedState::from_state(&new_state));
                            max_progress = std::cmp::max(
                                max_progress,
                                gained_progress.add(new_state_potential),
                            );
                        }
                        State::Failed { missing_progress } => {
                            let gained_progress = ReducedState::INF_PROGRESS.sub(missing_progress);
                            max_progress = std::cmp::max(max_progress, gained_progress);
                        }
                        State::Completed { .. } => {
                            unreachable!("INF_PROGRESS not high enough")
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
