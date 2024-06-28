use crate::actions::{MIXED_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS};
use simulator::{
    state::InProgress, Action, ActionMask, ComboAction, Condition, Effects, Settings,
    SimulationState, SingleUse,
};

use rustc_hash::FxHashMap as HashMap;

use super::pareto_front::{ParetoFrontBuilder, ParetoValue};

const SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS.union(QUALITY_ACTIONS).union(MIXED_ACTIONS);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct ReducedEffects {
    inner_quiet: u8,
    innovation: u8,
    veneration: u8,
    great_strides: u8,
    muscle_memory: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ReducedState {
    cp: i16,
    combo: Option<ComboAction>,
    effects: ReducedEffects,
}

impl ReducedState {
    fn from_state(state: InProgress, base_durability_cost: i16, waste_not_cost: i16) -> Self {
        let state = *state.raw_state();
        let used_durability = (i8::MAX - state.durability) / 5;
        let durability_cost = std::cmp::min(
            used_durability as i16 * base_durability_cost,
            (used_durability as i16 + 1) / 2 * base_durability_cost + waste_not_cost,
        );
        Self {
            cp: state.cp - durability_cost,
            combo: state.combo,
            effects: ReducedEffects {
                inner_quiet: state.effects.inner_quiet(),
                innovation: state.effects.innovation(),
                veneration: state.effects.veneration(),
                great_strides: state.effects.great_strides(),
                muscle_memory: state.effects.muscle_memory(),
            },
        }
    }
}

impl std::convert::From<ReducedState> for InProgress {
    fn from(state: ReducedState) -> Self {
        SimulationState {
            durability: i8::MAX,
            cp: state.cp,
            missing_progress: u16::MAX,
            missing_quality: u16::MAX,
            effects: Effects::new()
                .with_inner_quiet(state.effects.inner_quiet)
                .with_innovation(state.effects.innovation)
                .with_veneration(state.effects.veneration)
                .with_great_strides(state.effects.great_strides)
                .with_muscle_memory(state.effects.muscle_memory),
            combo: state.combo,
        }
        .try_into()
        .unwrap()
    }
}

pub struct UpperBoundSolver {
    settings: Settings,
    base_durability_cost: i16,
    waste_not_cost: i16,
    solved_states: HashMap<ReducedState, Box<[ParetoValue<u16, u16>]>>,
    pareto_front_builder: ParetoFrontBuilder<u16, u16>,
}

impl UpperBoundSolver {
    pub fn new(settings: Settings) -> Self {
        dbg!(std::mem::size_of::<ReducedState>());
        dbg!(std::mem::align_of::<ReducedState>());
        let mut durability_cost = Action::MasterMend.base_cp_cost() / 6;
        if settings.allowed_actions.has(Action::Manipulation) {
            durability_cost =
                std::cmp::min(durability_cost, Action::Manipulation.base_cp_cost() / 8);
        }
        if settings.allowed_actions.has(Action::ImmaculateMend) {
            durability_cost = std::cmp::min(
                durability_cost,
                Action::ImmaculateMend.base_cp_cost() / (settings.max_durability as i16 / 5 - 1),
            );
        }
        UpperBoundSolver {
            settings,
            base_durability_cost: durability_cost,
            waste_not_cost: if settings.allowed_actions.has(Action::WasteNot2) {
                Action::WasteNot2.base_cp_cost() / 8
            } else if settings.allowed_actions.has(Action::WasteNot) {
                Action::WasteNot.base_cp_cost() / 4
            } else {
                1000 // inf
            },
            solved_states: HashMap::default(),
            pareto_front_builder: ParetoFrontBuilder::new(settings.max_progress),
        }
    }

    /// Returns an upper-bound on the maximum Quality achievable from this state while also maxing out Progress.
    /// The returned upper-bound is NOT clamped to settings.max_quality.
    /// There is no guarantee on the tightness of the upper-bound.
    pub fn quality_upper_bound(&mut self, state: InProgress) -> u16 {
        let mut state = *state.raw_state();
        let current_quality = self.settings.max_quality - state.missing_quality;

        // refund effects and durability
        state.cp += state.effects.manipulation() as i16 * (Action::Manipulation.base_cp_cost() / 8);
        state.cp += state.effects.waste_not() as i16 * self.waste_not_cost;
        state.cp += state.durability as i16 / 5 * self.base_durability_cost;
        state.durability = i8::MAX;

        // assume Trained Perfection can be used to its fullest potential (20 durability)
        if self.settings.allowed_actions.has(Action::TrainedPerfection)
            && matches!(
                state.effects.trained_perfection(),
                SingleUse::Available | SingleUse::Active
            )
        {
            state.cp += self.base_durability_cost * 4;
        }

        let reduced_state = ReducedState::from_state(
            InProgress::try_from(state).unwrap(),
            self.base_durability_cost,
            self.waste_not_cost,
        );

        if !self.solved_states.contains_key(&reduced_state) {
            self.solve_state(reduced_state);
            self.pareto_front_builder.clear();
        }
        let pareto_front = self.solved_states.get(&reduced_state).unwrap();

        match pareto_front.first() {
            Some(first_element) => {
                if first_element.first < state.missing_progress {
                    return 0;
                }
            }
            None => return 0,
        }

        let mut lo = 0;
        let mut hi = pareto_front.len();
        while lo + 1 != hi {
            let m = (lo + hi) / 2;
            if pareto_front[m].first < state.missing_progress {
                hi = m;
            } else {
                lo = m;
            }
        }

        pareto_front[lo].second + current_quality
    }

    fn solve_state(&mut self, state: ReducedState) {
        match state.combo {
            Some(combo) => {
                let non_combo_state = ReducedState {
                    combo: None,
                    ..state
                };
                match self.solved_states.get(&non_combo_state) {
                    Some(pareto_front) => self.pareto_front_builder.push(pareto_front),
                    None => self.solve_non_combo_state(non_combo_state),
                }
                let combo_actions: &[Action] = match combo {
                    ComboAction::SynthesisBegin => &[Action::MuscleMemory, Action::Reflect],
                    ComboAction::BasicTouch => {
                        &[Action::ComboStandardTouch, Action::ComboRefinedTouch]
                    }
                    ComboAction::StandardTouch => &[Action::ComboAdvancedTouch],
                    ComboAction::Observe => &[Action::ComboAdvancedTouch],
                };
                for action in combo_actions {
                    if self.settings.allowed_actions.has(*action) {
                        self.build_child_front(state, *action);
                    }
                }
            }
            None => {
                self.solve_non_combo_state(state);
            }
        }
        let pareto_front = self.pareto_front_builder.peek().unwrap();
        self.solved_states.insert(state, pareto_front);
    }

    fn solve_non_combo_state(&mut self, state: ReducedState) {
        self.pareto_front_builder.push_empty();
        for action in SEARCH_ACTIONS
            .intersection(self.settings.allowed_actions)
            .actions_iter()
        {
            self.build_child_front(state, action);
        }
    }

    fn build_child_front(&mut self, state: ReducedState, action: Action) {
        if let Ok(new_state) =
            InProgress::from(state).use_action(action, Condition::Normal, &self.settings)
        {
            if let Ok(in_progress) = InProgress::try_from(new_state) {
                let action_progress = u16::MAX - new_state.missing_progress;
                let action_quality = u16::MAX - new_state.missing_quality;
                let new_state = ReducedState::from_state(
                    in_progress,
                    self.base_durability_cost,
                    self.waste_not_cost,
                );
                if new_state.cp > 0 {
                    match self.solved_states.get(&new_state) {
                        Some(pareto_front) => self.pareto_front_builder.push(pareto_front),
                        None => self.solve_state(new_state),
                    }
                    self.pareto_front_builder
                        .add(action_progress, action_quality);
                    self.pareto_front_builder.merge();
                } else if new_state.cp + self.base_durability_cost >= 0 && action_progress != 0 {
                    // "durability" must not go lower than -5
                    // last action must be a progress increase
                    self.pareto_front_builder.push(&[ParetoValue::new(0, 0)]);
                    self.pareto_front_builder
                        .add(action_progress, action_quality);
                    self.pareto_front_builder.merge();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solve(settings: Settings, actions: &[Action]) -> u16 {
        let state = SimulationState::from_macro(&settings, actions).unwrap();
        let result = UpperBoundSolver::new(settings).quality_upper_bound(state.try_into().unwrap());
        dbg!(result);
        result
    }

    #[test]
    fn test_01() {
        let settings = Settings {
            max_cp: 553,
            max_durability: 70,
            max_progress: 2400,
            max_quality: 20000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true),
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
        assert_eq!(result, 3485);
    }

    #[test]
    fn test_02() {
        let settings = Settings {
            max_cp: 700,
            max_durability: 70,
            max_progress: 2500,
            max_quality: 5000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true),
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
        assert_eq!(result, 4767);
    }

    #[test]
    fn test_03() {
        let settings = Settings {
            max_cp: 617,
            max_durability: 60,
            max_progress: 2120,
            max_quality: 5000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true),
        };
        let result = solve(
            settings,
            &[
                Action::MuscleMemory,
                Action::Manipulation,
                Action::Veneration,
                Action::WasteNot,
                Action::Groundwork,
                Action::CarefulSynthesis,
                Action::Groundwork,
                Action::PreparatoryTouch,
                Action::Innovation,
                Action::BasicTouch,
                Action::ComboStandardTouch,
            ],
        );
        assert_eq!(result, 4053);
    }

    #[test]
    fn test_04() {
        let settings = Settings {
            max_cp: 411,
            max_durability: 60,
            max_progress: 1990,
            max_quality: 5000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true),
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2220);
    }

    #[test]
    fn test_05() {
        let settings = Settings {
            max_cp: 450,
            max_durability: 60,
            max_progress: 1970,
            max_quality: 2000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true),
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2604);
    }

    #[test]
    fn test_06() {
        let settings = Settings {
            max_cp: 673,
            max_durability: 60,
            max_progress: 2345,
            max_quality: 8000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true),
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 4555); // tightness test
    }

    #[test]
    fn test_07() {
        let settings = Settings {
            max_cp: 673,
            max_durability: 60,
            max_progress: 2345,
            max_quality: 8000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true),
        };
        let result = solve(settings, &[Action::Reflect]);
        assert_eq!(result, 4633); // tightness test
    }

    #[test]
    fn test_08() {
        let settings = Settings {
            max_cp: 32,
            max_durability: 10,
            max_progress: 10000,
            max_quality: 20000,
            base_progress: 10000,
            base_quality: 10000,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true),
        };
        let result = solve(settings, &[Action::PrudentTouch]);
        assert_eq!(result, 10000); // tightness test
    }

    #[test]
    fn test_09() {
        let settings = Settings {
            max_cp: 700,
            max_durability: 70,
            max_progress: 2500,
            max_quality: 40000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, false),
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 4823); // tightness test
    }
}
