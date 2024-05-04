use rustc_hash::FxHashMap as HashMap;

use crate::game::{
    state::InProgress,
    units::{Durability, Progress, Quality, CP},
    Action, Condition, Effects, Settings, State,
};

use super::constants::{DURABILITY_COST, WASTE_NOT_COST};

const MAX_DURABILITY: Durability = 100;
const MAX_PROGRESS: Progress = Progress::new(100_000);
const MAX_QUALITY: Quality = Quality::new(100_000);

const ACTION_SEQUENCES: [&[Action]; 6] = [
    &[Action::BasicSynthesis],
    &[Action::CarefulSynthesis],
    &[Action::Groundwork],
    &[Action::PrudentSynthesis],
    &[Action::Observe, Action::FocusedSynthesis],
    &[Action::Veneration],
];

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
    progress_bound: HashMap<ReducedState, Progress>,
}

impl ProgressBoundSolver {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            progress_bound: HashMap::default(),
        }
    }

    pub fn progress_upper_bound(&mut self, cp: CP, muscle_memory: u8, veneration: u8) -> Progress {
        self._get_bound(ReducedState {
            cp,
            muscle_memory,
            veneration,
        })
    }

    fn _get_bound(&mut self, state: ReducedState) -> Progress {
        match self.progress_bound.get(&state) {
            Some(progress) => *progress,
            None => {
                let bound = self._solve_bound(state);
                self.progress_bound.insert(state, bound);
                bound
            }
        }
    }

    fn _solve_bound(&mut self, state: ReducedState) -> Progress {
        let mut best_progress = Progress::new(0);
        for action_sequence in ACTION_SEQUENCES {
            for use_waste_not in [false, true] {
                let mut full_state = State::InProgress(state.into());
                if use_waste_not {
                    full_state = Self::_apply_waste_not(full_state, action_sequence.len() as u8);
                }
                full_state =
                    full_state.use_actions(action_sequence, Condition::Normal, &self.settings);
                if let State::InProgress(in_progress) = full_state {
                    let action_progress = MAX_PROGRESS.saturating_sub(in_progress.missing_progress);
                    let total_progress = action_progress
                        .saturating_add(self._get_bound(ReducedState::from(in_progress)));
                    best_progress = std::cmp::max(best_progress, total_progress);
                }
            }
        }
        best_progress
    }

    fn _apply_waste_not(state: State, duration: u8) -> State {
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
}

impl Drop for ProgressBoundSolver {
    fn drop(&mut self) {
        let states = self.progress_bound.len();
        dbg!(states);
    }
}
