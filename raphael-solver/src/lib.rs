mod actions;

mod finish_solver;
use finish_solver::FinishSolver;

mod quality_upper_bound_solver;
use quality_upper_bound_solver::QualityUbSolver;

mod step_lower_bound_solver;
use step_lower_bound_solver::StepLowerBoundSolver;

mod macro_solver;
pub use macro_solver::MacroSolver;

mod utils;
pub use utils::AtomicFlag;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SolverException {
    NoSolution,
    Interrupted,
    InternalError(String),
}

#[derive(Clone, Copy, Debug)]
pub struct SolverSettings {
    pub simulator_settings: raphael_sim::Settings,
    pub backload_progress: bool,
    pub allow_unsound_branch_pruning: bool,
}

pub mod test_utils {
    use crate::{MacroSolver, SolverException, SolverSettings, utils::AtomicFlag};
    use raphael_sim::*;

    pub fn solve(
        settings: &Settings,
        backload_progress: bool,
        allow_unsound_branch_pruning: bool,
    ) -> Result<Vec<Action>, SolverException> {
        let solver_settings = SolverSettings {
            simulator_settings: *settings,
            backload_progress,
            allow_unsound_branch_pruning,
        };
        MacroSolver::new(
            solver_settings,
            Box::new(|_| {}),
            Box::new(|_| {}),
            AtomicFlag::new(),
        )
        .solve()
    }

    pub fn get_score_quad(settings: &Settings, actions: &[Action]) -> (u16, u8, u8, u16) {
        let quality = get_quality(settings, actions);
        let capped_quality = std::cmp::min(quality, settings.max_quality);
        let overflow_quality = quality.saturating_sub(settings.max_quality);
        let steps = actions.len() as u8;
        let duration: u8 = actions.iter().map(|action| action.time_cost()).sum();
        (capped_quality, steps, duration, overflow_quality)
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
        quality_lock.is_none_or(|quality| state.quality == quality)
    }
}
