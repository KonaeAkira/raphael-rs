use bump_scope::BumpPoolGuard;
use raphael_sim::{
    Action, ActionError, Effects, ImmaculateMend, Manipulation, MasterMend, SimulationState,
    TrainedPerfection, WasteNot, WasteNot2,
};
use rustc_hash::FxHashMap;

use crate::{
    SolverException, SolverSettings,
    actions::{ActionCombo, FULL_SEARCH_ACTIONS, use_action_combo},
    macros::internal_error,
    utils,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    cp: u16,
    unreliable_quality: u16,
    effects: Effects,
}

impl State {
    fn from_simulation_state<'alloc>(
        context: &Context<'alloc>,
        simulation_state: SimulationState,
    ) -> (Self, u16, u16) {
        let manipulation_refund =
            u16::from(simulation_state.effects.manipulation()) * context.manipulation_refund;
        let waste_not_refund =
            u16::from(simulation_state.effects.waste_not()) * context.waste_not_refund;
        let trained_perfection = simulation_state.effects.trained_perfection_active()
            | simulation_state.effects.trained_perfection_available();
        let trained_perfection_refund =
            u16::from(trained_perfection) * context.trained_perfection_refund;
        let state = Self {
            cp: simulation_state.cp
                + manipulation_refund
                + waste_not_refund
                + trained_perfection_refund,
            unreliable_quality: simulation_state.unreliable_quality,
            effects: simulation_state
                .effects
                .with_manipulation(0)
                .with_waste_not(0)
                .with_trained_perfection_active(false)
                .with_trained_perfection_available(false),
        };
        (state, simulation_state.progress, simulation_state.quality)
    }

    fn is_final<'alloc>(self, context: &Context<'alloc>) -> bool {
        // A state is final when it is no longer possible to use BasicSynthesis.
        self.cp < context.normal_action_durability_cost
    }

    fn use_action<'alloc>(
        self,
        context: &Context<'alloc>,
        action_combo: ActionCombo,
    ) -> Result<(Self, u16, u16), ActionError> {
        let simulation_state = SimulationState {
            cp: self.cp,
            durability: 100,
            progress: 0,
            quality: 0,
            unreliable_quality: self.unreliable_quality,
            effects: self.effects,
        };
        let mut next_simulation_state =
            use_action_combo(&context.settings, simulation_state, action_combo)?;
        let mut durability_cost = 0;
        for action in action_combo.actions() {
            durability_cost += match action {
                Action::BasicSynthesis => context.normal_action_durability_cost,
                Action::BasicTouch => context.normal_action_durability_cost,
                Action::MasterMend => 0,
                Action::Observe => 0,
                Action::TricksOfTheTrade => 0,
                Action::WasteNot => 0,
                Action::Veneration => 0,
                Action::StandardTouch => context.normal_action_durability_cost,
                Action::GreatStrides => 0,
                Action::Innovation => 0,
                Action::WasteNot2 => 0,
                Action::ByregotsBlessing => context.normal_action_durability_cost,
                Action::PreciseTouch => context.normal_action_durability_cost,
                Action::MuscleMemory => context.normal_action_durability_cost,
                Action::CarefulSynthesis => context.normal_action_durability_cost,
                Action::Manipulation => 0,
                Action::PrudentTouch => context.small_action_durability_cost,
                Action::AdvancedTouch => context.normal_action_durability_cost,
                Action::Reflect => context.normal_action_durability_cost,
                Action::PreparatoryTouch => context.big_action_durability_cost,
                Action::Groundwork => context.big_action_durability_cost,
                Action::DelicateSynthesis => context.normal_action_durability_cost,
                Action::IntensiveSynthesis => context.normal_action_durability_cost,
                Action::TrainedEye => 0,
                Action::HeartAndSoul => 0,
                Action::PrudentSynthesis => context.small_action_durability_cost,
                Action::TrainedFinesse => 0,
                Action::RefinedTouch => context.normal_action_durability_cost,
                Action::QuickInnovation => 0,
                Action::ImmaculateMend => 0,
                Action::TrainedPerfection => 0,
                Action::StellarSteadyHand => 0,
                Action::RapidSynthesis => context.normal_action_durability_cost,
                Action::HastyTouch => context.normal_action_durability_cost,
                Action::DaringTouch => context.normal_action_durability_cost,
            };
        }
        if durability_cost > next_simulation_state.cp {
            return Err(ActionError::InsufficientCP);
        }
        next_simulation_state.cp -= durability_cost;
        Ok(Self::from_simulation_state(context, next_simulation_state))
    }
}

