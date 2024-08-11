use crate::{
    actions::{PROGRESS_ACTIONS, QUALITY_ACTIONS},
    utils::{ParetoFrontBuilder, ParetoFrontId, ParetoValue},
};
use simulator::{Action, ActionMask, Combo, Condition, Settings, SimulationState, SingleUse};

use rustc_hash::FxHashMap as HashMap;

use super::state::ReducedState;

const SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(QUALITY_ACTIONS)
    .add(Action::WasteNot)
    .add(Action::WasteNot2);

pub struct QualityUpperBoundSolver {
    settings: Settings,
    durability_cost: i16,
    solved_states: HashMap<ReducedState, ParetoFrontId>,
    pareto_front_builder: ParetoFrontBuilder<u16, u16>,
}

impl QualityUpperBoundSolver {
    pub fn new(settings: Settings) -> Self {
        dbg!(std::mem::size_of::<ReducedState>());
        dbg!(std::mem::align_of::<ReducedState>());
        let mut durability_cost = Action::MasterMend.cp_cost() / 6;
        if settings.allowed_actions.has(Action::Manipulation) {
            durability_cost = std::cmp::min(durability_cost, Action::Manipulation.cp_cost() / 8);
        }
        if settings.allowed_actions.has(Action::ImmaculateMend) {
            durability_cost = std::cmp::min(
                durability_cost,
                Action::ImmaculateMend.cp_cost() / (settings.max_durability as i16 / 5 - 1),
            );
        }
        Self {
            settings,
            durability_cost,
            solved_states: HashMap::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.max_progress,
                settings.max_quality.saturating_mul(2),
            ),
        }
    }

    /// Returns an upper-bound on the maximum Quality achievable from this state while also maxing out Progress.
    /// The returned upper-bound is clamped to 2 times settings.max_quality.
    /// There is no guarantee on the tightness of the upper-bound.
    pub fn quality_upper_bound(&mut self, mut state: SimulationState) -> u16 {
        let current_quality = state.get_quality();
        let missing_progress = self.settings.max_progress.saturating_sub(state.progress);

        // refund effects and durability
        state.cp += state.effects.manipulation() as i16 * (Action::Manipulation.cp_cost() / 8);
        state.cp += state.durability as i16 / 5 * self.durability_cost;
        if state.effects.trained_perfection() != SingleUse::Unavailable
            && self.settings.allowed_actions.has(Action::TrainedPerfection)
        {
            state.cp += self.durability_cost * 4;
        }

        state.durability = i8::MAX;

        let reduced_state = ReducedState::from_state(state, self.durability_cost);
        let pareto_front = match self.solved_states.get(&reduced_state) {
            Some(id) => self.pareto_front_builder.retrieve(*id),
            None => {
                self.pareto_front_builder.clear();
                self.solve_state(reduced_state);
                self.pareto_front_builder.peek().unwrap()
            }
        };

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

    fn solve_state(&mut self, state: ReducedState) {
        if state.combo == Combo::None {
            self.solve_normal_state(state);
        } else {
            self.solve_combo_state(state)
        }
    }

    fn solve_normal_state(&mut self, state: ReducedState) {
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
        let id = self.pareto_front_builder.save().unwrap();
        self.solved_states.insert(state, id);
    }

    fn solve_combo_state(&mut self, state: ReducedState) {
        match self.solved_states.get(&state.to_non_combo()) {
            Some(id) => self.pareto_front_builder.push_from_id(*id),
            None => self.solve_normal_state(state.to_non_combo()),
        }
        match state.combo {
            Combo::None => unreachable!(),
            Combo::SynthesisBegin => {
                for action in [Action::MuscleMemory, Action::Reflect, Action::TrainedEye] {
                    if self.settings.allowed_actions.has(action) {
                        self.build_child_front(state, action);
                    }
                }
            }
            Combo::BasicTouch => {
                for action in [Action::ComboRefinedTouch, Action::ComboStandardTouch] {
                    if self.settings.allowed_actions.has(action) {
                        self.build_child_front(state, action);
                    }
                }
            }
            Combo::StandardTouch => {
                if self
                    .settings
                    .allowed_actions
                    .has(Action::ComboAdvancedTouch)
                {
                    self.build_child_front(state, Action::ComboAdvancedTouch);
                }
            }
        }
    }

    fn build_child_front(&mut self, state: ReducedState, action: Action) {
        if let Ok(new_state) =
            state
                .to_state()
                .use_action(action, Condition::Normal, &self.settings)
        {
            let action_progress = new_state.progress;
            let action_quality = new_state.get_quality();
            let new_state = ReducedState::from_state(new_state, self.durability_cost);
            if new_state.cp >= self.durability_cost {
                match self.solved_states.get(&new_state) {
                    Some(id) => self.pareto_front_builder.push_from_id(*id),
                    None => self.solve_state(new_state),
                }
                self.pareto_front_builder.map(move |value| {
                    value.first += action_progress;
                    value.second += action_quality;
                });
                self.pareto_front_builder.merge();
            } else if new_state.cp >= -self.durability_cost && action_progress != 0 {
                // "durability" must not go lower than -5
                // last action must be a progress increase
                self.pareto_front_builder
                    .push_from_slice(&[ParetoValue::new(action_progress, action_quality)]);
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

    fn solve(settings: Settings, actions: &[Action]) -> u16 {
        let state = SimulationState::from_macro(&settings, actions).unwrap();
        let result = QualityUpperBoundSolver::new(settings).quality_upper_bound(state);
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
        assert_eq!(result, 3352);
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
        assert_eq!(result, 3242);
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
        assert_eq!(result, 4693);
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
        assert_eq!(result, 4693);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2075);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: true,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 2075);
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
        assert_eq!(result, 2484);
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
        assert_eq!(result, 2484);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 4438);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: true,
        };
        let result = solve(settings, &[Action::MuscleMemory]);
        assert_eq!(result, 4438);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[Action::Reflect]);
        assert_eq!(result, 4449);
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
            job_level: 90,
            allowed_actions: ActionMask::from_level(90)
                .remove(Action::Manipulation)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 4510);
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
            job_level: 100,
            allowed_actions: ActionMask::from_level(100)
                .remove(Action::Manipulation)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
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
            job_level: 100,
            allowed_actions: ActionMask::from_level(100)
                .remove(Action::Manipulation)
                .remove(Action::TrainedEye)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
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
            job_level: 100,
            allowed_actions: ActionMask::from_level(100)
                .remove(Action::Manipulation)
                .remove(Action::HeartAndSoul)
                .remove(Action::QuickInnovation),
            adversarial: false,
        };
        let result = solve(settings, &[]);
        assert_eq!(result, 24260);
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
        let mut solver = QualityUpperBoundSolver::new(settings);
        for _ in 0..10000 {
            let state = random_state(&settings);
            let state_upper_bound = solver.quality_upper_bound(state);
            for action in settings.allowed_actions.actions_iter() {
                let child_upper_bound = match state.use_action(action, Condition::Normal, &settings)
                {
                    Ok(child) => match child.is_final(&settings) {
                        false => solver.quality_upper_bound(child),
                        true if child.progress >= settings.max_progress => child.get_quality(),
                        true => 0,
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
            max_quality: 20000,
            base_progress: 100,
            base_quality: 100,
            job_level: 100,
            allowed_actions: ActionMask::all(),
            adversarial: true,
        };
        monotonic_fuzz_check(settings);
    }
}
