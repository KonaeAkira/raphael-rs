use std::hash::Hash;

use rustc_hash::FxHashMap as HashMap;

use crate::{
    game::{
        state::InProgress,
        units::{Durability, Progress, Quality, CP},
        Condition, Effects, Settings, State,
    },
    solvers::action_sequences::{ActionSequence, ALL_PROGRESS_ACTIONS},
};

use super::{DURABILITY_COST, WASTE_NOT_COST};

const MAX_DURABILITY: Durability = 100;
const MAX_PROGRESS: Progress = Progress::new(100_000);
const MAX_QUALITY: Quality = Quality::new(100_000);

const ACTION_SEQUENCES: &[ActionSequence] = ALL_PROGRESS_ACTIONS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    cp: CP,
    muscle_memory: u8,
    veneration: u8,
}

impl std::convert::From<InProgress> for ReducedState {
    fn from(state: InProgress) -> Self {
        let durability_cost = DURABILITY_COST * (MAX_DURABILITY - state.durability) as CP / 5;
        Self {
            cp: state.cp - durability_cost,
            muscle_memory: state.effects.muscle_memory,
            veneration: state.effects.veneration,
        }
    }
}

impl std::convert::From<ReducedState> for InProgress {
    fn from(state: ReducedState) -> Self {
        Self {
            cp: state.cp,
            durability: MAX_DURABILITY,
            missing_progress: MAX_PROGRESS,
            missing_quality: MAX_QUALITY,
            effects: Effects {
                muscle_memory: state.muscle_memory,
                veneration: state.veneration,
                ..Default::default()
            },
            combo: None,
        }
    }
}

pub struct ProgressBoundSolver {
    settings: Settings,
    progress_upper_bound: HashMap<ReducedState, Progress>,
    cp_lower_bound: HashMap<(Progress, u8, u8), CP>,
}

impl ProgressBoundSolver {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            progress_upper_bound: HashMap::default(),
            cp_lower_bound: HashMap::default(),
        }
    }

    pub fn get_progress_upper_bound(
        &mut self,
        cp: CP,
        muscle_memory: u8,
        veneration: u8,
    ) -> Progress {
        self._get_progress_upper_bound(ReducedState {
            cp,
            muscle_memory,
            veneration,
        })
    }

    pub fn get_cp_lower_bound(
        &mut self,
        missing_progress: Progress,
        muscle_memory: u8,
        veneration: u8,
    ) -> CP {
        let key = (missing_progress, muscle_memory, veneration);
        match self.cp_lower_bound.get(&key) {
            Some(cp) => *cp,
            None => {
                let result =
                    self._solve_cp_lower_bound(missing_progress, muscle_memory, veneration);
                self.cp_lower_bound.insert(key, result);
                result
            }
        }
    }

    fn _solve_cp_lower_bound(
        &mut self,
        missing_progress: Progress,
        muscle_memory: u8,
        veneration: u8,
    ) -> CP {
        let mut base: CP = 1;
        while self.get_progress_upper_bound(2 * base, muscle_memory, veneration) < missing_progress
        {
            base *= 2;
        }
        let mut result: CP = 0;
        while base != 0 {
            if self.get_progress_upper_bound(result + base, muscle_memory, veneration)
                < missing_progress
            {
                result += base;
            }
            base /= 2;
        }
        result + 1
    }

    fn _get_progress_upper_bound(&mut self, state: ReducedState) -> Progress {
        match self.progress_upper_bound.get(&state) {
            Some(progress) => *progress,
            None => {
                let result = self._solve_progress_upper_bound(state);
                self.progress_upper_bound.insert(state, result);
                result
            }
        }
    }

    fn _solve_progress_upper_bound(&mut self, state: ReducedState) -> Progress {
        let mut best_progress = Progress::new(0);
        for action_sequence in ACTION_SEQUENCES {
            for use_waste_not in [false, true] {
                let mut full_state = State::InProgress(state.into());
                if use_waste_not {
                    full_state = apply_waste_not(full_state, action_sequence.len() as u8);
                }
                full_state =
                    full_state.use_actions(action_sequence, Condition::Normal, &self.settings);
                if let State::InProgress(in_progress) = full_state {
                    let action_progress = MAX_PROGRESS.saturating_sub(in_progress.missing_progress);
                    let total_progress = action_progress.saturating_add(
                        self._get_progress_upper_bound(ReducedState::from(in_progress)),
                    );
                    best_progress = std::cmp::max(best_progress, total_progress);
                }
            }
        }
        best_progress
    }
}

impl Drop for ProgressBoundSolver {
    fn drop(&mut self) {
        let states = self.progress_upper_bound.len();
        dbg!(states);
    }
}

fn apply_waste_not(state: State, duration: u8) -> State {
    match state {
        State::InProgress(mut in_progress) => {
            let effect_cost: CP = duration as CP * WASTE_NOT_COST;
            if effect_cost > in_progress.cp {
                return State::Invalid;
            }
            in_progress.cp -= effect_cost;
            in_progress.effects.waste_not = duration;
            State::InProgress(in_progress)
        }
        _ => State::Invalid,
    }
}
