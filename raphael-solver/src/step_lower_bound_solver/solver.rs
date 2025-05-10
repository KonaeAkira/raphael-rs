use std::num::NonZeroU8;

use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS, use_action_combo},
    utils,
};
use raphael_sim::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::state::ReducedState;

type ParetoValue = utils::ParetoValue<u32, u32>;
type ParetoFrontBuilder = utils::ParetoFrontBuilder<u32, u32>;
type SolvedStates = rustc_hash::FxHashMap<ReducedState, Box<[ParetoValue]>>;

#[derive(Debug, Clone, Copy)]
pub struct StepLbSolverStats {
    pub states: usize,
    pub pareto_values: usize,
}

pub struct StepLbSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    pareto_front_builder: ParetoFrontBuilder,
}

impl StepLbSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        ReducedState::optimize_action_mask(&mut settings.simulator_settings);
        Self {
            settings,
            interrupt_signal,
            solved_states: SolvedStates::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.max_progress(),
                settings.max_quality(),
            ),
        }
    }

    fn generate_precompute_templates(&self) -> Box<[Template]> {
        let mut templates = rustc_hash::FxHashSet::<Template>::default();
        let mut queue = std::collections::VecDeque::<Template>::new();

        let initial_node = Template {
            durability: self.settings.max_durability(),
            effects: Effects::initial(&self.settings.simulator_settings)
                .with_trained_perfection_available(false)
                .with_quick_innovation_available(false)
                .with_heart_and_soul_available(false)
                .with_adversarial_guard(true)
                .with_combo(Combo::None),
        };
        templates.insert(initial_node);
        queue.push_back(initial_node);

        while let Some(node) = queue.pop_front() {
            let state = ReducedState {
                steps_budget: NonZeroU8::MAX,
                durability: node.durability,
                effects: node.effects,
            };
            let search_actions = match node.effects.allow_quality_actions() {
                false => PROGRESS_ONLY_SEARCH_ACTIONS,
                true => FULL_SEARCH_ACTIONS,
            };
            for &action in search_actions {
                if let Ok(new_state) = use_action_combo(&self.settings, state.to_state(), action) {
                    let new_state = ReducedState::from_state(new_state, NonZeroU8::MAX);
                    let new_node = Template {
                        durability: new_state.durability,
                        effects: new_state.effects,
                    };
                    if !templates.contains(&new_node) {
                        templates.insert(new_node);
                        queue.push_back(new_node);
                    }
                }
            }
        }

        templates.into_iter().collect()
    }

    pub fn precompute(&mut self) {
        if !self.solved_states.is_empty() || rayon::current_num_threads() <= 1 {
            return;
        }

        let templates = self.generate_precompute_templates();
        for step_budget in 1..=8 {
            let step_budget = NonZeroU8::try_from(step_budget).unwrap();
            if self.interrupt_signal.is_set() {
                return;
            }
            let init = || {
                ParetoFrontBuilder::new(self.settings.max_progress(), self.settings.max_quality())
            };
            let solved_states = templates
                .par_iter()
                .map_init(init, |pareto_front_builder, template| {
                    let optimized_durability = ReducedState::optimize_durability(
                        template.effects,
                        template.durability,
                        step_budget,
                    );
                    let optimized_effects =
                        ReducedState::optimize_effects(template.effects, step_budget);
                    let state = ReducedState {
                        steps_budget: step_budget,
                        durability: optimized_durability,
                        effects: optimized_effects,
                    };
                    let pareto_front = self.solve_precompute_state(pareto_front_builder, state);
                    (state, pareto_front)
                })
                .collect_vec_list();
            for thread_solved_states in solved_states {
                self.solved_states.extend(thread_solved_states);
            }
        }

        log::debug!(
            "StepLbSolver - templates: {}, precomputed_states: {}",
            templates.len(),
            self.solved_states.len()
        );
    }

    fn solve_precompute_state(
        &self,
        pareto_front_builder: &mut ParetoFrontBuilder,
        state: ReducedState,
    ) -> Box<[ParetoValue]> {
        pareto_front_builder.clear();
        pareto_front_builder.push_empty();
        let search_actions = match state.effects.allow_quality_actions() {
            false => PROGRESS_ONLY_SEARCH_ACTIONS,
            true => FULL_SEARCH_ACTIONS,
        };
        for &action in search_actions {
            if state.steps_budget.get() < action.steps() {
                continue;
            }
            let new_step_budget = state.steps_budget.get() - action.steps();
            if let Ok(new_state) = use_action_combo(&self.settings, state.to_state(), action) {
                let progress = new_state.progress;
                let quality = new_state.quality;
                if let Ok(new_step_budget) = NonZeroU8::try_from(new_step_budget) {
                    let new_state = ReducedState::from_state(new_state, new_step_budget);
                    if let Some(pareto_front) = self.solved_states.get(&new_state) {
                        pareto_front_builder.push_slice(pareto_front);
                    } else {
                        unreachable!(
                            "Child state does not exist.\nParent state: {state:?}.\nChild state: {new_state:?}.\nAction: {action:?}."
                        );
                    }
                    pareto_front_builder
                        .peek_mut()
                        .unwrap()
                        .iter_mut()
                        .for_each(|value| {
                            value.first += progress;
                            value.second += quality;
                        });
                    pareto_front_builder.merge();
                } else if progress != 0 {
                    pareto_front_builder.push_slice(&[ParetoValue::new(progress, quality)]);
                    pareto_front_builder.merge();
                }
            }
        }
        Box::from(pareto_front_builder.peek().unwrap())
    }

    pub fn step_lower_bound(
        &mut self,
        state: SimulationState,
        hint: u8,
    ) -> Result<u8, SolverException> {
        if !state.effects.allow_quality_actions() && state.quality < self.settings.max_quality() {
            return Ok(u8::MAX);
        }
        let mut hint = NonZeroU8::try_from(std::cmp::max(hint, 1)).unwrap();
        while self
            .quality_upper_bound(state, hint)?
            .is_none_or(|quality_ub| quality_ub < self.settings.max_quality())
        {
            hint = hint.checked_add(1).unwrap();
        }
        Ok(hint.get())
    }

    fn quality_upper_bound(
        &mut self,
        state: SimulationState,
        step_budget: NonZeroU8,
    ) -> Result<Option<u32>, SolverException> {
        if state.effects.combo() != Combo::None {
            return Err(SolverException::InternalError(format!(
                "\"{:?}\" combo in step lower bound solver",
                state.effects.combo()
            )));
        }

        let reduced_state = ReducedState::from_state(state, step_budget);
        let required_progress = self.settings.max_progress() - state.progress;

        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality_ub = pareto_front
                .get(index)
                .map(|value| state.quality + value.second);
            return Ok(quality_ub);
        }

        self.solve_state(reduced_state)?;

        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality_ub = pareto_front
                .get(index)
                .map(|value| state.quality + value.second);
            return Ok(quality_ub);
        } else {
            unreachable!("State must be in memoization table after solver")
        }
    }

    fn solve_state(&mut self, reduced_state: ReducedState) -> Result<(), SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        self.pareto_front_builder.push_empty();
        let search_actions = match reduced_state.effects.allow_quality_actions() {
            false => PROGRESS_ONLY_SEARCH_ACTIONS,
            true => FULL_SEARCH_ACTIONS,
        };
        for &action in search_actions {
            if action.steps() <= reduced_state.steps_budget.get() {
                self.build_child_front(reduced_state, action)?;
                if self.pareto_front_builder.is_max() {
                    // stop early if both Progress and Quality are maxed out
                    // this optimization would work even better with better action ordering
                    // (i.e. if better actions are visited first)
                    break;
                }
            }
        }
        let pareto_front = Box::from(self.pareto_front_builder.peek().unwrap());
        self.solved_states.insert(reduced_state, pareto_front);
        Ok(())
    }

    fn build_child_front(
        &mut self,
        reduced_state: ReducedState,
        action: ActionCombo,
    ) -> Result<(), SolverException> {
        if let Ok(new_full_state) =
            use_action_combo(&self.settings, reduced_state.to_state(), action)
        {
            let action_progress = new_full_state.progress;
            let action_quality = new_full_state.quality;
            let new_step_budget = reduced_state.steps_budget.get() - action.steps();
            match NonZeroU8::try_from(new_step_budget) {
                Ok(new_step_budget) if new_full_state.durability > 0 => {
                    // New state is not final
                    let new_reduced_state =
                        ReducedState::from_state(new_full_state, new_step_budget);
                    if let Some(pareto_front) = self.solved_states.get(&new_reduced_state) {
                        self.pareto_front_builder.push_slice(pareto_front);
                    } else {
                        self.solve_state(new_reduced_state)?;
                    }
                    self.pareto_front_builder
                        .peek_mut()
                        .unwrap()
                        .iter_mut()
                        .for_each(|value| {
                            value.first += action_progress;
                            value.second += action_quality;
                        });
                    self.pareto_front_builder.merge();
                }
                _ if action_progress != 0 => {
                    // New state is final and last action increased Progress
                    self.pareto_front_builder
                        .push_slice(&[ParetoValue::new(action_progress, action_quality)]);
                    self.pareto_front_builder.merge();
                }
                _ => {
                    // New state is final but last action did not increase Progress
                    // Skip this state
                }
            }
        }
        Ok(())
    }

    pub fn runtime_stats(&self) -> StepLbSolverStats {
        StepLbSolverStats {
            states: self.solved_states.len(),
            pareto_values: self.solved_states.values().map(|value| value.len()).sum(),
        }
    }
}

impl Drop for StepLbSolver {
    fn drop(&mut self) {
        let runtime_stats = self.runtime_stats();
        log::debug!(
            "StepLbSolver - states: {}, values: {}",
            runtime_stats.states,
            runtime_stats.pareto_values
        );
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Template {
    durability: u16,
    effects: Effects,
}