type ParetoFront = nunny::Slice<Value>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct Value {
    progress: u16,
    quality: u16,
    step_count: u8,
}

impl Value {
    fn dominates(&self, other: &Self) -> bool {
        self.progress >= other.progress
            && self.quality >= other.quality
            && self.step_count <= other.step_count
    }
}

impl std::ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            progress: self.progress + rhs.progress,
            quality: self.quality + rhs.quality,
            step_count: self.step_count + rhs.step_count,
        }
    }
}

struct Context<'alloc> {
    allocator: BumpPoolGuard<'alloc>,
    settings: SolverSettings,
    interrupt_signal: utils::AtomicFlag,

    /// The amount of CP refunded for every tick of Manipulation still active.
    manipulation_refund: u16,
    /// The amount of CP refunded for every tick of WasteNot still active.
    waste_not_refund: u16,
    /// The amount of CP refunded if TrainedPerfection is active or available.
    trained_perfection_refund: u16,
    /// The amount of CP "refunded" for every 5 durability.
    durability_refund: u16,

    /// The CP cost of using durability for 5-durability actions.
    small_action_durability_cost: u16,
    /// The CP cost of using durability for 10-durability actions.
    normal_action_durability_cost: u16,
    /// The CP cost of using durability for 20-durability actions.
    big_action_durability_cost: u16,
}

