mod actions;
mod branch_pruning;
mod utils;

mod finish_solver;
use finish_solver::FinishSolver;

mod quality_upper_bound_solver;
use quality_upper_bound_solver::QualityUpperBoundSolver;

mod step_lower_bound_solver;
use step_lower_bound_solver::StepLowerBoundSolver;

mod macro_solver;
pub use macro_solver::MacroSolver;

pub mod test_utils {
    use crate::MacroSolver;
    use simulator::*;

    pub fn solve(
        settings: &Settings,
        backload_progress: bool,
        unsound_branch_pruning: bool,
    ) -> Option<Vec<Action>> {
        MacroSolver::new(
            *settings,
            backload_progress,
            unsound_branch_pruning,
            Box::new(|_| {}),
            Box::new(|_| {}),
        )
        .solve(SimulationState::new(settings))
    }

    pub fn get_score_triple(settings: &Settings, actions: &[Action]) -> (u16, u8, u8) {
        let quality = get_quality(settings, actions);
        let steps = actions.len() as u8;
        let duration: u8 = actions.iter().map(|action| action.time_cost() as u8).sum();
        (quality, steps, duration)
    }

    pub fn get_quality(settings: &Settings, actions: &[Action]) -> u16 {
        let mut state = SimulationState::new(settings);
        for action in actions {
            state = state
                .use_action(*action, Condition::Normal, settings)
                .unwrap();
        }
        assert!(state.progress >= settings.max_progress);
        state.quality
    }

    pub fn is_progress_backloaded(actions: &[Action], settings: &Settings) -> bool {
        let mut state = SimulationState::new(settings);
        let mut quality_lock = None;
        for action in actions {
            state = state
                .use_action(*action, Condition::Normal, settings)
                .unwrap();
            if state.progress != 0 && quality_lock.is_none() {
                quality_lock = Some(state.quality);
            }
        }
        match quality_lock {
            Some(quality) => state.quality == quality,
            None => true,
        }
    }
}
