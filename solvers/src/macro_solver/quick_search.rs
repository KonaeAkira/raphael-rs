use simulator::{Action, ActionMask, Combo, Condition, Settings, SimulationState};

use crate::{
    actions::{DURABILITY_ACTIONS, PROGRESS_ACTIONS, QUALITY_ACTIONS},
    finish_solver::FinishSolver,
    macro_solver::search_queue::SearchQueue,
    utils::NamedTimer,
    QualityUpperBoundSolver,
};

use super::search_queue::SearchScore;

const PROGRESS_SEARCH_ACTIONS: ActionMask = PROGRESS_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::DelicateSynthesis);

const QUALITY_SEARCH_ACTIONS: ActionMask = QUALITY_ACTIONS
    .union(DURABILITY_ACTIONS)
    .remove(Action::StandardTouch) // non-combo version
    .remove(Action::AdvancedTouch) // non-combo version
    .remove(Action::DelicateSynthesis);

#[derive(Clone, Copy)]
struct Solution {
    quality: u16,
    action: Action,
    backtrack_id: usize,
}

/// Check if a rotation that maxes out Quality can easily be found
/// This solve function is fast because it doesn't consider all search branches:
/// - Always increases Quality first, then finishes off Progress
/// - Has some manually-coded branch pruning
pub fn quick_search(
    initial_state: SimulationState,
    settings: &Settings,
    finish_solver: &mut FinishSolver,
    upper_bound_solver: &mut QualityUpperBoundSolver,
) -> Option<Vec<Action>> {
    let _timer = NamedTimer::new("Quick search");

    let initial_score = SearchScore::new(
        upper_bound_solver.quality_upper_bound(initial_state),
        0,
        0,
        settings,
    );
    let minimum_score = SearchScore {
        quality: settings.max_quality,
        duration: u8::MAX,
        steps: u8::MAX,
        quality_overflow: 0,
    };
    let mut search_queue = SearchQueue::new(initial_state, initial_score, minimum_score, *settings);

    let mut solution: Option<Solution> = None;

    while let Some((state, score, backtrack_id)) = search_queue.pop() {
        let allowed_actions = match state.get_quality() >= settings.max_quality {
            true => PROGRESS_SEARCH_ACTIONS.intersection(settings.allowed_actions),
            false => QUALITY_SEARCH_ACTIONS.intersection(settings.allowed_actions),
        };
        for action in allowed_actions.actions_iter() {
            if !should_use_action(action, &state, allowed_actions) {
                continue;
            }
            if let Ok(state) = state.use_action(action, Condition::Normal, settings) {
                if action == Action::ByregotsBlessing && state.get_quality() < settings.max_quality
                {
                    continue;
                }
                if !state.is_final(settings) {
                    if !finish_solver.can_finish(&state) {
                        continue;
                    }
                    let quality_upper_bound = if state.get_quality() >= settings.max_quality {
                        state.get_quality()
                    } else {
                        upper_bound_solver.quality_upper_bound(state)
                    };
                    search_queue.push(
                        state,
                        SearchScore::new(
                            quality_upper_bound,
                            score.duration + action.time_cost() as u8,
                            score.steps + 1,
                            settings,
                        ),
                        action,
                        backtrack_id,
                    );
                } else if state.progress >= settings.max_progress
                    && state.get_quality() >= settings.max_quality
                {
                    search_queue.update_min_score(SearchScore::new(
                        state.get_quality(),
                        score.duration,
                        score.steps,
                        settings,
                    ));
                    if solution.is_none() || solution.unwrap().quality < state.get_quality() {
                        solution = Some(Solution {
                            quality: state.get_quality(),
                            action,
                            backtrack_id,
                        });
                    }
                }
            }
        }
    }

    if let Some(solution) = solution {
        let actions: Vec<_> = search_queue
            .backtrack(solution.backtrack_id)
            .chain(std::iter::once(solution.action))
            .collect();
        dbg!(&actions);
        Some(actions)
    } else {
        None
    }
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
