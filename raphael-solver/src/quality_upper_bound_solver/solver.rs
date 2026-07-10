use crate::{
    SolverException, SolverSettings,
    actions::FULL_SEARCH_ACTIONS,
    macros::internal_error,
    utils::{self, ParetoFrontBuilder, ParetoValue},
};

use bump_scope::{BumpPool, BumpPoolGuard};
use raphael_sim::*;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use rustc_hash::FxHashMap;

use super::state::ReducedState;

pub type ParetoFront = nunny::Slice<ParetoValue>;
pub type SolvedStates<'alloc> = FxHashMap<ReducedState, &'alloc ParetoFront>;

#[derive(Default, Debug, Clone, Copy)]
pub struct QualityUbSolverStats {
    pub states_on_main: usize,
    pub states_on_shards: usize,
    pub values: usize,
}

#[derive(Clone)]
struct QualityUbSolverContext<'alloc> {
    allocator: &'alloc BumpPool,
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    iq_quality_lut: [u16; 11],
    durability_cost: u16,
    largest_progress_increase: u16,
}

#[derive(Debug, Clone, Copy)]
struct TemplateRecord {
    slots_start: u32,
    max_instantiated_cp: u16,
    /// Minimum CP at which the template reaches max Progress and max Quality,
    /// or `u16::MAX` if it never does.
    required_cp_for_max: u16,
}

const UNSOLVED_SLOT: u32 = u32::MAX;

const ARENA_SEGMENT_BITS: u32 = 20;
const ARENA_SEGMENT_LEN: usize = 1 << ARENA_SEGMENT_BITS;

/// Append-only storage for Pareto fronts
#[derive(Default)]
struct ValueArena {
    segments: Vec<Vec<ParetoValue>>,
}

impl ValueArena {
    fn push_front(&mut self, front: &[ParetoValue]) -> Option<u32> {
        let needed = front.len() + 1;
        debug_assert!(needed <= ARENA_SEGMENT_LEN);
        if self
            .segments
            .last()
            .is_none_or(|segment| ARENA_SEGMENT_LEN - segment.len() < needed)
        {
            self.segments.push(Vec::with_capacity(ARENA_SEGMENT_LEN));
        }
        let segment_index = self.segments.len() - 1;
        let segment = self.segments.last_mut().unwrap();
        let offset = u32::try_from(
            (segment_index as u64) << ARENA_SEGMENT_BITS | segment.len() as u64,
        )
        .ok()
        .filter(|&offset| offset != UNSOLVED_SLOT)?;
        segment.push(ParetoValue::new(front.len() as u16, 0));
        segment.extend_from_slice(front);
        Some(offset)
    }

    fn resolve(&self, offset: u32) -> &[ParetoValue] {
        let segment = &self.segments[(offset >> ARENA_SEGMENT_BITS) as usize];
        let index = (offset as usize) & (ARENA_SEGMENT_LEN - 1);
        let len = usize::from(segment[index].progress);
        &segment[index + 1..index + 1 + len]
    }
}

pub struct QualityUbSolver<'alloc> {
    context: QualityUbSolverContext<'alloc>,
    templates: FxHashMap<TemplateData, TemplateRecord>,
    /// indexed by `(cp - min_solved_cp) / 2`
    slots: Vec<u32>,
    arena: ValueArena,
    /// States solved by shards during the search and merged back
    overflow: SolvedStates<'alloc>,
    states_on_main: usize,
    values_on_main: usize,
    num_states_solved_on_shards: usize,
}

pub struct QualityUbSolverShard<'main, 'alloc> {
    context: &'main QualityUbSolverContext<'alloc>,
    templates: &'main FxHashMap<TemplateData, TemplateRecord>,
    slots: &'main [u32],
    arena: &'main ValueArena,
    overflow: &'main SolvedStates<'alloc>,
    local_states: SolvedStates<'alloc>,
}

impl<'alloc> QualityUbSolver<'alloc> {
    pub fn new(
        mut settings: SolverSettings,
        interrupt_signal: utils::AtomicFlag,
        allocator: &'alloc BumpPool,
    ) -> Self {
        let durability_cost = durability_cost(&settings.simulator_settings);
        settings.simulator_settings.max_cp = {
            let initial_state = SimulationState::new(&settings.simulator_settings);
            ReducedState::from_state(initial_state, &settings, durability_cost).cp
        };
        Self {
            context: QualityUbSolverContext {
                allocator,
                settings,
                interrupt_signal,
                iq_quality_lut: utils::compute_iq_quality_lut(&settings),
                durability_cost,
                largest_progress_increase: utils::maximum_muscle_memory_utilization(
                    &settings.simulator_settings,
                ),
            },
            templates: FxHashMap::default(),
            slots: Vec::new(),
            arena: ValueArena::default(),
            overflow: SolvedStates::default(),
            states_on_main: 0,
            values_on_main: 0,
            num_states_solved_on_shards: 0,
        }
    }

