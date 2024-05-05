use crate::{
    game::{state::InProgress, units::*, Action, Condition, Effects, Settings, State},
    solvers::action_sequences::{MIXED_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS},
};

use constcat::concat_slices;
use pareto_front::{Dominate, ParetoFront};
use rustc_hash::FxHashMap as HashMap;

use super::action_sequences::ActionSequence;

const ACTION_SEQUENCES: &[ActionSequence] = concat_slices!([ActionSequence]:
    PROGRESS_ACTIONS, QUALITY_ACTIONS, MIXED_ACTIONS, &[&[Action::WasteNot, Action::WasteNot2]]);

const INF_PROGRESS: Progress = Progress::new(100_000);
const INF_QUALITY: Quality = Quality::new(100_000);
const INF_DURABILITY: Durability = 100;

pub const MANIPULATION_COST: CP = Action::Manipulation.base_cp_cost() / 8;
// CP value for 5 Durability
pub const DURABILITY_COST: CP = Action::Manipulation.base_cp_cost() / 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    cp: CP,
    effects: Effects,
}

impl std::convert::From<InProgress> for ReducedState {
    fn from(state: InProgress) -> Self {
        let used_durability = (INF_DURABILITY - state.durability) / 5;
        Self {
            cp: state.cp - used_durability as CP * DURABILITY_COST
                + state.effects.manipulation as CP * MANIPULATION_COST,
            effects: Effects {
                manipulation: 0,
                ..state.effects
            },
        }
    }
}

impl std::convert::From<ReducedState> for InProgress {
    fn from(state: ReducedState) -> Self {
        Self {
            durability: INF_DURABILITY,
            cp: state.cp,
            missing_progress: INF_PROGRESS,
            missing_quality: INF_QUALITY,
            effects: state.effects,
            combo: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParetoValue {
    progress: Progress,
    quality: Quality,
}

impl Dominate for ParetoValue {
    fn dominate(&self, other: &Self) -> bool {
        self.progress >= other.progress && self.quality >= other.quality
    }
}

pub struct UpperBoundSolver {
    settings: Settings,
    memory: HashMap<ReducedState, ParetoFront<ParetoValue>>,
}

impl UpperBoundSolver {
    pub fn new(settings: Settings) -> Self {
        UpperBoundSolver {
            settings,
            memory: HashMap::default(),
        }
    }

    pub fn quality_upper_bound(&mut self, mut state: InProgress) -> Quality {
        state.durability += INF_DURABILITY;
        let current_quality = self
            .settings
            .max_quality
            .saturating_sub(state.missing_quality);
        self._solve(ReducedState::from(state));
        let pareto_front = self.memory.get(&ReducedState::from(state)).unwrap();
        let mut best_quality = Quality::new(0);
        for value in pareto_front.iter() {
            if value.progress >= state.missing_progress {
                best_quality = std::cmp::max(best_quality, value.quality);
            }
        }
        current_quality.saturating_add(best_quality)
    }

    fn _solve(&mut self, state: ReducedState) {
        if self.memory.contains_key(&state) {
            return;
        }
        let mut current_front: ParetoFront<ParetoValue> = ParetoFront::new();
        current_front.push(ParetoValue {
            progress: Progress::new(0),
            quality: Quality::new(0),
        });
        for actions in ACTION_SEQUENCES {
            let new_state = State::InProgress(state.into()).use_actions(
                actions,
                Condition::Normal,
                &self.settings,
            );
            match new_state {
                State::InProgress(new_state) => {
                    let action_progress = INF_PROGRESS.saturating_sub(new_state.missing_progress);
                    let action_quality = INF_QUALITY.saturating_sub(new_state.missing_quality);
                    self._solve(ReducedState::from(new_state));
                    let child_front = self.memory.get(&ReducedState::from(new_state)).unwrap();
                    for value in child_front.iter() {
                        current_front.push(ParetoValue {
                            progress: std::cmp::min(
                                self.settings.max_progress,
                                action_progress.saturating_add(value.progress),
                            ),
                            quality: std::cmp::min(
                                self.settings.max_quality,
                                action_quality.saturating_add(value.quality),
                            ),
                        });
                    }
                }
                State::Invalid => (),
                _ => panic!(),
            }
        }
        // dbg!(&state, &current_front);
        self.memory.insert(state, current_front);
    }
}

impl Drop for UpperBoundSolver {
    fn drop(&mut self) {
        let states = self.memory.len();
        let mut pareto_values: usize = 0;
        for (_, pareto_front) in self.memory.iter() {
            pareto_values += pareto_front.len();
        }
        dbg!(states, pareto_values);
    }
}
