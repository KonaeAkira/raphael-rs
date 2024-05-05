use crate::{
    game::{state::InProgress, units::*, Action, Condition, Effects, Settings, State},
    solvers::action_sequences::{MIXED_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS},
};

use constcat::concat_slices;
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

struct ParetoFront {
    values: Vec<(Progress, Quality)>,
}

impl ParetoFront {
    pub fn new() -> Self {
        Self {
            values: vec![(Progress::new(0), Quality::new(0))],
        }
    }

    pub fn get(&self, progress: Progress) -> Quality {
        let mut lo = 0;
        let mut hi = self.values.len();
        while lo + 1 < hi {
            if self.values[(lo + hi) / 2].0 < progress {
                hi = (lo + hi) / 2;
            } else {
                lo = (lo + hi) / 2;
            }
        }
        self.values[lo].1
    }

    fn _try_push(&mut self, new_value: (Progress, Quality)) {
        match self.values.last_mut() {
            Some(value) => {
                if value.0 == new_value.0 {
                    value.1 = std::cmp::max(value.1, new_value.1);
                } else {
                    assert!(value.0 > new_value.0);
                    if value.1 < new_value.1 {
                        self.values.push(new_value);
                    }
                }
            }
            None => self.values.push(new_value),
        }
    }

    pub fn merge<I: Iterator<Item = (Progress, Quality)>>(self, values: I) -> Self {
        let mut new_front = Self { values: Vec::new() };

        let mut i = self.values.into_iter().peekable();
        let mut j = values.peekable();

        let mut i_opt = i.next();
        let mut j_opt = j.next();

        loop {
            match (i_opt, j_opt) {
                (None, None) => break,
                (None, Some(j_value)) => {
                    new_front._try_push(j_value);
                    j_opt = j.next();
                }
                (Some(i_value), None) => {
                    new_front._try_push(i_value);
                    i_opt = i.next();
                }
                (Some(i_value), Some(j_value)) => {
                    if i_value.0 > j_value.0 {
                        new_front._try_push(i_value);
                        i_opt = i.next();
                    } else {
                        new_front._try_push(j_value);
                        j_opt = j.next();
                    }
                }
            }
        }

        if new_front.values.last().unwrap().0 == Progress::new(0) {
            new_front.values.pop();
        }
        new_front
    }
}

pub struct UpperBoundSolver {
    settings: Settings,
    memory: HashMap<ReducedState, ParetoFront>,
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
        self.memory
            .get(&ReducedState::from(state))
            .unwrap()
            .get(state.missing_progress)
            .saturating_add(current_quality)
    }

    fn _solve(&mut self, state: ReducedState) {
        if self.memory.contains_key(&state) {
            return;
        }
        let mut current_front: ParetoFront = ParetoFront::new();
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
                    current_front = current_front.merge(child_front.values.iter().map(
                        |(progress, quality)| {
                            (
                                std::cmp::min(
                                    self.settings.max_progress,
                                    action_progress.saturating_add(*progress),
                                ),
                                std::cmp::min(
                                    self.settings.max_quality,
                                    action_quality.saturating_add(*quality),
                                ),
                            )
                        },
                    ));
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
            pareto_values += pareto_front.values.len();
        }
        dbg!(states, pareto_values);
    }
}
