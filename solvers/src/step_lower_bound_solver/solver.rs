use crate::{
    actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS},
    utils::{ParetoFrontBuilder, ParetoValue},
};
use simulator::{Action, ActionMask, Condition, Settings, SimulationState};

use rustc_hash::FxHashMap as HashMap;

use super::state::{ReducedState, ReducedStateWithDurability, ReducedStateWithoutDurability};

const SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .union(DURABILITY_ACTIONS)
    .remove(Action::Manipulation);

pub struct StepLowerBoundSolver {
    settings: Settings,
    fast_solver: StepLowerBoundSolverImpl<ReducedStateWithoutDurability>,
    slow_solver: StepLowerBoundSolverImpl<ReducedStateWithDurability>,
}

impl StepLowerBoundSolver {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            fast_solver: StepLowerBoundSolverImpl::new(settings),
            slow_solver: StepLowerBoundSolverImpl::new(settings),
        }
    }

    /// Returns a lower-bound on the additional steps required to max out both Progress and Quality from this state.
    pub fn step_lower_bound(&mut self, state: SimulationState, fast_mode: bool) -> u8 {
        let mut lo = 0;
        let mut hi = 1;
        while self.fast_solver.quality_upper_bound(state, hi) < self.settings.max_quality {
            lo = hi;
            hi *= 2;
        }
        while lo + 1 < hi {
            if self.fast_solver.quality_upper_bound(state, (lo + hi) / 2)
                < self.settings.max_quality
            {
                lo = (lo + hi) / 2;
            } else {
                hi = (lo + hi) / 2;
            }
        }
        if fast_mode {
            return hi;
        }
        while self.slow_solver.quality_upper_bound(state, hi) < self.settings.max_quality {
            hi += 1;
        }
        hi
    }
}

struct StepLowerBoundSolverImpl<S: ReducedState> {
    settings: Settings,
    solved_states: HashMap<S, Box<[ParetoValue<u16, u16>]>>,
    pareto_front_builder: ParetoFrontBuilder<u16, u16>,
}

