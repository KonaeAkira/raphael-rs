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

const ACTION_SEQUENCES: [&[Action]; 7] = [
    &[Action::PrudentTouch],
    &[
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
    ],
    &[Action::PreparatoryTouch],
    &[Action::Observe, Action::FocusedTouch],
    &[Action::ByregotsBlessing],
    &[Action::Innovation],
    &[Action::GreatStrides],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    cp: CP,
    inner_quiet: u8,
    innovation: u8,
    great_strides: u8,
}

impl std::convert::From<InProgress> for ReducedState {
    fn from(state: InProgress) -> Self {
        let durability_cost = DURABILITY_COST * (MAX_DURABILITY - state.durability) as CP / 5;
        Self {
            cp: state.cp - durability_cost,
            inner_quiet: state.effects.inner_quiet,
            innovation: state.effects.innovation,
            great_strides: state.effects.great_strides,
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
                inner_quiet: state.inner_quiet,
                innovation: state.innovation,
                great_strides: state.great_strides,
                ..Default::default()
            },
            combo: None,
        }
    }
}

pub struct QualityBoundSolver {
    settings: Settings,
    quality_bound: HashMap<ReducedState, Quality>,
}

impl QualityBoundSolver {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            quality_bound: HashMap::default(),
        }
    }

    pub fn quality_upper_bound(
        &mut self,
        cp: CP,
        inner_quiet: u8,
        innovation: u8,
        great_strides: u8,
    ) -> Quality {
        self._get_bound(ReducedState {
            cp,
            inner_quiet,
            innovation,
            great_strides,
        })
    }

    fn _get_bound(&mut self, state: ReducedState) -> Quality {
        match self.quality_bound.get(&state) {
            Some(quality) => *quality,
            None => {
                let bound = self._solve_bound(state);
                self.quality_bound.insert(state, bound);
                bound
            }
        }
    }

    fn _solve_bound(&mut self, state: ReducedState) -> Quality {
        let mut best_quality = Quality::new(0);
        for action_sequence in ACTION_SEQUENCES {
            for use_waste_not in [false, true] {
                let mut full_state = State::InProgress(state.into());
                if use_waste_not {
                    full_state = Self::_apply_waste_not(full_state, action_sequence.len() as u8);
                }
                full_state =
                    full_state.use_actions(action_sequence, Condition::Normal, &self.settings);
                if let State::InProgress(in_progress) = full_state {
                    let new_state = ReducedState::from(in_progress);
                    if new_state.cp < 0 {
                        continue;
                    }
                    let action_quality = MAX_QUALITY.saturating_sub(in_progress.missing_quality);
                    let total_quality = action_quality
                        .saturating_add(self._get_bound(new_state));
                    best_quality = std::cmp::max(best_quality, total_quality);
                }
            }
        }
        best_quality
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

impl Drop for QualityBoundSolver {
    fn drop(&mut self) {
        let states = self.quality_bound.len();
        dbg!(states);
    }
}
