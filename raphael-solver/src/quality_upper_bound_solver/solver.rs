use crate::{
    SolverException, SolverSettings,
    actions::FULL_SEARCH_ACTIONS,
    macros::internal_error,
    utils::{self, ParetoFrontBuilder, ParetoValue},
};
use raphael_sim::*;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rustc_hash::FxHashMap;

use super::state::ReducedState;

#[derive(Debug, Clone, Copy)]
pub struct QualityUbSolverStats {
    pub parallel_states: usize,
    pub sequential_states: usize,
    pub pareto_values: usize,
}

pub struct QualityUbSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: FxHashMap<ReducedState, Box<[ParetoValue]>>,
    iq_quality_lut: [u32; 11],
    maximal_templates: FxHashMap<TemplateData, u16>,
    durability_cost: u16,
    largest_progress_increase: u32,
    precomputed_states: usize,
}

impl QualityUbSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        let durability_cost = durability_cost(&settings.simulator_settings);
        settings.simulator_settings.max_cp = {
            let initial_state = SimulationState::new(&settings.simulator_settings);
            ReducedState::from_state(initial_state, &settings, durability_cost).cp
        };
        Self {
            settings,
            interrupt_signal,
            solved_states: FxHashMap::default(),
            iq_quality_lut: utils::compute_iq_quality_lut(&settings),
            maximal_templates: FxHashMap::default(),
            durability_cost,
            largest_progress_increase: utils::largest_single_action_progress_increase(&settings),
            precomputed_states: 0,
        }
    }

    fn generate_precompute_templates(&self) -> Box<[Template]> {
        let mut templates = rustc_hash::FxHashMap::<TemplateData, u16>::default();
        let mut heap = std::collections::BinaryHeap::<Template>::default();

        let seed_template = {
            let seed_effects = Effects::initial(&self.settings.simulator_settings)
                .with_special_quality_state(SpecialQualityState::Normal)
                .with_trained_perfection_available(false)
                .with_combo(Combo::None);
            Template::new(self.settings.max_cp(), TemplateData::new(seed_effects, 0))
        };
        heap.push(seed_template);

        while let Some(template) = heap.pop() {
            let entry = templates.entry(template.data).or_default();
            if template.max_instantiated_cp > *entry {
                *entry = template.max_instantiated_cp;
                let state = template.instantiate(template.max_instantiated_cp).unwrap();
                for action in FULL_SEARCH_ACTIONS {
                    if let Some((new_state, _, _)) =
                        state.use_action(action, &self.settings, self.durability_cost)
                    {
                        let new_template_data = TemplateData {
                            effects: new_state.effects,
                            compressed_unreliable_quality: new_state.compressed_unreliable_quality,
                        };
                        let new_template = Template::new(
                            new_state.cp,
                            TemplateData::new(
                                new_state.effects,
                                new_state.compressed_unreliable_quality,
                            ),
                        );
                        let new_entry = templates.entry(new_template_data).or_default();
                        if new_template.max_instantiated_cp > *new_entry {
                            heap.push(new_template);
                        }
                    }
                }
            }
        }

        templates
            .into_iter()
            .map(|(template_data, max_cp)| Template::new(max_cp, template_data))
            .collect()
    }

    pub fn precompute(&mut self) -> Result<(), SolverException> {
        let all_templates = self.generate_precompute_templates();
        // States are computed in order of less CP to more CP.
        // States currently being computed assume that child states have already been computed.
        // This is the reason why states with HeartAndSoul and QuickInnovation available must be computed separately.
        // HeartAndSoul enables the use of TricksOfTrade, which restores CP.
        // QuickInnovation requires no CP (and no durability, so durability cost in terms of CP is 0).
        for (heart_and_soul, quick_innovation) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            let mut templates: Vec<_> = all_templates
                .iter()
                .filter(|template| {
                    template.data.effects.heart_and_soul_available() == heart_and_soul
                        && template.data.effects.quick_innovation_available() == quick_innovation
                })
                .copied()
                .collect();
            // 2 * durability_cost is the minimum CP a state must have to not be considered "final".
            // See `ReducedState::is_final` for details.
            for cp in (2 * self.durability_cost..=self.settings.max_cp()).step_by(2) {
                if self.interrupt_signal.is_set() {
                    return Err(SolverException::Interrupted);
                }
                let solved_states = templates
                    .par_iter_mut()
                    .filter_map(|template| template.instantiate(cp).map(|state| (template, state)))
                    .map(|(template, state)| -> Result<_, SolverException> {
                        let pareto_front = self.solve_precompute_state(state)?;
                        let template_is_maximal = {
                            // A template is "maximal" if there is no benefit of solving it with higher CP
                            let required_progress = self.settings.max_progress();
                            let required_quality = self.settings.max_quality().saturating_sub(
                                self.iq_quality_lut[usize::from(state.effects.inner_quiet())],
                            );
                            if let Some(value) = pareto_front.last() {
                                value.progress >= required_progress
                                    && value.quality >= required_quality
                            } else {
                                return Err(internal_error!(
                                    "Unexpected empty pareto front.",
                                    self.settings,
                                    state
                                ));
                            }
                        };
                        if template_is_maximal {
                            template.required_cp_for_max_progress_and_quality = Some(cp);
                        }
                        Ok((state, pareto_front))
                    })
                    .collect::<Result<Vec<_>, SolverException>>()?;
                self.solved_states.extend(solved_states);
            }
            self.maximal_templates
                .extend(templates.into_iter().filter_map(|template| {
                    template
                        .required_cp_for_max_progress_and_quality
                        .map(|required_cp| (template.data, required_cp))
                }));
        }
        self.precomputed_states = self.solved_states.len();
        log::debug!(
            "QualityUbSolver - templates: {}, precomputed_states: {}",
            all_templates.len(),
            self.solved_states.len()
        );

        Ok(())
    }

    fn solve_precompute_state(
        &self,
        state: ReducedState,
    ) -> Result<Box<[ParetoValue]>, SolverException> {
        let mut pareto_front_builder = ParetoFrontBuilder::new();
        let progress_cutoff = self.settings.max_progress();
        let quality_cutoff = self
            .settings
            .max_quality()
            .saturating_sub(self.iq_quality_lut[usize::from(state.effects.inner_quiet())]);
        for action in FULL_SEARCH_ACTIONS {
            if let Some((new_state, progress, quality)) =
                state.use_action(action, &self.settings, self.durability_cost)
            {
                if !new_state.is_final(self.durability_cost) {
                    if let Some(pareto_front) = self.solved_states.get(&new_state) {
                        pareto_front_builder.push_slice(pareto_front, progress, quality)?;
                    } else {
                        return Err(internal_error!(
                            "Required precompute state does not exist.",
                            self.settings,
                            action,
                            state,
                            new_state
                        ));
                    }
                } else if progress != 0 {
                    pareto_front_builder.push(progress, quality);
                }
            }
        }
        Ok(pareto_front_builder.build(progress_cutoff, quality_cutoff))
    }

    /// Returns an upper-bound on the maximum Quality achievable from this state while also maxing out Progress.
    /// There is no guarantee on the tightness of the upper-bound.
    pub fn quality_upper_bound(
        &mut self,
        mut state: SimulationState,
    ) -> Result<u32, SolverException> {
        if state.effects.combo() != Combo::None {
            return Err(internal_error!(
                "Unexpected combo state.",
                self.settings,
                state
            ));
        }

        let mut required_progress = self.settings.max_progress() - state.progress;
        if state.effects.muscle_memory() != 0 {
            // Assume MuscleMemory can be used to its max potential and remove the effect to reduce the number of states that need to be solved.
            required_progress = required_progress.saturating_sub(self.largest_progress_increase);
            state.effects.set_muscle_memory(0);
        }

        let reduced_state = ReducedState::from_state(state, &self.settings, self.durability_cost);

        let template_data = TemplateData::new(
            reduced_state.effects,
            reduced_state.compressed_unreliable_quality,
        );
        if let Some(&required_cp) = self.maximal_templates.get(&template_data)
            && reduced_state.cp >= required_cp
        {
            let reduced_state = ReducedState {
                cp: required_cp,
                ..reduced_state
            };
            if let Some(pareto_front) = self.solved_states.get(&reduced_state)
                && let Some(value) = pareto_front.last()
                && value.progress >= required_progress
                && value.quality + state.quality >= self.settings.max_quality()
            {
                return Ok(self.settings.max_quality());
            } else {
                return Err(internal_error!(
                    "Maximal template list is inconsistent with actual solved states.",
                    self.settings,
                    reduced_state
                ));
            }
        }

        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.progress < required_progress);
            let quality = pareto_front
                .get(index)
                .map_or(0, |value| state.quality + value.quality);
            return Ok(std::cmp::min(self.settings.max_quality(), quality));
        }

        self.solve_state(reduced_state)?;

        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.progress < required_progress);
            let quality = pareto_front
                .get(index)
                .map_or(0, |value| state.quality + value.quality);
            Ok(std::cmp::min(self.settings.max_quality(), quality))
        } else {
            Err(internal_error!(
                "State not found in memoization table after solve.",
                self.settings,
                reduced_state
            ))
        }
    }

    fn solve_state(&mut self, state: ReducedState) -> Result<(), SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }

        let progress_cutoff = self.settings.max_progress();
        let quality_cutoff = self
            .settings
            .max_quality()
            .saturating_sub(self.iq_quality_lut[usize::from(state.effects.inner_quiet())]);

        let child_states = FULL_SEARCH_ACTIONS
            .iter()
            .filter_map(|&action| state.use_action(action, &self.settings, self.durability_cost))
            .collect::<Vec<_>>();

        for (child_state, action_progress, action_quality) in &child_states {
            if !child_state.is_final(self.durability_cost)
                && !self.solved_states.contains_key(child_state)
            {
                self.solve_state(*child_state)?;
                let child_pareto_front = self.solved_states.get(&child_state).ok_or(
                    internal_error!("State not found in memoization table after solving.",),
                )?;
                let is_maximal = |value: &ParetoValue| {
                    value.progress + action_progress >= progress_cutoff
                        && value.quality + action_quality >= quality_cutoff
                };
                if child_pareto_front.iter().any(is_maximal) {
                    self.solved_states.insert(
                        state,
                        [ParetoValue::new(progress_cutoff, quality_cutoff)].into(),
                    );
                    return Ok(());
                }
            }
        }

        let mut pareto_front_builder = ParetoFrontBuilder::new();
        for (child_state, action_progress, action_quality) in child_states {
            if !child_state.is_final(self.durability_cost) {
                let child_pareto_front = self.solved_states.get(&child_state).ok_or(
                    internal_error!("State not found in memoization table after solving.",),
                )?;
                pareto_front_builder.push_slice(
                    child_pareto_front,
                    action_progress,
                    action_quality,
                )?;
            } else if action_progress != 0 {
                pareto_front_builder.push(action_progress, action_quality);
            }
        }

        let pareto_front = pareto_front_builder.build(progress_cutoff, quality_cutoff);
        self.solved_states.insert(state, pareto_front);
        Ok(())
    }

    pub fn runtime_stats(&self) -> QualityUbSolverStats {
        QualityUbSolverStats {
            parallel_states: self.precomputed_states,
            sequential_states: self.solved_states.len() - self.precomputed_states,
            pareto_values: self.solved_states.values().map(|value| value.len()).sum(),
        }
    }
}