impl<'alloc> Context<'alloc> {
    pub fn new(
        mut settings: SolverSettings,
        interrupt_signal: utils::AtomicFlag,
        allocator: BumpPoolGuard<'alloc>,
    ) -> Self {
        let manipulation_available = settings
            .simulator_settings
            .is_action_allowed::<Manipulation>();
        let waste_not_available = settings.simulator_settings.is_action_allowed::<WasteNot>();
        let waste_not_2_available = settings.simulator_settings.is_action_allowed::<WasteNot2>();
        let immaculate_mend_available = settings
            .simulator_settings
            .is_action_allowed::<ImmaculateMend>();
        let trained_perfection_available = settings
            .simulator_settings
            .is_action_allowed::<TrainedPerfection>();

        let manipulation_refund = if manipulation_available {
            Manipulation::CP_COST / 8
        } else {
            0
        };

        let waste_not_refund = if waste_not_2_available {
            WasteNot2::CP_COST / 8
        } else if waste_not_available {
            WasteNot::CP_COST / 4
        } else {
            0
        };

        let mut durability_refund = MasterMend::CP_COST / 6;
        if manipulation_available {
            durability_refund = durability_refund.min(Manipulation::CP_COST / 8);
        }
        if immaculate_mend_available {
            let max_restore = (settings.simulator_settings.max_durability - 5) / 5;
            durability_refund = durability_refund.min(ImmaculateMend::CP_COST / max_restore);
        }

        let small_action_durability_cost = durability_refund;
        let normal_action_durability_cost =
            std::cmp::min(2 * durability_refund, durability_refund + waste_not_refund);
        let big_action_durability_cost = std::cmp::min(
            4 * durability_refund,
            2 * durability_refund + waste_not_refund,
        );

        let trained_perfection_refund = if trained_perfection_available {
            // Assume TrainedPerfection can be used at max potential.
            small_action_durability_cost
                .max(normal_action_durability_cost)
                .max(big_action_durability_cost)
        } else {
            0
        };

        // Disable certain actions to prevent infinite recursion when solving states.
        settings.simulator_settings.allowed_actions = settings
            .simulator_settings
            .allowed_actions
            .remove(Action::Manipulation)
            .remove(Action::WasteNot)
            .remove(Action::WasteNot2)
            .remove(Action::TrainedPerfection)
            .remove(Action::ImmaculateMend)
            .remove(Action::MasterMend);

        Self {
            allocator,
            settings,
            interrupt_signal,
            manipulation_refund,
            waste_not_refund,
            trained_perfection_refund,
            durability_refund,
            small_action_durability_cost,
            normal_action_durability_cost,
            big_action_durability_cost,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ScoreUpperBound {
    pub quality: u16,
    pub step_count: u8,
}

impl ScoreUpperBound {
    const MIN: Self = Self {
        quality: 0,
        step_count: u8::MAX,
    };
}

impl PartialOrd for ScoreUpperBound {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoreUpperBound {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quality
            .cmp(&other.quality)
            .then(other.step_count.cmp(&self.step_count))
    }
}

pub struct ScoreUbSolver<'alloc> {
    context: Context<'alloc>,
    solved_states: FxHashMap<State, &'alloc ParetoFront>,
}

impl<'alloc> ScoreUbSolver<'alloc> {
    pub fn new(
        settings: SolverSettings,
        interrupt_signal: utils::AtomicFlag,
        allocator: BumpPoolGuard<'alloc>,
    ) -> Self {
        Self {
            context: Context::new(settings, interrupt_signal, allocator),
            solved_states: FxHashMap::default(),
        }
    }

    pub fn score_upper_bound(
        &mut self,
        simulation_state: SimulationState,
        current_step_count: u8,
    ) -> Result<ScoreUpperBound, SolverException> {
        let (mut state, current_progress, current_quality) =
            State::from_simulation_state(&self.context, simulation_state);
        let durability_refund =
            (simulation_state.durability + 5) / 5 * self.context.durability_refund;
        state.cp += durability_refund;
        let pareto_front = if let Some(pareto_front) = self.solved_states.get(&state) {
            pareto_front.as_slice()
        } else {
            let mut query_solution = |state, solution| {
                if let Some(solution) = solution {
                    self.solved_states.insert(state, solution)
                } else {
                    self.solved_states.get(&state).copied()
                }
            };
            let ret = solve_state(&self.context, &mut query_solution, state)?.as_slice();
            dbg!(self.solved_states.len());
            ret
        };
        let mut score_ub = ScoreUpperBound::MIN;
        for value in pareto_front {
            if current_progress + value.progress >= self.context.settings.max_progress() {
                let candidate_score_ub = ScoreUpperBound {
                    quality: self
                        .context
                        .settings
                        .max_quality()
                        .min(current_quality.saturating_add(value.quality)),
                    step_count: current_step_count + value.step_count,
                };
                score_ub = std::cmp::max(score_ub, candidate_score_ub);
            }
        }
        Ok(score_ub)
    }
}

fn solve_state<'alloc>(
    context: &Context<'alloc>,
    query_solution: &mut impl FnMut(State, Option<&'alloc ParetoFront>) -> Option<&'alloc ParetoFront>,
    state: State,
) -> Result<&'alloc ParetoFront, SolverException> {
    if context.interrupt_signal.is_set() {
        return Err(SolverException::Interrupted);
    }
    let mut pareto_front = Vec::new();
    for action in FULL_SEARCH_ACTIONS {
        let Ok((next_state, progress, quality)) = state.use_action(context, action) else {
            continue;
        };
        let next_state_pareto_front = if next_state.is_final(context) {
            // The last action must be a Progress-increasing action.
            if progress > 0 {
                &[Value::default()]
            } else {
                [].as_slice()
            }
        } else if let Some(pareto_front) = query_solution(next_state, None) {
            pareto_front.as_slice()
        } else {
            solve_state(context, query_solution, next_state)?.as_slice()
        };
        let offset = Value {
            progress,
            quality,
            step_count: action.steps(),
        };
        extend_pareto_front(&mut pareto_front, next_state_pareto_front, offset);
    }
    let allocated_slice = context.allocator.alloc_slice_move(pareto_front).into_ref();
    let checked_slice = allocated_slice
        .try_into()
        .map_err(|_| internal_error!("Empty ParetoFront.", context.settings, state))?;
    query_solution(state, Some(checked_slice));
    Ok(checked_slice)
}

fn extend_pareto_front(
    current_values: &mut Vec<Value>,
    new_values: &[Value],
    new_values_offset: Value,
) {
    current_values.retain(|value| {
        !new_values
            .iter()
            .any(|new_value| (*new_value + new_values_offset).dominates(value))
    });
    for new_value in new_values {
        let new_value = *new_value + new_values_offset;
        let dominated = current_values
            .iter()
            .any(|value| value.dominates(&new_value));
        if !dominated {
            current_values.push(new_value);
        }
    }
}
