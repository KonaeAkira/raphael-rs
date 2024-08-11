use radix_heap::RadixHeapMap;
use simulator::{Action, ActionMask, Combo, Condition, Settings, SimulationState};

use crate::{
    actions::{DURABILITY_ACTIONS, QUALITY_ACTIONS},
    finish_solver::FinishSolver,
    macro_solver::pareto_front::QualityParetoFront,
    utils::NamedTimer,
    QualityUpperBoundSolver,
};

const SEARCH_ACTIONS: ActionMask = QUALITY_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::StandardTouch) // non-combo version
    .remove(Action::AdvancedTouch) // non-combo version
    .remove(Action::DelicateSynthesis);

pub fn fast_lower_bound(
    state: SimulationState,
    settings: &Settings,
    finish_solver: &mut FinishSolver,
    upper_bound_solver: &mut QualityUpperBoundSolver,
) -> u16 {
    let _timer = NamedTimer::new("Fast lower bound");
    let allowed_actions = settings.allowed_actions.intersection(SEARCH_ACTIONS);

    let mut search_queue: RadixHeapMap<u16, SimulationState> = RadixHeapMap::default();
    let mut pareto_set = QualityParetoFront::default();

    let mut quality_lower_bound = 0;

    search_queue.push(upper_bound_solver.quality_upper_bound(state), state);

    while let Some((score, state)) = search_queue.pop() {
        if score <= quality_lower_bound {
            break;
        }
        for action in allowed_actions.actions_iter() {
            if !should_use_action(action, &state, allowed_actions) {
                continue;
            }
            if let Ok(state) = state.use_action(action, Condition::Normal, settings) {
                if !state.is_final(settings) {
                    if !finish_solver.can_finish(&state) {
                        continue;
                    }
                    quality_lower_bound = std::cmp::max(quality_lower_bound, state.get_quality());
                    if action == Action::ByregotsBlessing {
                        continue;
                    }
                    let quality_upper_bound = upper_bound_solver.quality_upper_bound(state);
                    if quality_upper_bound <= quality_lower_bound {
                        continue;
                    }
                    if !pareto_set.insert(state, settings) {
                        continue;
                    }
                    search_queue.push(quality_upper_bound, state);
                }
            }
        }
    }

    dbg!(quality_lower_bound);
    std::cmp::min(settings.max_quality, quality_lower_bound)
}

fn should_use_action(action: Action, state: &SimulationState, allowed_actions: ActionMask) -> bool {
    // Force the use of the next combo action if it is available
    match state.combo {
        Combo::None => (),
        Combo::BasicTouch => {
            let combo_available = allowed_actions.has(Action::ComboStandardTouch)
                || allowed_actions.has(Action::ComboRefinedTouch);
            return !combo_available
                || matches!(
                    action,
                    Action::ComboStandardTouch | Action::ComboRefinedTouch
                );
        }
        Combo::StandardTouch => {
            let combo_available = allowed_actions.has(Action::ComboAdvancedTouch);
            return !combo_available || matches!(action, Action::ComboAdvancedTouch);
        }
        Combo::SynthesisBegin => {
            let combo_available = allowed_actions.has(Action::Reflect)
                || allowed_actions.has(Action::MuscleMemory)
                || allowed_actions.has(Action::TrainedEye);
            return !combo_available
                || matches!(
                    action,
                    Action::Reflect | Action::MuscleMemory | Action::TrainedEye
                );
        }
    }

    // Misc
    match action {
        Action::Innovation => state.effects.innovation() == 0,
        Action::Veneration => state.effects.veneration() == 0,
        Action::Manipulation => state.effects.manipulation() == 0,
        Action::WasteNot | Action::WasteNot2 => state.effects.waste_not() == 0,
        Action::GreatStrides => state.effects.great_strides() == 0,
        Action::TrainedPerfection => state.effects.waste_not() == 0,
        _ => true,
    }
}