    pub fn extend_solved_states(&mut self, new_solved_states: SolvedStates<'alloc>) {
        let len_before = self.overflow.len();
        self.overflow.extend(new_solved_states);
        let len_after = self.overflow.len();
        self.num_states_solved_on_shards += len_after - len_before;
    }

    pub fn create_shard<'main>(&'main self) -> QualityUbSolverShard<'main, 'alloc> {
        QualityUbSolverShard {
            context: &self.context,
            templates: &self.templates,
            slots: &self.slots,
            arena: &self.arena,
            overflow: &self.overflow,
            local_states: SolvedStates::default(),
        }
    }

    fn generate_precompute_templates(&self) -> Box<[Template]> {
        let mut templates = rustc_hash::FxHashMap::<TemplateData, u16>::default();
        let mut heap = std::collections::BinaryHeap::<Template>::default();

        let seed_template = {
            let seed_effects = Effects::initial(&self.context.settings.simulator_settings)
                .with_special_quality_state(SpecialQualityState::Normal)
                .with_trained_perfection_available(false)
                .with_combo(Combo::None);
            Template::new(
                self.context.settings.max_cp(),
                TemplateData::new(seed_effects, 0),
            )
        };
        heap.push(seed_template);

        while let Some(template) = heap.pop() {
            let entry = templates.entry(template.data).or_default();
            if template.max_instantiated_cp > *entry {
                *entry = template.max_instantiated_cp;
                let state = template.instantiate(template.max_instantiated_cp).unwrap();
                for action in FULL_SEARCH_ACTIONS {
                    if let Some((new_state, _, _)) = state.use_action(
                        action,
                        &self.context.settings,
                        self.context.durability_cost,
                        self.context.largest_progress_increase,
                    ) {
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

    fn min_solved_cp(&self) -> u16 {
        2 * self.context.durability_cost
    }

    fn lookup_slot(&self, state: &ReducedState) -> Option<&ParetoFront> {
        lookup_slot(
            &self.templates,
            &self.slots,
            &self.arena,
            self.min_solved_cp(),
            state,
        )
    }

    pub fn precompute(&mut self) -> Result<(), SolverException> {
        let min_solved_cp = self.min_solved_cp();
        let mut all_templates = self.generate_precompute_templates();

        // Lay out each template's CP range as a contiguous span
        let mut total_slots: usize = 0;
        for template in &mut all_templates {
            template.slots_start = total_slots as u32;
            if template.max_instantiated_cp >= min_solved_cp {
                total_slots +=
                    usize::from((template.max_instantiated_cp - min_solved_cp) / 2) + 1;
            }
        }
        self.slots = vec![UNSOLVED_SLOT; total_slots];
        self.templates = all_templates
            .iter()
            .map(|template| {
                (
                    template.data,
                    TemplateRecord {
                        slots_start: template.slots_start,
                        max_instantiated_cp: template.max_instantiated_cp,
                        required_cp_for_max: u16::MAX,
                    },
                )
            })
            .collect();

        // States are computed in order of less CP to more CP.
        // States currently being computed assume that child states have already been computed.
        // This is the reason why states with HeartAndSoul and QuickInnovation available must be computed separately.
        // HeartAndSoul enables the use of TricksOfTrade, which restores CP.
        // QuickInnovation requires no CP (and no durability, so durability cost in terms of CP is 0).
        for (heart_and_soul, quick_innovation) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            for stellar_steady_hand in 0..3 {
                let mut templates: Vec<_> = all_templates
                    .iter()
                    .filter(|template| {
                        template.data.effects.heart_and_soul_available() == heart_and_soul
                            && template.data.effects.quick_innovation_available()
                                == quick_innovation
                            && template.data.effects.stellar_steady_hand_charges()
                                == stellar_steady_hand
                    })
                    .copied()
                    .collect();
                // 2 * durability_cost is the minimum CP a state must have to not be considered "final".
                // See `ReducedState::is_final` for details.
                for cp in (min_solved_cp..=self.context.settings.max_cp()).step_by(2) {
                    if self.context.interrupt_signal.is_set() {
                        return Err(SolverException::Interrupted);
                    }
                    let solved_states = templates
                        .par_iter_mut()
                        .filter_map(|template| {
                            template.instantiate(cp).map(|state| (template, state))
                        })
                        .map_init(
                            ParetoFrontBuilder::new,
                            |pf_builder, (template, state)| -> Result<_, SolverException> {
                                let pareto_front =
                                    self.solve_precompute_state(pf_builder, state)?;
                                let template_is_maximal = {
                                    // A template is "maximal" if there is no benefit of solving it with higher CP
                                    let required_progress = self.context.settings.max_progress();
                                    let required_quality =
                                        self.context.settings.max_quality().saturating_sub(
                                            self.context.iq_quality_lut
                                                [usize::from(state.effects.inner_quiet())],
                                        );
                                    pareto_front[0].progress >= required_progress
                                        && pareto_front[0].quality >= required_quality
                                };
                                if template_is_maximal {
                                    template.required_cp_for_max_progress_and_quality = Some(cp);
                                }
                                let slot = template.slots_start as usize
                                    + usize::from((cp - min_solved_cp) / 2);
                                Ok((slot, pareto_front))
                            },
                        )
                        .collect::<Result<Vec<_>, SolverException>>()?;
                    self.states_on_main += solved_states.len();
                    for (slot, pareto_front) in solved_states {
                        let offset =
                            self.arena.push_front(&pareto_front).ok_or_else(|| {
                                internal_error!(
                                    "QualityUbSolver value arena exceeds u32 offsets.",
                                    self.context.settings
                                )
                            })?;
                        self.slots[slot] = offset;
                        self.values_on_main += pareto_front.len();
                    }
                }
                for template in templates {
                    if let Some(required_cp) = template.required_cp_for_max_progress_and_quality
                    {
                        self.templates
                            .get_mut(&template.data)
                            .unwrap()
                            .required_cp_for_max = required_cp;
                    }
                }
            }
        }
        Ok(())
    }

    fn solve_precompute_state(
        &self,
        pf_builder: &mut ParetoFrontBuilder,
        state: ReducedState,
    ) -> Result<Vec<ParetoValue>, SolverException> {
        let cutoff = ParetoValue::new(
            self.context.settings.max_progress(),
            self.context.settings.max_quality().saturating_sub(
                self.context.iq_quality_lut[usize::from(state.effects.inner_quiet())],
            ),
        );
        pf_builder.initialize_with_cutoff(cutoff);
        for action in FULL_SEARCH_ACTIONS {
            if let Some((new_state, progress, quality)) = state.use_action(
                action,
                &self.context.settings,
                self.context.durability_cost,
                self.context.largest_progress_increase,
            ) {
                let action_value = ParetoValue::new(progress, quality);
                if !new_state.is_final(self.context.durability_cost) {
                    if let Some(pareto_front) = self.lookup_slot(&new_state) {
                        pf_builder.push_slice(
                            pareto_front
                                .iter()
                                .map(|value| value.saturating_add(action_value)),
                        );
                    } else {
                        return Err(internal_error!(
                            "Required precompute state does not exist.",
                            self.context.settings,
                            action,
                            state,
                            new_state
                        ));
                    }
                } else if progress != 0 {
                    pf_builder.push(action_value);
                }
                if pf_builder.is_maximal(cutoff) {
                    break;
                }
            }
        }
        let pareto_front = pf_builder.result_as_slice();
        if pareto_front.is_empty() {
            return Err(internal_error!(
                "Empty precompute Pareto front.",
                self.context.settings,
                state
            ));
        }
        Ok(pareto_front.to_vec())
    }

    pub fn runtime_stats(&self) -> QualityUbSolverStats {
        QualityUbSolverStats {
            states_on_main: self.states_on_main,
            states_on_shards: self.num_states_solved_on_shards,
            values: self.values_on_main
                + self
                    .overflow
                    .values()
                    .map(|front| front.len())
                    .sum::<usize>(),
        }
    }
}

impl<'main, 'alloc> QualityUbSolverShard<'main, 'alloc> {
    pub fn solved_states(self) -> SolvedStates<'alloc> {
        self.local_states
    }

    fn lookup_shared(&self, state: &ReducedState) -> Option<&'main ParetoFront> {
        lookup_slot(
            self.templates,
            self.slots,
            self.arena,
            2 * self.context.durability_cost,
            state,
        )
        .or_else(|| self.overflow.get(state).copied())
    }

    pub fn quality_upper_bound(
        &mut self,
        mut state: SimulationState,
    ) -> Result<u16, SolverException> {
        let mut required_progress = self.context.settings.max_progress() - state.progress;
        if state.effects.muscle_memory() != 0 {
            // Assume MuscleMemory can be used to its max potential and remove the effect to reduce the number of states that need to be solved.
            required_progress =
                required_progress.saturating_sub(self.context.largest_progress_increase);
            state.effects.set_muscle_memory(0);
        }

        let reduced_state =
            ReducedState::from_state(state, &self.context.settings, self.context.durability_cost);

        let template_data = TemplateData::new(
            reduced_state.effects,
            reduced_state.compressed_unreliable_quality,
        );
        if let Some(record) = self.templates.get(&template_data)
            && record.required_cp_for_max != u16::MAX
            && reduced_state.cp >= record.required_cp_for_max
        {
            let reduced_state = ReducedState {
                cp: record.required_cp_for_max,
                ..reduced_state
            };
            if let Some(pareto_front) = self.lookup_shared(&reduced_state)
                && pareto_front.first().progress >= required_progress
                && pareto_front.first().quality.saturating_add(state.quality)
                    >= self.context.settings.max_quality()
            {
                return Ok(self.context.settings.max_quality());
            } else {
                return Err(internal_error!(
                    "Maximal template list is inconsistent with actual solved states.",
                    self.context.settings,
                    reduced_state
                ));
            }
        }

        let pareto_front =
            if let Some(pareto_front) = self.lookup_shared(&reduced_state) {
                pareto_front
            } else if let Some(pareto_front) = self.local_states.get(&reduced_state).copied() {
                pareto_front
            } else {
                let allocator = self.context.allocator.get();
                self.solve_state(reduced_state, &allocator)?;
                if let Some(pareto_front) = self.local_states.get(&reduced_state).copied() {
                    pareto_front
                } else {
                    return Err(internal_error!(
                        "State not found in memoization table after solve.",
                        self.context.settings,
                        reduced_state
                    ));
                }
            };
        let i = pareto_front.partition_point(|value| value.progress < required_progress);
        let quality = pareto_front
            .get(i)
            .map_or(0, |value| state.quality.saturating_add(value.quality));
        Ok(std::cmp::min(self.context.settings.max_quality(), quality))
    }

    fn solve_state(
        &mut self,
        state: ReducedState,
        allocator: &BumpPoolGuard<'alloc>,
    ) -> Result<(), SolverException> {
        if self.context.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }

        let cutoff = ParetoValue::new(
            self.context.settings.max_progress(),
            self.context.settings.max_quality().saturating_sub(
                self.context.iq_quality_lut[usize::from(state.effects.inner_quiet())],
            ),
        );
        let mut pareto_front_builder = ParetoFrontBuilder::new();
        pareto_front_builder.initialize_with_cutoff(cutoff);

        for action in FULL_SEARCH_ACTIONS {
            if let Some((child_state, progress, quality)) = state.use_action(
                action,
                &self.context.settings,
                self.context.durability_cost,
                self.context.largest_progress_increase,
            ) {
                let action_value = ParetoValue::new(progress, quality);
                if !child_state.is_final(self.context.durability_cost) {
                    let child_pareto_front = if let Some(child_pareto_front) =
                        self.lookup_shared(&child_state)
                    {
                        child_pareto_front
                    } else if let Some(child_pareto_front) =
                        self.local_states.get(&child_state).copied()
                    {
                        child_pareto_front
                    } else {
                        self.solve_state(child_state, allocator)?;
                        self.local_states
                            .get(&child_state)
                            .copied()
                            .ok_or_else(|| {
                                internal_error!(
                                    "State not found in memoization table after solving.",
                                    self.context.settings,
                                    child_state
                                )
                            })?
                    };
                    pareto_front_builder.push_slice(
                        child_pareto_front
                            .iter()
                            .map(|value| value.saturating_add(action_value)),
                    );
                    if pareto_front_builder.is_maximal(cutoff) {
                        break;
                    }
                } else if action_value.progress != 0 {
                    pareto_front_builder.push(action_value);
                }
            }
        }
        let pareto_front = allocator
            .alloc_slice_copy(pareto_front_builder.result_as_slice())
            .into_ref();
        let pareto_front = pareto_front.try_into().map_err(|_| {
            internal_error!(
                "Solver produced empty Pareto front.",
                self.context.settings,
                state
            )
        })?;
        self.local_states.insert(state, pareto_front);
        Ok(())
    }
}

fn lookup_slot<'arena>(
    templates: &FxHashMap<TemplateData, TemplateRecord>,
    slots: &[u32],
    arena: &'arena ValueArena,
    min_solved_cp: u16,
    state: &ReducedState,
) -> Option<&'arena ParetoFront> {
    let data = TemplateData::new(state.effects, state.compressed_unreliable_quality);
    let record = templates.get(&data)?;
    if state.cp < min_solved_cp || state.cp > record.max_instantiated_cp {
        return None;
    }
    let index = record.slots_start as usize + usize::from((state.cp - min_solved_cp) / 2);
    let offset = slots[index];
    if offset == UNSOLVED_SLOT {
        return None;
    }
    let front = arena.resolve(offset);
    Some(front.try_into().expect("arena Pareto fronts are never empty"))
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

    /// Start of this template's span in `QualityUbSolver::slots`.
    /// Assigned at the start of `QualityUbSolver::precompute`.
    slots_start: u32,
}

impl Template {
    pub fn new(max_cp: u16, data: TemplateData) -> Self {
        Self {
            max_instantiated_cp: max_cp,
            required_cp_for_max_progress_and_quality: None,
            data,
            slots_start: 0,
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