impl<S: ReducedState> StepLowerBoundSolverImpl<S> {
    pub fn new(settings: Settings) -> Self {
        dbg!(std::mem::size_of::<S>());
        dbg!(std::mem::align_of::<S>());
        Self {
            settings,
            solved_states: HashMap::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.max_progress,
                settings.max_quality,
            ),
        }
    }

    pub fn quality_upper_bound(&mut self, state: SimulationState, step_budget: u8) -> u16 {
        let current_quality = state.get_quality();
        let missing_progress = self.settings.max_progress.saturating_sub(state.progress);

        let reduced_state = ReducedState::from_state(state, step_budget);

        if !self.solved_states.contains_key(&reduced_state) {
            self.solve_state(reduced_state);
            self.pareto_front_builder.clear();
        }
        let pareto_front = self.solved_states.get(&reduced_state).unwrap();

        match pareto_front.last() {
            Some(element) => {
                if element.first < missing_progress {
                    return 0;
                }
            }
            None => return 0,
        }

        let index = match pareto_front.binary_search_by_key(&missing_progress, |value| value.first)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        std::cmp::min(
            self.settings.max_quality.saturating_mul(2),
            pareto_front[index].second.saturating_add(current_quality),
        )
    }

    fn solve_state(&mut self, state: S) {
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

    fn build_child_front(&mut self, state: S, action: Action) {
        if let Ok(new_state) =
            state
                .to_state()
                .use_action(action, Condition::Normal, &self.settings)
        {
            let action_progress = new_state.progress;
            let action_quality = new_state.get_quality();
            let new_state = S::from_state(new_state, state.steps_budget() - 1);
            if new_state.steps_budget() != 0 {
                match self.solved_states.get(&new_state) {
                    Some(pareto_front) => self.pareto_front_builder.push(pareto_front),
                    None => self.solve_state(new_state),
                }
                self.pareto_front_builder.map(move |value| {
                    value.first += action_progress;
                    value.second += action_quality;
                });
                self.pareto_front_builder.merge();
            } else if action_progress != 0 {
                // last action must be a progress increase
                self.pareto_front_builder
                    .push(&[ParetoValue::new(action_progress, action_quality)]);
                self.pareto_front_builder.merge();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use simulator::{Combo, Effects, SimulationState};

    use super::*;

    fn solve(settings: Settings, actions: &[Action]) -> u8 {
        let state = SimulationState::from_macro(&settings, actions).unwrap();
        let result = StepLowerBoundSolver::new(settings).step_lower_bound(state, false);
        dbg!(result);
        result
    }

    #[test]
    fn test_01() {
        let settings = Settings {
            max_cp: 553,
            max_durability: 70,
            max_progress: 2400,
            max_quality: 1700,
            base_progress: 100,
            base_quality: 100,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
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
        assert_eq!(result, 5);
    }

    #[test]
    fn test_adversarial_01() {
        let settings = Settings {
            max_cp: 553,
            max_durability: 70,
            max_progress: 2400,
            max_quality: 1700,
            base_progress: 100,
            base_quality: 100,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
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
        assert_eq!(result, 6);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
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
        assert_eq!(result, 15);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
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
        assert_eq!(result, 15);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
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
        assert_eq!(result, 12);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
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
        assert_eq!(result, 12);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 18);
    }

    #[test]
    fn test_adversarial_04() {
        let settings = Settings {
            max_cp: 411,
            max_durability: 60,
            max_progress: 1990,
            max_quality: 2900,
            base_progress: 100,
            base_quality: 100,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: true,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 13);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 11);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: true,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 11);
    }

    #[test]
    fn test_06() {
        let settings = Settings {
            max_cp: 673,
            max_durability: 60,
            max_progress: 2345,
            max_quality: 3500,
            base_progress: 100,
            base_quality: 100,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_adversarial_06() {
        let settings = Settings {
            max_cp: 673,
            max_durability: 60,
            max_progress: 2345,
            max_quality: 1200,
            base_progress: 100,
            base_quality: 100,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: true,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 10);
    }

    #[test]
    fn test_07() {
        let settings = Settings {
            max_cp: 673,
            max_durability: 60,
            max_progress: 2345,
            max_quality: 3123,
            base_progress: 100,
            base_quality: 100,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[Action::Reflect]);
        assert_eq!(result, 14);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[Action::PrudentTouch]);
        assert_eq!(result, 1);
    }

    #[test]
    fn test_09() {
        let settings = Settings {
            max_cp: 700,
            max_durability: 70,
            max_progress: 2500,
            max_quality: 3000,
            base_progress: 100,
            base_quality: 100,
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::Manipulation)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 15);
    }

    #[test]
    fn test_10() {
        let settings = Settings {
            max_cp: 400,
            max_durability: 80,
            max_progress: 1200,
            max_quality: 2400,
            base_progress: 100,
            base_quality: 100,
            job_level: 100,
            allowed_actions: ActionMask::from_level(100)
                .remove(Action::Manipulation)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 11);
    }

    #[test]
    fn test_11() {
        let settings = Settings {
            max_cp: 320,
            max_durability: 80,
            max_progress: 1600,
            max_quality: 2000,
            base_progress: 100,
            base_quality: 100,
            job_level: 100,
            allowed_actions: ActionMask::from_level(100)
                .remove(Action::Manipulation)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 11);
    }

    #[test]
    fn test_12() {
        let settings = Settings {
            max_cp: 320,
            max_durability: 80,
            max_progress: 1600,
            max_quality: 2100,
            base_progress: 100,
            base_quality: 100,
            job_level: 100,
            allowed_actions: ActionMask::from_level(100)
                .remove(Action::Manipulation)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 5);
    }

    fn random_effects(adversarial: bool) -> Effects {
        Effects::default()
            .with_inner_quiet(rand::thread_rng().gen_range(0..=10))
            .with_great_strides(rand::thread_rng().gen_range(0..=3))
            .with_innovation(rand::thread_rng().gen_range(0..=4))
            .with_veneration(rand::thread_rng().gen_range(0..=4))
            .with_waste_not(rand::thread_rng().gen_range(0..=8))
            .with_manipulation(rand::thread_rng().gen_range(0..=8))
            .with_quick_innovation_used(rand::random())
            .with_guard(if adversarial {
                rand::thread_rng().gen_range(0..=1)
            } else {
                0
            })
    }

    fn random_state(settings: &Settings) -> SimulationState {
        const COMBOS: [Combo; 3] = [Combo::None, Combo::BasicTouch, Combo::StandardTouch];
        SimulationState {
            cp: rand::thread_rng().gen_range(0..=settings.max_cp),
            durability: rand::thread_rng().gen_range(1..=(settings.max_durability / 5)) * 5,
            progress: rand::thread_rng().gen_range(0..settings.max_progress),
            unreliable_quality: [settings.max_quality; 2],
            effects: random_effects(settings.adversarial),
            combo: COMBOS[rand::thread_rng().gen_range(0..3)],
        }
        .try_into()
        .unwrap()
    }

    /// Test that the upper-bound solver is monotonic,
    /// i.e. the quality UB of a state is never less than the quality UB of any of its children.
    fn monotonic_fuzz_check(settings: Settings) {
        let mut solver = StepLowerBoundSolver::new(settings);
        for _ in 0..10000 {
            let fast_mode: bool = rand::random();
            let state = random_state(&settings);
            let state_lower_bound = solver.step_lower_bound(state, fast_mode);
            for action in settings.allowed_actions.actions_iter() {
                let child_lower_bound = match state.use_action(action, Condition::Normal, &settings)
                {
                    Ok(child) => match child.is_final(&settings) {
                        false => solver.step_lower_bound(child, fast_mode),
                        true if child.progress >= settings.max_progress
                            && child.get_quality() >= settings.max_quality =>
                        {
                            0
                        }
                        true => u8::MAX,
                    },
                    Err(_) => u8::MAX,
                };
                if state_lower_bound > child_lower_bound.saturating_add(1) {
                    dbg!(state, action, state_lower_bound, child_lower_bound);
                    panic!("Parent's step lower bound is greater than child's step lower bound");
                }
            }
        }
        for _ in 0..10000 {
            let state = random_state(&settings);
            let fast_mode_lower_bound = solver.step_lower_bound(state, true);
            let slow_mode_lower_bound = solver.step_lower_bound(state, false);
            if fast_mode_lower_bound > slow_mode_lower_bound {
                dbg!(state, fast_mode_lower_bound, slow_mode_lower_bound);
                panic!("Slow mode must be at least as tight as fast mode");
            }
        }
    }

    #[test]
    fn test_monotonic_normal_sim() {
        let settings = Settings {
            max_cp: 360,
            max_durability: 70,
            max_progress: 1000,
            max_quality: 2600,
            base_progress: 100,
            base_quality: 100,
            job_level: 100,
            allowed_actions: ActionMask::all(),
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
            max_quality: 2400,
            base_progress: 100,
            base_quality: 100,
            job_level: 100,
            allowed_actions: ActionMask::all(),
            adversarial: true,
        };
        monotonic_fuzz_check(settings);
    }
}
