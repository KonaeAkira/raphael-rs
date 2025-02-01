use radix_heap::RadixHeapMap;
use simulator::{Action, ActionMask, Combo, Settings, SimulationState};

use crate::{
    actions::{use_solver_action, SolverAction, QUALITY_ONLY_SEARCH_ACTIONS},
    finish_solver::FinishSolver,
    macro_solver::pareto_front::QualityParetoFront,
    utils::NamedTimer,
    AtomicFlag, QualityUpperBoundSolver, SolverException,
};

pub fn fast_lower_bound(
    state: SimulationState,
    settings: &Settings,
    interrupt_signal: AtomicFlag,
    finish_solver: &mut FinishSolver,
    upper_bound_solver: &mut QualityUpperBoundSolver,
) -> Result<u16, SolverException> {
    let _timer = NamedTimer::new("Fast lower bound");

    let mut search_queue: RadixHeapMap<u16, SimulationState> = RadixHeapMap::default();
    let mut pareto_set = QualityParetoFront::default();

    let mut quality_lower_bound = 0;

    search_queue.push(upper_bound_solver.quality_upper_bound(state)?, state);

    while let Some((score, state)) = search_queue.pop() {
        if interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        if score <= quality_lower_bound {
            break;
        }
        for action in QUALITY_ONLY_SEARCH_ACTIONS.iter() {
            if !should_use_action(*action, &state, settings.allowed_actions) {
                continue;
            }
            if let Ok(state) = use_solver_action(settings, state, *action) {
                if !state.is_final(settings) {
                    if !finish_solver.can_finish(&state) {
                        continue;
                    }
                    quality_lower_bound = std::cmp::max(quality_lower_bound, state.quality);
                    if *action == SolverAction::Single(Action::ByregotsBlessing) {
                        continue;
                    }
                    let quality_upper_bound = upper_bound_solver.quality_upper_bound(state)?;
                    if quality_upper_bound <= quality_lower_bound {
                        continue;
                    }
                    if !pareto_set.insert(state) {
                        continue;
                    }
                    search_queue.push(std::cmp::min(score, quality_upper_bound), state);
                }
            }
        }
    }

    log::debug!("Fast quality lower bound: {}", quality_lower_bound);
    Ok(std::cmp::min(settings.max_quality, quality_lower_bound))
}

fn should_use_action(
    action: SolverAction,
    state: &SimulationState,
    allowed_actions: ActionMask,
) -> bool {
    // Force the use of the next combo action if it is available
    match state.combo {
        Combo::None => (),
        Combo::BasicTouch => {
            let combo_available = allowed_actions.has(Action::StandardTouch)
                || allowed_actions.has(Action::RefinedTouch);
            return !combo_available
                || matches!(
                    action,
                    SolverAction::Single(Action::StandardTouch | Action::RefinedTouch)
                );
        }
        Combo::StandardTouch => {
            let combo_available = allowed_actions.has(Action::AdvancedTouch);
            return !combo_available
                || matches!(action, SolverAction::Single(Action::AdvancedTouch));
        }
        Combo::SynthesisBegin => {
            let combo_available = allowed_actions.has(Action::Reflect)
                || allowed_actions.has(Action::MuscleMemory)
                || allowed_actions.has(Action::TrainedEye);
            return !combo_available
                || matches!(
                    action,
                    SolverAction::Single(
                        Action::Reflect | Action::MuscleMemory | Action::TrainedEye
                    )
                );
        }
    }

    // Misc
    match action {
        SolverAction::Single(Action::Innovation) => state.effects.innovation() == 0,
        SolverAction::Single(Action::Veneration) => state.effects.veneration() == 0,
        SolverAction::Single(Action::Manipulation) => state.effects.manipulation() == 0,
        SolverAction::Single(Action::WasteNot | Action::WasteNot2) => {
            state.effects.waste_not() == 0
        }
        SolverAction::Single(Action::GreatStrides) => state.effects.great_strides() == 0,
        SolverAction::Single(Action::TrainedPerfection) => state.effects.waste_not() == 0,
        _ => true,
    }
}
