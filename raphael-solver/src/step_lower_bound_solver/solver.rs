use std::num::NonZeroU8;

use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS, use_action_combo},
    utils,
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
    pub sequential_states: usize,
    pub pareto_values: usize,
}

pub struct StepLbSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    pareto_front_builder: ParetoFrontBuilder,
    precompute_templates: Vec<Template>,
    next_precompute_step_budget: NonZeroU8,
    precomputed_states: usize,
    /// Maps InnerQuiet to the minimum amount of Quality that
    /// a state with the corresponding InnerQuiet can have.
    iq_quality_lut: [u32; 11],
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
            precompute_templates: Self::generate_precompute_templates(&settings),
            next_precompute_step_budget: NonZeroU8::new(1).unwrap(),
            precomputed_states: 0,
            iq_quality_lut: compute_iq_quality_lut(&settings),
        }
    }

    fn generate_precompute_templates(settings: &SolverSettings) -> Vec<Template> {
        let mut templates = rustc_hash::FxHashSet::<Template>::default();
        let mut queue = std::collections::VecDeque::<Template>::new();

        let seed_template = Template {
            durability: settings.max_durability(),
            effects: Effects::initial(&settings.simulator_settings)
                .with_quick_innovation_available(false)
                .with_heart_and_soul_available(false)
                .with_adversarial_guard(true)
                .with_combo(Combo::None),
        };
        templates.insert(seed_template);
        queue.push_back(seed_template);

        while let Some(template) = queue.pop_front() {
            let state = template.instantiate(NonZeroU8::MAX);
            let search_actions = match state.effects.allow_quality_actions() {
                false => PROGRESS_ONLY_SEARCH_ACTIONS,
                true => FULL_SEARCH_ACTIONS,
            };
            for &action in search_actions {
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

    fn precompute_next_step_budget(&mut self) {
        // A lot of templates map to the same state at lower step budgets due to effect and durability optimizations.
        // Here we deduplicate the instantiated templates to avoid solving duplicate states.
        let instantiated_templates: FxHashSet<ReducedState> = self
            .precompute_templates
            .iter()
            .map(|template| template.instantiate(self.next_precompute_step_budget))
            .collect();

        let init =
            || ParetoFrontBuilder::new(self.settings.max_progress(), self.settings.max_quality());
        let solved_templates = instantiated_templates
            .into_par_iter()
            .map_init(init, |pareto_front_builder, state| {
                let pareto_front = self.solve_precompute_state(pareto_front_builder, state);
                (state, pareto_front)
            })
            .collect_vec_list();

        let num_solved_states_before = self.solved_states.len();
        self.solved_states
            .extend(solved_templates.into_iter().flatten());
        self.precomputed_states += self.solved_states.len() - num_solved_states_before;

        let filtered_templates = self.precompute_templates.par_iter().filter(|template| {
            let state = template.instantiate(self.next_precompute_step_budget);
            let pareto_front = self.solved_states.get(&state).unwrap();
            // Values are sorted Progress-increaasing and Quality-decreasing.
            // The last value is the value with the most Progress.
            let value = pareto_front.last().unwrap();
            // Estimate the max quality that this state ever needs to achieve.
            // Over-estimating the max needed quality leads to redundant states being precomputed.
            // Under-estimating the max needed quality could lead to solver crash during precompute from templates being removed too early.
            let max_needed_quality = {
                let min_cur_quality = self.iq_quality_lut[usize::from(state.effects.inner_quiet())];
                self.settings.max_quality().saturating_sub(min_cur_quality)
            };
            value.first < self.settings.max_progress() || value.second < max_needed_quality
        });
        self.precompute_templates = Vec::from_par_iter(filtered_templates.copied());

        self.next_precompute_step_budget = self.next_precompute_step_budget.saturating_add(1);

        log::trace!(
            "StepLbSolver - templates: {}, solved_states: {}",
            self.precompute_templates.len(),
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
                if let Ok(new_step_budget) = NonZeroU8::try_from(new_step_budget)
                    && new_state.durability > 0
                {
                    let new_state = ReducedState::from_state(new_state, new_step_budget);
                    if let Some(pareto_front) = self.solved_states.get(&new_state) {
                        pareto_front_builder.push_slice(pareto_front);
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
        state: SimulationState,
        step_budget: NonZeroU8,
    ) -> Result<Option<u32>, SolverException> {
        if state.effects.combo() != Combo::None {
            return Err(SolverException::InternalError(format!(
                "\"{:?}\" combo in step lower bound solver",
                state.effects.combo()
            )));
        }

        while self.next_precompute_step_budget <= step_budget {
            self.precompute_next_step_budget();
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
            parallel_states: self.precomputed_states,
            sequential_states: self.solved_states.len() - self.precomputed_states,
            pareto_values: self.solved_states.values().map(|value| value.len()).sum(),
        }
    }
}

impl Drop for StepLbSolver {
    fn drop(&mut self) {
        let runtime_stats = self.runtime_stats();
        log::debug!(
            "StepLbSolver - par_states: {}, seq_states: {}, values: {}",
            runtime_stats.parallel_states,
            runtime_stats.sequential_states,
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

fn compute_iq_quality_lut(settings: &SolverSettings) -> [u32; 11] {
    let mut result = [u32::MAX; 11];
    result[0] = 0;
    for iq in 0..10 {
        let state = SimulationState {
            cp: 500,
            durability: 100,
            progress: 0,
            quality: 0,
            unreliable_quality: 0,
            effects: Effects::new()
                .with_allow_quality_actions(true)
                .with_adversarial_guard(true)
                .with_inner_quiet(iq),
        };
        for &action in FULL_SEARCH_ACTIONS {
            if let Ok(new_state) = use_action_combo(settings, state, action) {
                let new_iq = new_state.effects.inner_quiet();
                if new_iq > iq {
                    let action_quality = new_state.quality;
                    result[usize::from(new_iq)] = std::cmp::min(
                        result[usize::from(new_iq)],
                        result[usize::from(iq)] + action_quality,
                    );
                }
            }
        }
    }
    result
}
