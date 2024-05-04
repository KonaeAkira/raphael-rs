use rustc_hash::FxHashMap as HashMap;

use crate::game::{
    state::InProgress,
    units::{Durability, Progress, Quality, CP},
    Action, Condition, Effects, Settings, State,
};

use super::constants::{DURABILITY_COST, VENERATION_COST, WASTE_NOT_COST};

const MAX_DURABILITY: Durability = 100;
const MAX_PROGRESS: Progress = Progress::new(100_000);
const MAX_QUALITY: Quality = Quality::new(100_000);

const ACTION_SEQUENCES: [&[Action]; 4] = [
    &[Action::BasicSynthesis],
    &[Action::CarefulSynthesis],
    &[Action::Groundwork],
    &[Action::Observe, Action::FocusedSynthesis],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    cp: CP,
}

impl std::convert::From<InProgress> for ReducedState {
    fn from(state: InProgress) -> Self {
        let durability_cost = DURABILITY_COST * (MAX_DURABILITY - state.durability) as CP / 5;
        Self {
            cp: state.cp - durability_cost,
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
            effects: Effects::default(),
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

    pub fn progress_upper_bound(&mut self, cp: CP) -> Progress {
        self._get_bound(ReducedState { cp })
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
        for veneration in 0..=2 {
            for waste_not in 0..=2 {
                let effect_cost: CP = veneration * VENERATION_COST + waste_not * WASTE_NOT_COST;
                if effect_cost > state.cp {
                    continue;
                }
                let mut full_state = InProgress::from(state);
                full_state.cp -= effect_cost;
                full_state.effects.veneration = veneration as u8;
                full_state.effects.waste_not = waste_not as u8;
                for action_sequence in ACTION_SEQUENCES {
                    let new_state = State::InProgress(full_state).use_actions(
                        action_sequence,
                        Condition::Normal,
                        &self.settings,
                    );
                    if let State::InProgress(in_progress) = new_state {
                        let action_progress =
                            MAX_PROGRESS.saturating_sub(in_progress.missing_progress);
                        let total_progress = action_progress
                            .saturating_add(self._get_bound(ReducedState::from(in_progress)));
                        best_progress = std::cmp::max(best_progress, total_progress);
                    }
                }
            }
        }
        best_progress
    }
}

impl Drop for ProgressBoundSolver {
    fn drop(&mut self) {
        let states = self.progress_bound.len();
        dbg!(states);
    }
}