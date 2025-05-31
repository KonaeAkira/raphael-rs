use std::num::NonZeroU8;

use crate::{
    SolverException, SolverSettings,
    actions::{FULL_SEARCH_ACTIONS, use_action_combo},
    utils::{self, largest_single_action_progress_increase},
};
use raphael_sim::*;
use rayon::iter::{
    FromParallelIterator, IntoParallelIterator, IntoParallelRefIterator, ParallelIterator,
};
use rustc_hash::FxHashSet;

use super::state::ReducedState;

type ParetoValue = utils::ParetoValue<u32, u32>;
type ParetoFrontBuilder = utils::ParetoFrontBuilder<u32, u32>;
type SolvedStates = rustc_hash::FxHashMap<ReducedState, Box<[ParetoValue]>>;

#[derive(Debug, Clone, Copy)]
pub struct StepLbSolverStats {
    pub parallel_states: usize,
    pub pareto_values: usize,
}

pub struct StepLbSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    precompute_templates: Vec<Template>,
    iq_quality_lut: [u32; 11],
    largest_progress_increase: u32,
}

impl StepLbSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        let iq_quality_lut = utils::compute_iq_quality_lut(&settings);
        settings.simulator_settings.adversarial = false;
        ReducedState::optimize_action_mask(&mut settings.simulator_settings);
        Self {
            settings,
            interrupt_signal,
            solved_states: SolvedStates::default(),
            precompute_templates: Self::generate_precompute_templates(&settings),
            iq_quality_lut,
            largest_progress_increase: largest_single_action_progress_increase(&settings),
        }
    }

    fn generate_precompute_templates(settings: &SolverSettings) -> Vec<Template> {
        let mut templates = rustc_hash::FxHashSet::<Template>::default();
        let mut queue = std::collections::VecDeque::<Template>::new();

        let seed_template = Template {
            durability: settings.max_durability(),
            effects: Effects::initial(&settings.simulator_settings)
                .with_adversarial_guard(false)
                .with_combo(Combo::None),
        };
        templates.insert(seed_template);
        queue.push_back(seed_template);

        while let Some(template) = queue.pop_front() {
            let state = template.instantiate(NonZeroU8::MAX);
            for action in FULL_SEARCH_ACTIONS {
                if let Ok(new_state) = use_action_combo(settings, state.to_state(), action) {
                    let new_state = ReducedState::from_state(new_state, NonZeroU8::MAX);
                    if new_state.durability > 0 {
                        let new_template = Template {
                            durability: new_state.durability,
                            effects: new_state.effects,
                        };
                        if !templates.contains(&new_template) {
                            templates.insert(new_template);
                            queue.push_back(new_template);
                        }
                    }
                }
            }
        }

        templates.into_iter().collect()
    }

    pub fn precompute(&mut self) {
        let mut next_precompute_step_budget = NonZeroU8::new(1).unwrap();
        while !self.precompute_templates.is_empty() && !self.interrupt_signal.is_set() {
            // A lot of templates map to the same state at lower step budgets due to effect and durability optimizations.
            // Here we deduplicate the instantiated templates to avoid solving duplicate states.
            let instantiated_templates: FxHashSet<ReducedState> = self
                .precompute_templates
                .iter()
                .map(|template| template.instantiate(next_precompute_step_budget))
                .collect();

            let init = || {
                ParetoFrontBuilder::new(self.settings.max_progress(), self.settings.max_quality())
            };
            let solved_templates = instantiated_templates
                .into_par_iter()
                .map_init(init, |pareto_front_builder, state| {
                    let pareto_front = self.solve_precompute_state(pareto_front_builder, state);
                    (state, pareto_front)
                })
                .collect_vec_list();

            self.solved_states
                .extend(solved_templates.into_iter().flatten());

            let filtered_templates = self.precompute_templates.par_iter().filter(|template| {
                let state = template.instantiate(next_precompute_step_budget);
                let pareto_front = self.solved_states.get(&state).unwrap();
                // Values are sorted Progress-increaasing and Quality-decreasing.
                // The last value is the value with the most Progress.
                let value = pareto_front.last().unwrap();
                // Estimate the max quality that this state ever needs to achieve.
                // Over-estimating the max needed quality leads to redundant states being precomputed.
                // Under-estimating the max needed quality could lead to solver crash during precompute from templates being removed too early.
                let max_needed_quality = if state.effects.allow_quality_actions() {
                    let min_cur_quality =
                        self.iq_quality_lut[usize::from(state.effects.inner_quiet())];
                    self.settings.max_quality().saturating_sub(min_cur_quality)
                } else {
                    0
                };
                value.first < self.settings.max_progress() || value.second < max_needed_quality
            });
            self.precompute_templates = Vec::from_par_iter(filtered_templates.copied());

            next_precompute_step_budget = next_precompute_step_budget.saturating_add(1);

            log::trace!(
                "StepLbSolver - templates: {}, solved_states: {}",
                self.precompute_templates.len(),
                self.solved_states.len()
            );
        }
    }

    fn solve_precompute_state(
        &self,
        pareto_front_builder: &mut ParetoFrontBuilder,
        state: ReducedState,
    ) -> Box<[ParetoValue]> {
        pareto_front_builder.clear();
        pareto_front_builder.push_empty();
        for action in FULL_SEARCH_ACTIONS {
            if state.steps_budget.get() < action.steps() {
                continue;
            }
            let new_step_budget = state.steps_budget.get() - action.steps();
            if let Ok(new_state) = use_action_combo(&self.settings, state.to_state(), action) {
                let progress = new_state.progress;
                let quality = new_state.quality;
                if let Ok(new_step_budget) = NonZeroU8::try_from(new_step_budget)
                    && new_state.durability > 0
                {
                    let new_state = ReducedState::from_state(new_state, new_step_budget);
                    if let Some(pareto_front) = self.solved_states.get(&new_state) {
                        pareto_front_builder.push_slice(pareto_front);
                    } else if !new_state.effects.allow_quality_actions() {
                        // States that disallow quality actions get filtered out early because they only need to reach max_progress, whereas normal states need to reach both max_progress and max_quality to be fitered out.
                        // So if the new state does not allow quality actions and cannot be found in the already solved state, we assume that it has reached max_progress using a lower step budget.
                        // IMPORTANT: A missing child state could also mean that there is something wrong with the precompute template generation, but the consistency fuzz check should hopefully catch this case.
                        pareto_front_builder
                            .push_slice(&[ParetoValue::new(self.settings.max_progress(), 0)]);
                    } else {
                        unreachable!("Parent: {state:?}\nChild: {new_state:?}\nAction: {action:?}");
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
        mut state: SimulationState,
        step_budget: NonZeroU8,
    ) -> Result<Option<u32>, SolverException> {
        let mut required_progress = self.settings.max_progress() - state.progress;
        if state.effects.muscle_memory() != 0 {
            // Assume MuscleMemory can be used to its max potential and remove the effect to reduce the number of states that need to be solved.
            required_progress = required_progress.saturating_sub(self.largest_progress_increase);
            state.effects.set_muscle_memory(0);
        }
        let reduced_state = ReducedState::from_state(state, step_budget);
        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality_ub = pareto_front
                .get(index)
                .map(|value| state.quality + value.second);
            Ok(quality_ub)
        } else {
            Err(SolverException::InternalError(
                "StepLbSolver: Unknown state queried.".to_string(),
            ))
        }
    }

    pub fn runtime_stats(&self) -> StepLbSolverStats {
        StepLbSolverStats {
            parallel_states: self.solved_states.len(),
            pareto_values: self.solved_states.values().map(|value| value.len()).sum(),
        }
    }
}

impl Drop for StepLbSolver {
    fn drop(&mut self) {
        let runtime_stats = self.runtime_stats();
        log::debug!(
            "StepLbSolver - par_states: {}, values: {}",
            runtime_stats.parallel_states,
            runtime_stats.pareto_values
        );
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct Template {
    durability: u16,
    effects: Effects,
}

impl Template {
    pub fn instantiate(&self, step_budget: NonZeroU8) -> ReducedState {
        let state = SimulationState {
            durability: self.durability,
            effects: self.effects,
            cp: 0,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
        };
        ReducedState::from_state(state, step_budget)
    }
}
