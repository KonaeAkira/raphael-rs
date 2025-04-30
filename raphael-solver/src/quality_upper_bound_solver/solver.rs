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
    pub states: usize,
    pub pareto_values: usize,
}

pub struct QualityUbSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    pareto_front_builder: ParetoFrontBuilder,
    durability_cost: u16,
}

impl QualityUbSolver {
    pub fn new(mut settings: SolverSettings, interrupt_signal: utils::AtomicFlag) -> Self {
        let durability_cost = durability_cost(&settings.simulator_settings);
        settings.simulator_settings.max_cp += durability_cost * (settings.max_durability() / 5);
        Self {
            settings,
            interrupt_signal,
            solved_states: SolvedStates::default(),
            pareto_front_builder: ParetoFrontBuilder::new(
                settings.max_progress(),
                settings.max_quality(),
            ),
            durability_cost,
        }
    }

    fn generate_precompute_templates(&self) -> Box<[(Template, u16)]> {
        let mut templates = rustc_hash::FxHashMap::<Template, u16>::default();
        let mut queue = std::collections::BinaryHeap::<Node>::default();

        let initial_node = Node {
            template: Template {
                effects: Effects::initial(&self.settings.simulator_settings)
                    .with_trained_perfection_available(false)
                    .with_quick_innovation_available(false)
                    .with_heart_and_soul_available(false)
                    .with_combo(Combo::None),
                compressed_unreliable_quality: 0,
            },
            required_cp: 0,
        };
        queue.push(initial_node);

        while let Some(node) = queue.pop() {
            if templates.contains_key(&node.template) {
                continue;
            }
            templates.insert(node.template, node.required_cp);
            let state = ReducedState {
                cp: self.settings.max_cp(),
                compressed_unreliable_quality: node.template.compressed_unreliable_quality,
                effects: node.template.effects,
            };
            for &action in FULL_SEARCH_ACTIONS {
                if let Some((new_state, _, _)) =
                    state.use_action(action, &self.settings, self.durability_cost)
                {
                    let used_cp = self.settings.max_cp() - new_state.cp;
                    let new_node = Node {
                        template: Template {
                            effects: new_state.effects,
                            compressed_unreliable_quality: new_state.compressed_unreliable_quality,
                        },
                        required_cp: node.required_cp + used_cp,
                    };
                    if !templates.contains_key(&new_node.template) {
                        queue.push(new_node);
                    }
                }
            }
        }

        templates.into_iter().collect()
    }

    pub fn precompute(&mut self, precompute_cp: u16) {
        if !self.solved_states.is_empty() || rayon::current_num_threads() <= 1 {
            return;
        }

        let templates = self.generate_precompute_templates();
        for cp in self.durability_cost..=precompute_cp {
            if self.interrupt_signal.is_set() {
                return;
            }
            let init = || {
                ParetoFrontBuilder::new(self.settings.max_progress(), self.settings.max_quality())
            };
            let missing_cp = self.settings.max_cp() - cp;
            let solved_states = templates
                .par_iter()
                .filter_map(|(template, required_cp)| {
                    if missing_cp >= *required_cp {
                        Some(ReducedState {
                            cp: cp + self.durability_cost,
                            compressed_unreliable_quality: template.compressed_unreliable_quality,
                            effects: template.effects,
                        })
                    } else {
                        None
                    }
                })
                .map_init(init, |pareto_front_builder, state| {
                    let state = ReducedState { cp, ..state };
                    let pareto_front = self.solve_precompute_state(pareto_front_builder, state);
                    (state, pareto_front)
                })
                .collect_vec_list();
            for thread_solved_states in solved_states {
                self.solved_states.extend(thread_solved_states);
            }
        }

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

    /// Returns an upper-bound on the maximum Quality achievable from this state while also maxing out Progress.
    /// There is no guarantee on the tightness of the upper-bound.
    pub fn quality_upper_bound(&mut self, state: SimulationState) -> Result<u32, SolverException> {
        if state.effects.combo() != Combo::None {
            return Err(SolverException::InternalError(format!(
                "\"{:?}\" combo in quality upper bound solver",
                state.effects.combo()
            )));
        }

        let reduced_state =
            ReducedState::from_simulation_state(state, &self.settings, self.durability_cost);
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
            states: self.solved_states.len(),
            pareto_values: self.solved_states.values().map(|value| value.len()).sum(),
        }
    }
}

impl Drop for QualityUbSolver {
    fn drop(&mut self) {
        let runtime_stats = self.runtime_stats();
        log::debug!(
            "QualityUbSolver - states: {}, values: {}",
            runtime_stats.states,
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

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Template {
    effects: Effects,
    compressed_unreliable_quality: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Node {
    template: Template,
    required_cp: u16,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.required_cp.cmp(&self.required_cp))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.required_cp.cmp(&self.required_cp)
    }
}
