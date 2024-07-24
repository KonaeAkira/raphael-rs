use radix_heap::RadixHeapMap;
use simulator::{
    state::InProgress, Action, ActionMask, ComboAction, Condition, Settings, SimulationState,
};

use crate::{
    actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS},
    finish_solver::FinishSolver,
    upper_bound_solver::UpperBoundSolver,
    utils::{Backtracking, NamedTimer, Score},
};

use super::pareto_set::ParetoSet;

const PROGRESS_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::DelicateSynthesis);

const QUALITY_SEARCH_ACTIONS: ActionMask = QUALITY_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::StandardTouch) // non-combo version
    .remove(Action::AdvancedTouch) // non-combo version
    .remove(Action::DelicateSynthesis);

#[derive(Debug, Clone, Copy)]
struct SearchNode {
    state: InProgress,
    backtrack_index: u32,
}

/// Check if a rotation that maxes out Quality can easily be found
/// This solve function is fast because it doesn't consider all search branches:
/// - Always increases Quality first, then finishes off Progress
/// - Has some manually-coded branch pruning
pub fn quick_search(
    state: InProgress,
    settings: &Settings,
    finish_solver: &mut FinishSolver,
    upper_bound_solver: &mut UpperBoundSolver,
) -> Option<Vec<Action>> {
    let _timer = NamedTimer::new("Quick search");

    let mut search_queue: RadixHeapMap<Score, SearchNode> = RadixHeapMap::default();
    let mut backtracking: Backtracking<Action> = Backtracking::new();
    let mut pareto_set = ParetoSet::default();

    search_queue.push(
        Score::new(
            upper_bound_solver.quality_upper_bound(state),
            0,
            0,
            settings,
        ),
        SearchNode {
            state,
            backtrack_index: Backtracking::<Action>::SENTINEL,
        },
    );

    let mut best_score = Score::new(settings.max_quality, u8::MAX, u8::MAX, settings);
    let mut best_actions = None;

    while let Some((score, node)) = search_queue.pop() {
        if score <= best_score {
            break;
        }
        let allowed_actions = match node.state.raw_state().get_quality() >= settings.max_quality {
            true => PROGRESS_SEARCH_ACTIONS.intersection(settings.allowed_actions),
            false => QUALITY_SEARCH_ACTIONS.intersection(settings.allowed_actions),
        };
        for action in allowed_actions.actions_iter() {
            if !should_use_action(action, node.state.raw_state(), allowed_actions) {
                continue;
            }
            if let Ok(state) = node.state.use_action(action, Condition::Normal, settings) {
                if let Ok(in_progress) = InProgress::try_from(state) {
                    if action == Action::ByregotsBlessing
                        && state.get_quality() < settings.max_quality
                    {
                        continue;
                    }
                    if !finish_solver.can_finish(&in_progress) {
                        continue;
                    }
                    let new_score = Score::new(
                        upper_bound_solver.quality_upper_bound(in_progress),
                        score.duration + action.time_cost() as u8,
                        score.steps + 1,
                        settings,
                    );
                    if new_score <= best_score {
                        continue;
                    }
                    if !pareto_set.insert(state) {
                        continue;
                    }
                    let backtrack_index = backtracking.push(action, node.backtrack_index);
                    search_queue.push(
                        new_score,
                        SearchNode {
                            state: in_progress,
                            backtrack_index,
                        },
                    );
                } else if state.missing_progress == 0 && state.get_quality() >= settings.max_quality
                {
                    let score =
                        Score::new(state.get_quality(), score.duration, score.steps, settings);
                    if score > best_score {
                        let actions = backtracking
                            .get(node.backtrack_index)
                            .chain(std::iter::once(action))
                            .collect();
                        best_score = score;
                        best_actions = Some(actions);
                    }
                }
            }
        }
    }

    dbg!(&best_score, &best_actions);
    best_actions
}

fn should_use_action(action: Action, state: &SimulationState, allowed_actions: ActionMask) -> bool {
    // Force the use of the next combo action if it is available
    match state.combo {
        None => (),
        Some(ComboAction::BasicTouch) => {
            let combo_available = allowed_actions.has(Action::ComboStandardTouch)
                || allowed_actions.has(Action::ComboRefinedTouch);
            return !combo_available
                || matches!(
                    action,
                    Action::ComboStandardTouch | Action::ComboRefinedTouch
                );
        }
        Some(ComboAction::StandardTouch) => {
            let combo_available = allowed_actions.has(Action::ComboAdvancedTouch);
            return !combo_available || matches!(action, Action::ComboAdvancedTouch);
        }
        Some(ComboAction::SynthesisBegin) => {
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
