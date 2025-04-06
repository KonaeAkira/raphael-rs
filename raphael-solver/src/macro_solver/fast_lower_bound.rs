use raphael_sim::*;

use crate::{
    AtomicFlag, QualityUpperBoundSolver, SolverException, SolverSettings,
    actions::{ActionCombo, QUALITY_ONLY_SEARCH_ACTIONS, use_action_combo},
    finish_solver::FinishSolver,
    utils::ScopedTimer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Node {
    quality_upper_bound: u16,
    state: SimulationState,
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.quality_upper_bound
            .cmp(&other.quality_upper_bound)
            .then(self.state.quality.cmp(&other.state.quality))
    }
}

pub fn fast_lower_bound(
    initial_state: SimulationState,
    settings: SolverSettings,
    interrupt_signal: AtomicFlag,
    finish_solver: &mut FinishSolver,
    quality_ub_solver: &mut QualityUpperBoundSolver,
) -> Result<u16, SolverException> {
    let _timer = ScopedTimer::new("Fast lower bound");

    let mut search_queue = std::collections::BinaryHeap::default();
    let initial_node = Node {
        quality_upper_bound: settings.simulator_settings.max_quality,
        state: initial_state,
    };
    search_queue.push(initial_node);

    let mut best_achieved_quality = 0;

    while let Some(node) = search_queue.pop() {
        if interrupt_signal.is_set() {
            return Err(SolverException::Interrupted);
        }
        if node.quality_upper_bound <= best_achieved_quality {
            break;
        }
        for action in QUALITY_ONLY_SEARCH_ACTIONS {
            if !should_use_action(
                *action,
                &node.state,
                settings.simulator_settings.allowed_actions,
            ) {
                continue;
            }
            if let Ok(state) = use_action_combo(&settings, node.state, *action) {
                if !state.is_final(&settings.simulator_settings) {
                    if !finish_solver.can_finish(&state) {
                        continue;
                    }
                    best_achieved_quality = std::cmp::max(best_achieved_quality, state.quality);
                    if *action == ActionCombo::Single(Action::ByregotsBlessing) {
                        continue;
                    }
                    let quality_upper_bound = quality_ub_solver.quality_upper_bound(state)?;
                    if quality_upper_bound <= best_achieved_quality {
                        continue;
                    }
                    search_queue.push(Node {
                        quality_upper_bound,
                        state,
                    });
                }
            }
        }
    }

    Ok(std::cmp::min(
        settings.simulator_settings.max_quality,
        best_achieved_quality,
    ))
}

fn should_use_action(
    action: ActionCombo,
    state: &SimulationState,
    allowed_actions: ActionMask,
) -> bool {
    // Force the use of an opener if one is available
    if state.effects.combo() == Combo::SynthesisBegin {
        let action_is_opener = matches!(
            action,
            ActionCombo::Single(Action::Reflect | Action::MuscleMemory | Action::TrainedEye)
        );
        let opener_available = allowed_actions.has(Action::Reflect)
            || allowed_actions.has(Action::MuscleMemory)
            || allowed_actions.has(Action::TrainedEye);
        return action_is_opener || !opener_available;
    }

    // Misc
    match action {
        ActionCombo::Single(Action::Innovation) => state.effects.innovation() == 0,
        ActionCombo::Single(Action::Veneration) => state.effects.veneration() == 0,
        ActionCombo::Single(Action::Manipulation) => state.effects.manipulation() == 0,
        ActionCombo::Single(Action::WasteNot | Action::WasteNot2) => {
            state.effects.waste_not() == 0 && !state.effects.trained_perfection_active()
        }
        ActionCombo::Single(Action::GreatStrides) => state.effects.great_strides() == 0,
        ActionCombo::Single(Action::TrainedPerfection) => state.effects.waste_not() == 0,
        _ => true,
    }
}