impl Drop for QualityUbSolver {
    fn drop(&mut self) {
        let runtime_stats = self.runtime_stats();
        log::debug!(
            "QualityUbSolver - par_states: {}, seq_states: {}, values: {}",
            runtime_stats.parallel_states,
            runtime_stats.sequential_states,
            runtime_stats.pareto_values
        );
    }
}

/// Calculates the CP cost to "magically" restore 5 durability
fn durability_cost(settings: &Settings) -> u16 {
    let mut cost = 100;
    if settings.is_action_allowed::<MasterMend>() {
        let cost_per_five = MasterMend::CP_COST / std::cmp::min(6, settings.max_durability / 5 - 1);
        cost = std::cmp::min(cost, cost_per_five);
    }
    if settings.is_action_allowed::<Manipulation>() {
        let cost_per_five = Manipulation::CP_COST / 8;
        cost = std::cmp::min(cost, cost_per_five);
    }
    if settings.is_action_allowed::<ImmaculateMend>() {
        let cost_per_five = ImmaculateMend::CP_COST / (settings.max_durability / 5 - 1);
        cost = std::cmp::min(cost, cost_per_five);
    }
    cost
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
struct TemplateData {
    effects: Effects,
    compressed_unreliable_quality: u8,
}

impl TemplateData {
    pub fn new(effects: Effects, compressed_unreliable_quality: u8) -> Self {
        Self {
            effects,
            compressed_unreliable_quality,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Template {
    /// The maximum amount of CP the template can be instantiated with.
    ///
    /// The purpose of this limit is to avoid instantiating unreachable states.
    /// For example, if the solve configuration has a max CP of 500, then instantiating a template with Waste Not II at 450 CP is not useful as the instantiated state cannot be reached from the initial state using any action sequence.
    max_instantiated_cp: u16,

    /// Minimum amount of CP required for the instantiated state to reach max Progress and max Quality.
    ///
    /// This also takes into account the minimum existing Quality of the state (e.g. a template with 10 Inner Quiet must already have some Quality, so it's not necessary for the template to reach max Quality on its own).
    required_cp_for_max_progress_and_quality: Option<u16>,

    data: TemplateData,
}

impl Template {
    pub fn new(max_cp: u16, data: TemplateData) -> Self {
        Self {
            max_instantiated_cp: max_cp,
            required_cp_for_max_progress_and_quality: None,
            data,
        }
    }

    pub fn instantiate(&self, cp: u16) -> Option<ReducedState> {
        if cp > self.max_instantiated_cp {
            return None;
        }
        if let Some(max_cp) = self.required_cp_for_max_progress_and_quality
            && cp > max_cp
        {
            return None;
        }
        Some(ReducedState {
            cp,
            compressed_unreliable_quality: self.data.compressed_unreliable_quality,
            effects: self.data.effects,
        })
    }
}
