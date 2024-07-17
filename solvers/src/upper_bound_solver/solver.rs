use crate::{
    actions::{PROGRESS_ACTIONS, QUALITY_ACTIONS},
    utils::{ParetoFrontBuilder, ParetoValue},
};
use simulator::{state::InProgress, Action, ActionMask, Condition, Settings};

use rustc_hash::FxHashMap as HashMap;

use super::state::ReducedState;

const SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .add(Action::TrainedPerfection);

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
            } else {
                Action::WasteNot.base_cp_cost() / 4
            },
            solved_states: HashMap::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.max_progress,
                settings.max_quality,
            ),
        }
    }

    /// Returns an upper-bound on the maximum Quality achievable from this state while also maxing out Progress.
    /// The returned upper-bound is clamped to settings.max_quality.
    /// There is no guarantee on the tightness of the upper-bound.
    pub fn quality_upper_bound(&mut self, state: InProgress) -> u16 {
        let mut state = *state.raw_state();
        let current_quality = self.settings.max_quality - state.get_missing_quality();

        if current_quality == self.settings.max_quality {
            return current_quality;
        }

        // refund effects and durability
        state.cp += state.effects.manipulation() as i16 * (Action::Manipulation.base_cp_cost() / 8);
        state.cp += state.effects.waste_not() as i16 * self.waste_not_cost;
        state.cp += state.durability as i16 / 5 * self.base_durability_cost;
        state.durability = i8::MAX;

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

        std::cmp::min(
            self.settings.max_quality,
            pareto_front[lo].second.saturating_add(current_quality),
        )
    }

    fn solve_state(&mut self, state: ReducedState) {
        self.pareto_front_builder.push_empty();
        for action in SEARCH_ACTIONS
            .intersection(self.settings.allowed_actions)
            .actions_iter()
        {
            self.build_child_front(state, action);
            if self.pareto_front_builder.is_max() {
                // stop early if both Progress and Quality are maxed out
                // this optimization would work even better with better action ordering
                // (i.e. if better actions are visited first)
                break;
            }
        }
        let pareto_front = self.pareto_front_builder.peek().unwrap();
        self.solved_states.insert(state, pareto_front);
    }

    fn build_child_front(&mut self, state: ReducedState, action: Action) {
        if let Ok(new_state) =
            InProgress::from(state).use_action(action, Condition::Normal, &self.settings)
        {
            if let Ok(in_progress) = InProgress::try_from(new_state) {
                let action_progress = u16::MAX - new_state.missing_progress;
                let action_quality = u16::MAX - new_state.get_missing_quality();
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
                }
                if new_state.cp + self.base_durability_cost >= 0 && action_progress != 0 {
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
    use rand::Rng;
    use simulator::{Effects, SimulationState};

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
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: false,
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
    fn test_adversarial_01() {
        let settings = Settings {
            max_cp: 553,
            max_durability: 70,
            max_progress: 2400,
            max_quality: 20000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: true,
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
        assert_eq!(result, 3375);
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
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: false,
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
    fn test_adversarial_02() {
        let settings = Settings {
            max_cp: 700,
            max_durability: 70,
            max_progress: 2500,
            max_quality: 5000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: true,
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
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: false,
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
    fn test_adversarial_03() {
        let settings = Settings {
            max_cp: 617,
            max_durability: 60,
            max_progress: 2120,
            max_quality: 5000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: true,
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
        assert_eq!(result, 3953);
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
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: false,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2220);
    }

    #[test]
    fn test_adversarial_04() {
        let settings = Settings {
            max_cp: 411,
            max_durability: 60,
            max_progress: 1990,
            max_quality: 5000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: true,
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
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: false,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2000);
    }

    #[test]
    fn test_adversarial_05() {
        let settings = Settings {
            max_cp: 450,
            max_durability: 60,
            max_progress: 1970,
            max_quality: 2000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: true,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2000);
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
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: false,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 4555);
    }

    #[test]
    fn test_adversarial_06() {
        let settings = Settings {
            max_cp: 673,
            max_durability: 60,
            max_progress: 2345,
            max_quality: 8000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: true,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 4555);
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
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: false,
        };
        let result = solve(settings, &[Action::Reflect]);
        assert_eq!(result, 4633);
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
            allowed_actions: ActionMask::from_level(90, true, false),
            adversarial: false,
        };
        let result = solve(settings, &[Action::PrudentTouch]);
        assert_eq!(result, 10000);
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
            allowed_actions: ActionMask::from_level(90, false, false),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 4823);
    }

    #[test]
    fn test_10() {
        let settings = Settings {
            max_cp: 400,
            max_durability: 80,
            max_progress: 1200,
            max_quality: 24000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 100,
            allowed_actions: ActionMask::from_level(100, false, false),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 4269);
    }

    #[test]
    fn test_11() {
        let settings = Settings {
            max_cp: 320,
            max_durability: 80,
            max_progress: 1600,
            max_quality: 24000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 100,
            allowed_actions: ActionMask::from_level(100, false, false),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 2986);
    }

    #[test]
    fn test_12() {
        let settings = Settings {
            max_cp: 320,
            max_durability: 80,
            max_progress: 1600,
            max_quality: 24000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 100,
            allowed_actions: ActionMask::from_level(100, false, true),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 24000);
    }

    fn random_effects(adversarial: bool) -> Effects {
        Effects::default()
            .with_inner_quiet(rand::thread_rng().gen_range(0..=10))
            .with_great_strides(rand::thread_rng().gen_range(0..=3))
            .with_innovation(rand::thread_rng().gen_range(0..=4))
            .with_veneration(rand::thread_rng().gen_range(0..=4))
            .with_waste_not(rand::thread_rng().gen_range(0..=8))
            .with_manipulation(rand::thread_rng().gen_range(0..=8))
            .with_guard(if adversarial { rand::random() } else { false })
    }

    fn random_state(settings: &Settings) -> InProgress {
        SimulationState {
            cp: rand::thread_rng().gen_range(0..=settings.max_cp),
            durability: rand::thread_rng().gen_range(1..=(settings.max_durability / 5)) * 5,
            missing_progress: rand::thread_rng().gen_range(1..=settings.max_progress),
            unreliable_quality: [settings.max_quality; 2],
            effects: random_effects(settings.adversarial),
            combo: None, // TODO: random combo
        }
        .try_into()
        .unwrap()
    }

    /// Test that the upper-bound solver is monotonic,
    /// i.e. the quality UB of a state is never less than the quality UB of any of its children.
    fn monotonic_fuzz_check(settings: Settings) {
        let mut solver = UpperBoundSolver::new(settings);
        for _ in 0..10000 {
            let state = random_state(&settings);
            let state_upper_bound = solver.quality_upper_bound(state);
            for action in settings.allowed_actions.actions_iter() {
                let child_upper_bound = match state.use_action(action, Condition::Normal, &settings)
                {
                    Ok(child) => match InProgress::try_from(child) {
                        Ok(child) => solver.quality_upper_bound(child),
                        Err(_) if child.missing_progress == 0 => {
                            settings.max_quality - child.get_missing_quality()
                        }
                        Err(_) => 0,
                    },
                    Err(_) => 0,
                };
                if state_upper_bound < child_upper_bound {
                    dbg!(state, action, state_upper_bound, child_upper_bound);
                    panic!("Parent's upper bound is less than child's upper bound");
                }
            }
        }
    }

    #[test]
    fn test_monotonic_normal_sim() {
        let settings = Settings {
            max_cp: 360,
            max_durability: 70,
            max_progress: 1000,
            max_quality: 20000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 100,
            allowed_actions: ActionMask::from_level(100, true, false),
            adversarial: false,
        };
        monotonic_fuzz_check(settings);
    }

    #[test]
    fn test_monotonic_adversarial_sim() {
        let settings = Settings {
            max_cp: 360,
            max_durability: 70,
            max_progress: 1000,
            max_quality: 20000,
            base_progress: 100,
            base_quality: 100,
            initial_quality: 0,
            job_level: 100,
            allowed_actions: ActionMask::from_level(100, true, false),
            adversarial: true,
        };
        monotonic_fuzz_check(settings);
    }
}
