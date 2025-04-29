mod actions;

mod finish_solver;
use finish_solver::FinishSolver;

mod quality_upper_bound_solver;
use quality_upper_bound_solver::QualityUbSolver;

mod step_lower_bound_solver;
use step_lower_bound_solver::StepLbSolver;

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
}
