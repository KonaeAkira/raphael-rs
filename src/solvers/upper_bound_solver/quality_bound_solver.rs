use rustc_hash::FxHashMap as HashMap;

use crate::game::{
    state::InProgress,
    units::{Durability, Progress, Quality, CP},
    Action, Condition, Effects, Settings, State,
};

use super::constants::{DURABILITY_COST, GREAT_STRIDES_COST, VENERATION_COST, WASTE_NOT_COST};

const MAX_DURABILITY: Durability = 100;
const MAX_PROGRESS: Progress = Progress::new(100_000);
const MAX_QUALITY: Quality = Quality::new(100_000);

const ACTION_SEQUENCES: [&[Action]; 5] = [
    &[Action::PrudentTouch],
    &[
        Action::BasicTouch,
        Action::StandardTouch,
        Action::AdvancedTouch,
    ],
    &[Action::PreparatoryTouch],
    &[Action::Observe, Action::FocusedTouch],
    &[Action::ByregotsBlessing],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    cp: CP,
    inner_quiet: u8,
}

impl std::convert::From<InProgress> for ReducedState {
    fn from(state: InProgress) -> Self {
        let durability_cost = DURABILITY_COST * (MAX_DURABILITY - state.durability) as CP / 5;
        Self {
            cp: state.cp - durability_cost,
            inner_quiet: state.effects.inner_quiet,
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

    pub fn quality_upper_bound(&mut self, cp: CP, inner_quiet: u8) -> Quality {
        self._get_bound(ReducedState { cp, inner_quiet })
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
        for great_strides in [false, true] {
            for innovation in 0..=2 {
                for waste_not in 0..=2 {
                    let effect_cost: CP = innovation * VENERATION_COST
                        + waste_not * WASTE_NOT_COST
                        + if great_strides { GREAT_STRIDES_COST } else { 0 };
                    if effect_cost > state.cp {
                        continue;
                    }
                    let mut full_state = InProgress::from(state);
                    full_state.cp -= effect_cost;
                    full_state.effects.great_strides = if great_strides { 5 } else { 0 };
                    full_state.effects.innovation = innovation as u8;
                    full_state.effects.waste_not = waste_not as u8;
                    for action_sequence in ACTION_SEQUENCES {
                        let new_state = State::InProgress(full_state).use_actions(
                            action_sequence,
                            Condition::Normal,
                            &self.settings,
                        );
                        if let State::InProgress(in_progress) = new_state {
                            let action_quality =
                                MAX_QUALITY.saturating_sub(in_progress.missing_quality);
                            let total_quality = action_quality
                                .saturating_add(self._get_bound(ReducedState::from(in_progress)));
                            best_quality = std::cmp::max(best_quality, total_quality);
                        }
                    }
                }
            }
        }
        best_quality
    }
}

impl Drop for QualityBoundSolver {
    fn drop(&mut self) {
        let states = self.quality_bound.len();
        dbg!(states);
    }
}
