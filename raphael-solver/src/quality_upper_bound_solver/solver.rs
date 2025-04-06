use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, FULL_SEARCH_ACTIONS, PROGRESS_ONLY_SEARCH_ACTIONS},
    utils,
};
use itertools::{Itertools, iproduct};
use raphael_sim::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use super::state::ReducedState;

type ParetoValue = utils::ParetoValue<u32, u32>;
type ParetoFrontBuilder = utils::ParetoFrontBuilder<u32, u32>;
type SolvedStates = rustc_hash::FxHashMap<ReducedState, Box<[ParetoValue]>>;

pub struct QualityUpperBoundSolver {
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,
    solved_states: SolvedStates,
    pareto_front_builder: ParetoFrontBuilder,
    durability_cost: i16,
}

impl QualityUpperBoundSolver {
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

    pub fn precompute(&mut self, precompute_cp: i16) {
        if !self.solved_states.is_empty() || rayon::current_num_threads() <= 1 {
            return;
        }
        let cost_per_inner_quiet_tick = minimum_cp_cost_per_inner_quiet_tick(
            &self.settings.simulator_settings,
            self.durability_cost,
        );
        let templates = templates_for_precompute(&self.settings.simulator_settings);
        for cp in self.durability_cost..=precompute_cp {
            if self.interrupt_signal.is_set() {
                return;
            }
            let init = || {
                ParetoFrontBuilder::new(self.settings.max_progress(), self.settings.max_quality())
            };
            let solved_states = templates
                .par_iter()
                .filter(|state| {
                    let missing_cp = self.settings.max_cp() - cp;
                    minimum_cp_cost(state, cost_per_inner_quiet_tick) <= missing_cp
                })
                .map_init(init, |pareto_front_builder, &state| {
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
            "QualityUpperBoundSolver - templates: {}, precomputed_states: {}",
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
            if let Ok((new_state, progress, quality)) =
                state.use_action(action, &self.settings, self.durability_cost)
            {
                if new_state.cp >= self.durability_cost {
                    if let Some(pareto_front) = self.solved_states.get(&new_state) {
                        pareto_front_builder.push_slice(pareto_front);
                    } else {
                        log::error!("Parent: {state:?}");
                        log::error!("Child: {new_state:?}");
                        log::error!("Action: {action:?}");
                        unreachable!("Precompute child state {new_state:?} does not exist.");
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
                } else if new_state.cp >= -self.durability_cost && progress != 0 {
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
        let search_actions = match state.progress_only {
            true => PROGRESS_ONLY_SEARCH_ACTIONS,
            false => FULL_SEARCH_ACTIONS,
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
        if let Ok((new_state, progress, quality)) =
            state.use_action(action, &self.settings, self.durability_cost)
        {
            if new_state.cp >= self.durability_cost {
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
            } else if new_state.cp >= -self.durability_cost && progress != 0 {
                // "durability" must not go lower than -5
                // last action must be a progress increase
                self.pareto_front_builder
                    .push_slice(&[ParetoValue::new(progress, quality)]);
                self.pareto_front_builder.merge();
            }
        }
        Ok(())
    }
}

impl Drop for QualityUpperBoundSolver {
    fn drop(&mut self) {
        let num_states = self.solved_states.len();
        let num_values = self
            .solved_states
            .values()
            .map(|value| value.len())
            .sum::<usize>();
        log::debug!("QualityUpperBoundSolver - states: {num_states}, values: {num_values}");
    }
}

/// Calculates the CP cost to "magically" restore 5 durability
fn durability_cost(settings: &Settings) -> i16 {
    let mut cost = 20;
    if settings.is_action_allowed::<MasterMend>() {
        let cost_per_five = MasterMend::CP_COST / 6;
        cost = std::cmp::min(cost, cost_per_five);
    }
    if settings.is_action_allowed::<Manipulation>() {
        let cost_per_five = Manipulation::CP_COST / 8;
        cost = std::cmp::min(cost, cost_per_five);
    }
    if settings.is_action_allowed::<ImmaculateMend>() {
        let cost_per_five = ImmaculateMend::CP_COST / (settings.max_durability as i16 / 5 - 1);
        cost = std::cmp::min(cost, cost_per_five);
    }
    cost
}

fn minimum_cp_cost_per_inner_quiet_tick(settings: &Settings, durability_cost: i16) -> i16 {
    let mut cost_per_tick = BasicTouch::CP_COST + durability_cost;
    if settings.is_action_allowed::<PreparatoryTouch>() {
        cost_per_tick = std::cmp::min(
            cost_per_tick,
            (PreparatoryTouch::CP_COST + 2 * durability_cost) / 2,
        );
    }
    if settings.is_action_allowed::<RefinedTouch>() {
        cost_per_tick = std::cmp::min(
            cost_per_tick,
            (Observe::CP_COST + RefinedTouch::CP_COST + durability_cost) / 2,
        );
        cost_per_tick = std::cmp::min(
            cost_per_tick,
            (BasicTouch::CP_COST + RefinedTouch::CP_COST + 2 * durability_cost) / 3,
        );
    }
    cost_per_tick
}

/// Calculates a lower bound of the minimum CP cost to reach the given state from the initial state.
fn minimum_cp_cost(state: &ReducedState, cost_per_inner_quiet_tick: i16) -> i16 {
    let mut result = 0;
    // InnerQuiet effect
    result += state.effects.inner_quiet() as i16 * cost_per_inner_quiet_tick;
    // WasteNot effect
    if state.effects.waste_not() > 4 {
        result += WasteNot2::CP_COST;
    } else if state.effects.waste_not() > 0 {
        result += WasteNot::CP_COST;
    }
    // Innovation effect
    if state.effects.innovation() > 1 {
        // Innovation = 1 could be because of QuickInnovation
        result += Innovation::CP_COST;
    }
    // Veneration effect
    if state.effects.veneration() > 0 {
        result += Veneration::CP_COST;
    }
    // GreatStrides effect
    if state.effects.great_strides() > 0 {
        result += GreatStrides::CP_COST;
    }
    result
}

fn templates_for_precompute(settings: &Settings) -> Box<[ReducedState]> {
    let mut templates = rustc_hash::FxHashSet::<ReducedState>::default();
    let mut add = |effects: Effects, compressed_unreliable_quality: u8| {
        // TODO: add validity checks
        if !settings.adversarial && (compressed_unreliable_quality != 0 || effects.guard() != 0) {
            return;
        }
        if !settings.is_action_allowed::<QuickInnovation>() && effects.quick_innovation_available()
        {
            return;
        }
        let template = ReducedState {
            effects,
            compressed_unreliable_quality,
            progress_only: false,
            cp: 0,
        };
        templates.insert(template);
        if template.has_no_quality_attributes() {
            templates.insert(ReducedState {
                progress_only: true,
                ..template
            });
        }
    };

    for indices in (0..=8).permutations(3) {
        let waste_not = 8u8.saturating_sub(indices[0]);
        let innovation = 4u8.saturating_sub(indices[1]);
        let veneration = 4u8.saturating_sub(indices[2]);
        for (inner_quiet, great_strides) in iproduct!(0..=10, 0..=1) {
            let effects = Effects::new()
                .with_waste_not(waste_not)
                .with_innovation(innovation)
                .with_veneration(veneration)
                .with_inner_quiet(inner_quiet)
                .with_great_strides(great_strides * 3);

            // TODO: handle quick innovation
            // TODO: handle heart and soul

            let max_compressed_unreliable_quality = if settings.adversarial {
                // Maximum quality potency of the last quality-action based on current InnerQuiet
                const MAX_QUALITY_POTENCY: [u32; 11] = [
                    1500, // ByregotsBlessing at 10 InnerQuiet + Innovation + GreatStrides
                    375,  // AdvancedTouch at 0 InnerQuiet + Innovation + GreatStrides
                    500,  // PreparatoryTouch at 0 InnerQuiet + Innovation + GreatStrides
                    550,  // PreparatoryTouch at 1 InnerQuiet + Innovation + GreatStrides
                    600,  // PreparatoryTouch at 2 InnerQuiet + Innovation + GreatStrides
                    650,  // PreparatoryTouch at 3 InnerQuiet + Innovation + GreatStrides
                    700,  // PreparatoryTouch at 4 InnerQuiet + Innovation + GreatStrides
                    750,  // PreparatoryTouch at 5 InnerQuiet + Innovation + GreatStrides
                    800,  // PreparatoryTouch at 6 InnerQuiet + Innovation + GreatStrides
                    850,  // PreparatoryTouch at 7 InnerQuiet + Innovation + GreatStrides
                    1000, // PreparatoryTouch at 10 InnerQuiet + Innovation + GreatStrides
                ];
                // Unreliable quality is at most half of the last action's quality potency.
                // Each point of compressed unreliable quality is equivalent to 200 unreliable quality potency.
                let max_quality_potency = MAX_QUALITY_POTENCY[effects.inner_quiet() as usize];
                max_quality_potency.div_ceil(2 * 200) as u8
            } else {
                0
            };

            for compressed_unreliable_quality in 0..=max_compressed_unreliable_quality {
                add(effects, compressed_unreliable_quality);
                if !indices.contains(&0) {
                    // Use a quality-increasing action as the most recent action
                    if settings.adversarial {
                        add(effects.with_guard(1), compressed_unreliable_quality);
                    }
                }
            }
        }
    }

    templates.into_iter().collect()
}
