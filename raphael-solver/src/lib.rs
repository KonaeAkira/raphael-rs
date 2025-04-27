mod actions;

mod finish_solver;
use finish_solver::FinishSolver;

mod quality_upper_bound_solver;
use quality_upper_bound_solver::QualityUpperBoundSolver;

mod step_lower_bound_solver;
use raphael_sim::SimulationState;
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
    pub simulator_initial_state: Option<SimulationState>,
    pub backload_progress: bool,
    pub allow_unsound_branch_pruning: bool,
}

impl SolverSettings {
    pub fn max_durability(&self) -> u16 {
        self.simulator_settings.max_durability
    }

    pub fn max_cp(&self) -> u16 {
        self.simulator_settings.max_cp
    }

    pub fn max_progress(&self) -> u32 {
        #[allow(clippy::useless_conversion)]
        u32::from(self.simulator_settings.max_progress)
    }

    pub fn max_quality(&self) -> u32 {
        #[allow(clippy::useless_conversion)]
        u32::from(self.simulator_settings.max_quality)
    }

    pub fn base_progress(&self) -> u32 {
        #[allow(clippy::useless_conversion)]
        u32::from(self.simulator_settings.base_progress)
    }

    pub fn base_quality(&self) -> u32 {
        #[allow(clippy::useless_conversion)]
        u32::from(self.simulator_settings.base_quality)
    }

    pub fn initial_state(&self) -> SimulationState {
        self.simulator_initial_state
            .unwrap_or_else(|| SimulationState::new(&self.simulator_settings))
    }
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
            simulator_initial_state: None,
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

    pub fn get_score_quad(settings: &Settings, actions: &[Action]) -> (u32, u8, u8, u32) {
        let quality = get_quality(settings, actions);
        let capped_quality = std::cmp::min(quality, u32::from(settings.max_quality));
        let overflow_quality = quality.saturating_sub(u32::from(settings.max_quality));
        let steps = actions.len() as u8;
        let duration: u8 = actions.iter().map(|action| action.time_cost()).sum();
        (capped_quality, steps, duration, overflow_quality)
    }

    pub fn get_quality(settings: &Settings, actions: &[Action]) -> u32 {
        let mut state = SimulationState::new(settings);
        for action in actions {
            state = state.use_action(*action, settings).unwrap();
        }
        assert!(state.progress >= u32::from(settings.max_progress));
        state.quality
    }

    pub fn is_progress_backloaded(actions: &[Action], settings: &Settings) -> bool {
        let mut state = SimulationState::new(settings);
        let mut quality_lock = None;
        for action in actions {
            state = state.use_action(*action, settings).unwrap();
            if state.progress != 0 && quality_lock.is_none() {
                quality_lock = Some(state.quality);
            }
        }
        quality_lock.is_none_or(|quality| state.quality == quality)
    }
}
