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
        #[cfg(test)]
        assert_eq!(state.combo, None);
        let used_durability = (INF_DURABILITY - state.durability) / 5;
        Self {
            cp: state.cp - used_durability as CP * DURABILITY_COST,
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
        let current_quality = self
            .settings
            .max_quality
            .saturating_sub(state.missing_quality);
        state.durability += INF_DURABILITY;
        state.cp += state.effects.manipulation as CP * MANIPULATION_COST;
        let reduced_state = ReducedState::from(state);

        if !self.solved_states.contains_key(&reduced_state) {
            self.solve_state(reduced_state);
            self.pareto_front_builder.clear();
        }
        let pareto_front = self.solved_states.get(&reduced_state).unwrap();

        match pareto_front.first() {
            Some(first) => {
                if first.progress < state.missing_progress {
                    return Quality::new(0);
                }
            }
            None => return Quality::new(0),
        }

        let mut lo = 0;
        let mut hi = pareto_front.len();
        while lo + 1 != hi {
            let m = (lo + hi) / 2;
            if pareto_front[m].progress < state.missing_progress {
                hi = m;
            } else {
                lo = m;
            }
        }

        std::cmp::min(
            self.settings.max_quality,
            pareto_front[lo].quality.saturating_add(current_quality),
        )
    }

    fn solve_state(&mut self, state: ReducedState) {
        self.pareto_front_builder.push_empty();
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
                            Some(pareto_front) => {
                                self.pareto_front_builder.push(pareto_front.as_ref())
                            }
                            None => self.solve_state(new_state),
                        }
                        self.pareto_front_builder
                            .add(action_progress, action_quality);
                        self.pareto_front_builder.merge();
                    } else if action_progress != Progress::new(0) {
                        // last action must be a progress increase
                        self.pareto_front_builder
                            .push(&[ParetoValue::new(Progress::new(0), Quality::new(0))]);
                        self.pareto_front_builder
                            .add(action_progress, action_quality);
                        self.pareto_front_builder.merge();
                    }
                }
                State::Invalid => (),
                _ => panic!(),
            }
        }
        self.solved_states
            .insert(state, self.pareto_front_builder.peek().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts::*;

    fn solve(settings: Settings, actions: &[Action]) -> f32 {
        let state = State::new(&settings).use_actions(actions, Condition::Normal, &settings);
        let result = UpperBoundSolver::new(settings)
            .quality_upper_bound(state.as_in_progress().unwrap())
            .into();
        dbg!(result);
        result
    }

    #[test]
    fn test_01() {
        let settings = Settings {
            max_cp: 553,
            max_durability: 70,
            max_progress: Progress::from(2400.00),
            max_quality: Quality::from(20000.00),
        };
        let result = solve(
            settings,
            &[
                Action::MuscleMemory,
                Action::PrudentTouch,
                Action::Manipulation,
                Action::Veneration,
                Action::WasteNot2,
                Action::Groundwork,
                Action::Groundwork,
                Action::Groundwork,
                Action::PreparatoryTouch,
            ],
        );
        assert_eq!(result, 3455.00); // tightness test
        assert_ge!(result, 3352.50); // correctness test
    }

    #[test]
    fn test_02() {
        let settings = Settings {
            max_cp: 700,
            max_durability: 70,
            max_progress: Progress::from(2500.00),
            max_quality: Quality::from(5000.00),
        };
        let result = solve(
            settings,
            &[
                Action::MuscleMemory,
                Action::Manipulation,
                Action::Veneration,
                Action::WasteNot,
                Action::Groundwork,
                Action::Groundwork,
            ],
        );
        assert_eq!(result, 4797.50); // tightness test
        assert_ge!(result, 4685.50); // correctness test
    }

    #[test]
    fn test_03() {
        let settings = Settings {
            max_cp: 617,
            max_durability: 60,
            max_progress: Progress::from(2120.00),
            max_quality: Quality::from(5000.00),
        };
        let result = solve(
            settings,
            &[
                Action::MuscleMemory,
                Action::Manipulation,
                Action::Veneration,
                Action::Groundwork,
                Action::CarefulSynthesis,
                Action::PrudentSynthesis,
                Action::PrudentSynthesis,
                Action::Innovation,
                Action::PrudentTouch,
                Action::PrudentTouch,
            ],
        );
        assert_eq!(result, 4103.75); // tightness test
        assert_ge!(result, 4016.25); // correctness test
    }

    #[test]
    fn test_04() {
        let settings = Settings {
            max_cp: 411,
            max_durability: 60,
            max_progress: Progress::from(1990.00),
            max_quality: Quality::from(5000.00),
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2335.00); // tightness test
        assert_ge!(result, 2005.00); // correctness test
    }

    #[test]
    fn test_05() {
        let settings = Settings {
            max_cp: 450,
            max_durability: 60,
            max_progress: Progress::from(1970.00),
            max_quality: Quality::from(2000.00),
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2000.00); // tightness test
        assert_ge!(result, 2000.00); // correctness test
    }

    #[test]
    fn test_06() {
        let settings = Settings {
            max_cp: 673,
            max_durability: 60,
            max_progress: Progress::from(2345.00),
            max_quality: Quality::from(8000.00),
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 4455.00); // tightness test
        assert_ge!(result, 4340.00); // correctness test
    }

    #[test]
    fn test_07() {
        let settings = Settings {
            max_cp: 673,
            max_durability: 60,
            max_progress: Progress::from(2345.00),
            max_quality: Quality::from(8000.00),
        };
        let result = solve(settings, &[Action::Reflect]);
        assert_eq!(result, 4322.50); // tightness test
        assert_ge!(result, 4135.00); // correctness test
    }
}
