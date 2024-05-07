use crate::{
    game::{state::InProgress, units::*, Action, Condition, Effects, Settings, State},
    solvers::action_sequences::{ActionSequence, MIXED_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS},
};

use constcat::concat_slices;
use rustc_hash::FxHashMap as HashMap;

use super::pareto_front::{ParetoFrontBuilder, ParetoValue};

const ACTION_SEQUENCES: &[ActionSequence] = concat_slices!([ActionSequence]:
    PROGRESS_ACTIONS, QUALITY_ACTIONS, MIXED_ACTIONS, &[&[Action::WasteNot, Action::WasteNot2]]);

const INF_PROGRESS: Progress = Progress::new(100_000);
const INF_QUALITY: Quality = Quality::new(100_000);
const INF_DURABILITY: Durability = 100;

pub const MANIPULATION_COST: CP = Action::Manipulation.base_cp_cost() / 8;
// CP value for 5 Durability
pub const DURABILITY_COST: CP = Action::Manipulation.base_cp_cost() / 8;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct ReducedEffects {
    pub inner_quiet: u8,
    pub waste_not: u8,
    pub innovation: u8,
    pub veneration: u8,
    pub great_strides: u8,
    pub muscle_memory: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    cp: CP,
    effects: ReducedEffects,
}

impl std::convert::From<InProgress> for ReducedState {
    fn from(state: InProgress) -> Self {
        let used_durability = (INF_DURABILITY - state.durability) / 5;
        Self {
            cp: state.cp - used_durability as CP * DURABILITY_COST
                + state.effects.manipulation as CP * MANIPULATION_COST,
            effects: ReducedEffects {
                inner_quiet: state.effects.inner_quiet,
                waste_not: state.effects.waste_not,
                innovation: state.effects.innovation,
                veneration: state.effects.veneration,
                great_strides: state.effects.great_strides,
                muscle_memory: state.effects.muscle_memory,
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
            effects: Effects {
                inner_quiet: state.effects.inner_quiet,
                waste_not: state.effects.waste_not,
                innovation: state.effects.innovation,
                veneration: state.effects.veneration,
                great_strides: state.effects.great_strides,
                muscle_memory: state.effects.muscle_memory,
                manipulation: 0,
            },
            combo: None,
        }
    }
}

pub struct UpperBoundSolver {
    settings: Settings,
    solved_states: HashMap<ReducedState, Box<[ParetoValue]>>,
    pareto_front_builder: ParetoFrontBuilder,
}

impl UpperBoundSolver {
    pub fn new(settings: Settings) -> Self {
        dbg!(std::mem::size_of::<ReducedState>());
        dbg!(std::mem::align_of::<ReducedState>());
        UpperBoundSolver {
            settings,
            solved_states: HashMap::default(),
            pareto_front_builder: ParetoFrontBuilder::new(settings),
        }
    }

    pub fn quality_upper_bound(&mut self, mut state: InProgress) -> Quality {
        state.durability += INF_DURABILITY;
        let current_quality = self
            .settings
            .max_quality
            .saturating_sub(state.missing_quality);
        let pareto_front = self.get_pareto_front(ReducedState::from(state));
        if pareto_front.is_empty() {
            return Quality::new(0);
        }
        let mut lo = 0;
        let mut hi = pareto_front.len();
        while lo + 1 < hi {
            if pareto_front[(lo + hi) / 2].progress < state.missing_progress {
                hi = (lo + hi) / 2;
            } else {
                lo = (lo + hi) / 2;
            }
        }
        pareto_front[lo].quality.saturating_add(current_quality)
    }

    fn get_pareto_front(&mut self, state: ReducedState) -> Box<[ParetoValue]> {
        match self.solved_states.get(&state) {
            Some(pareto_front) => pareto_front.clone(),
            None => {
                self.solve_state(state);
                let pareto_front = self.pareto_front_builder.finalize();
                self.pareto_front_builder.clear();
                pareto_front
            }
        }
    }

    fn solve_state(&mut self, state: ReducedState) {
        self.pareto_front_builder.start_new_front();
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
                    let new_state = ReducedState::from(new_state);
                    if new_state.cp > 0 {
                        match self.solved_states.get(&new_state) {
                            Some(pareto_front) => self
                                .pareto_front_builder
                                .import_front(pareto_front.as_ref()),
                            None => self.solve_state(new_state),
                        }
                    } else {
                        self.pareto_front_builder
                            .import_front(&[ParetoValue::new(Progress::new(0), Quality::new(0))]);
                    }
                    self.pareto_front_builder
                        .shift_last_front_value(action_progress, action_quality);
                    self.pareto_front_builder.merge_last_two_fronts();
                }
                State::Invalid => (),
                _ => panic!(),
            }
        }
        self.solved_states
            .insert(state, self.pareto_front_builder.finalize());
    }
}
