use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS},
    utils,
};
use raphael_sim::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::state::ReducedState;

type ParetoValue = utils::ParetoValue<u32, u32>;
type ParetoFrontBuilder = utils::ParetoFrontBuilder<u32, u32>;
type SolvedStates = rustc_hash::FxHashMap<ReducedState, Box<[ParetoValue]>>;

#[derive(Debug, Clone, Copy)]
pub struct QualityUbSolverStats {
    pub parallel_states: usize,
    pub sequential_states: usize,
    pub pareto_values: usize,
}

pub struct QualityUbSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    pareto_front_builder: ParetoFrontBuilder,
    durability_cost: u16,
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
            solved_states: SolvedStates::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.max_progress(),
                settings.max_quality(),
            ),
            durability_cost,
            precomputed_states: 0,
        }
    }

    fn generate_precompute_templates(&self) -> Box<[Template]> {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        struct TemplateData {
            effects: Effects,
            compressed_unreliable_quality: u8,
        }

        let mut templates = rustc_hash::FxHashMap::<TemplateData, u16>::default();
        let mut heap = std::collections::BinaryHeap::<Template>::default();

        let seed_template = {
            let seed_effects = Effects::initial(&self.settings.simulator_settings)
                .with_trained_perfection_available(false)
                .with_combo(Combo::None);
            Template {
                max_cp: self.settings.max_cp(),
                effects: seed_effects,
                compressed_unreliable_quality: 0,
            }
        };
        heap.push(seed_template);

        while let Some(template) = heap.pop() {
            let template_data = TemplateData {
                effects: template.effects,
                compressed_unreliable_quality: template.compressed_unreliable_quality,
            };
            let entry = templates.entry(template_data).or_default();
            if template.max_cp > *entry {
                *entry = template.max_cp;
                let state = template.instantiate(template.max_cp).unwrap();
                for &action in FULL_SEARCH_ACTIONS {
                    if let Some((new_state, _, _)) =
                        state.use_action(action, &self.settings, self.durability_cost)
                    {
                        let new_template_data = TemplateData {
                            effects: new_state.effects,
                            compressed_unreliable_quality: new_state.compressed_unreliable_quality,
                        };
                        let new_template = Template {
                            max_cp: new_state.cp,
                            effects: new_state.effects,
                            compressed_unreliable_quality: new_state.compressed_unreliable_quality,
                        };
                        let new_entry = templates.entry(new_template_data).or_default();
                        if new_template.max_cp > *new_entry {
                            heap.push(new_template);
                        }
                    }
                }
            }
        }

        templates
            .into_iter()
            .map(|(template_data, max_cp)| Template {
                max_cp,
                effects: template_data.effects,
                compressed_unreliable_quality: template_data.compressed_unreliable_quality,
            })
            .collect()
    }

    pub fn precompute(&mut self) {
        assert!(self.solved_states.is_empty());
        let templates = self.generate_precompute_templates();
        // States are computed in order of less CP to more CP.
        // States currently being computed assume that child states have already been computed.
        // This is the reason why states with HeartAndSoul and QuickInnovation available must be computed separately.
        // HeartAndSoul enables the use of TricksOfTrade, which restores CP.
        // QuickInnovation requires no CP (and no durability, so durability cost in terms of CP is 0).
        for (heart_and_soul, quick_innovation) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            let filtered_templates: Vec<_> = templates
                .iter()
                .filter(|template| {
                    template.effects.heart_and_soul_available() == heart_and_soul
                        && template.effects.quick_innovation_available() == quick_innovation
                })
                .collect();
            // 2 * durability_cost is the minimum CP a state must have to not be considered "final".
            // See `ReducedState::is_final` for details.
            for cp in 2 * self.durability_cost..=self.settings.max_cp() {
                if self.interrupt_signal.is_set() {
                    return;
                }
                let solved_states = filtered_templates
                    .par_iter()
                    .filter_map(|template| template.instantiate(cp))
                    .map_init(
                        || {
                            ParetoFrontBuilder::new(
                                self.settings.max_progress(),
                                self.settings.max_quality(),
                            )
                        },
                        |pf_builder, state| (state, self.solve_precompute_state(pf_builder, state)),
                    )
                    .collect_vec_list();
                self.solved_states
                    .extend(solved_states.into_iter().flatten());
            }
        }
        self.precomputed_states = self.solved_states.len();
        log::debug!(
            "QualityUbSolver - templates: {}, precomputed_states: {}",
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
        for &action in FULL_SEARCH_ACTIONS {
            if let Some((new_state, progress, quality)) =
                state.use_action(action, &self.settings, self.durability_cost)
            {
                if !new_state.is_final(self.durability_cost) {
                    if let Some(pareto_front) = self.solved_states.get(&new_state) {
                        pareto_front_builder.push_slice(pareto_front);
                    } else {
                        unreachable!(
                            "Precompute state does not exist.\nParent: {state:?}\nChild: {new_state:?}\nAction: {action:?}"
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

    /// Returns an upper-bound on the maximum Quality achievable from this state while also maxing out Progress.
    /// There is no guarantee on the tightness of the upper-bound.
    pub fn quality_upper_bound(&mut self, state: SimulationState) -> Result<u32, SolverException> {
        if state.effects.combo() != Combo::None {
            return Err(SolverException::InternalError(format!(
                "\"{:?}\" combo in quality upper bound solver",
                state.effects.combo()
            )));
        }

        let reduced_state = ReducedState::from_state(state, &self.settings, self.durability_cost);
        let required_progress = self.settings.max_progress() - state.progress;

        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality = pareto_front
                .get(index)
                .map_or(0, |value| state.quality + value.second);
            return Ok(std::cmp::min(self.settings.max_quality(), quality));
        }

        self.pareto_front_builder.clear();
        self.solve_state(reduced_state)?;

        if let Some(pareto_front) = self.solved_states.get(&reduced_state) {
            let index = pareto_front.partition_point(|value| value.first < required_progress);
            let quality = pareto_front
                .get(index)
                .map_or(0, |value| state.quality + value.second);
            Ok(std::cmp::min(self.settings.max_quality(), quality))
        } else {
            unreachable!("State must be in memoization table after solver")
        }
    }

    fn solve_state(&mut self, state: ReducedState) -> Result<(), SolverException> {
        if self.interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        self.pareto_front_builder.push_empty();
        let search_actions = match state.effects.allow_quality_actions() {
            false => PROGRESS_ONLY_SEARCH_ACTIONS,
            true => FULL_SEARCH_ACTIONS,
        };
        for &action in search_actions {
            self.build_child_front(state, action)?;
            if self.pareto_front_builder.is_max() {
                // stop early if both Progress and Quality are maxed out
                // this optimization would work even better with better action ordering
                // (i.e. if better actions are visited first)
                break;
            }
        }
        let pareto_front = Box::from(self.pareto_front_builder.peek().unwrap());
        self.solved_states.insert(state, pareto_front);
        Ok(())
    }

    #[inline(always)]
    fn build_child_front(
        &mut self,
        state: ReducedState,
        action: ActionCombo,
    ) -> Result<(), SolverException> {
        if let Some((new_state, progress, quality)) =
            state.use_action(action, &self.settings, self.durability_cost)
        {
            if !new_state.is_final(self.durability_cost) {
                if let Some(pareto_front) = self.solved_states.get(&new_state) {
                    self.pareto_front_builder.push_slice(pareto_front);
                } else {
                    self.solve_state(new_state)?;
                }
                self.pareto_front_builder
                    .peek_mut()
                    .unwrap()
                    .iter_mut()
                    .for_each(|value| {
                        value.first += progress;
                        value.second += quality;
                    });
                self.pareto_front_builder.merge();
            } else if progress != 0 {
                // last action must be a progress increase
                self.pareto_front_builder
                    .push_slice(&[ParetoValue::new(progress, quality)]);
                self.pareto_front_builder.merge();
            }
        }
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Template {
    max_cp: u16, // The template cannot be instantiated with CP above this value
    effects: Effects,
    compressed_unreliable_quality: u8,
}

impl Template {
    pub fn instantiate(&self, cp: u16) -> Option<ReducedState> {
        if cp > self.max_cp || !cp.is_multiple_of(2) {
            return None;
        }
        Some(ReducedState {
            cp,
            compressed_unreliable_quality: self.compressed_unreliable_quality,
            effects: self.effects,
        })
    }
}
