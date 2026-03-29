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

#[cfg(test)]
pub mod test_utils;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SolverException {
    NoSolution,
    Interrupted,
    /// The `SearchQueueCapacityExceeded` error is raised when there are no more valid
    /// indices for the already visited nodes in the search queue.
    ///
    /// On 32-bit platforms, the index into the queue is stored as a 26-bit integer.
    /// This means that there can be a maximum of 67,108,864 nodes visited before
    /// the indices are exhausted.
    ///
    /// On 64-bit platforms, the index is a 56-bit integer, so this error is realistically
    /// never raised on 64-bit platforms.
    SearchQueueCapacityExceeded,
    InternalError(String),
}

impl std::fmt::Debug for SolverException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSolution => write!(f, "NoSolution"),
            Self::Interrupted => write!(f, "Interrupted"),
            Self::SearchQueueCapacityExceeded => write!(f, "SearchQueueCapacityExceeded"),
            Self::InternalError(message) => f.write_str(message),
        }
    }
}

mod macros {
    macro_rules! internal_error {
        ( $desc:expr, $( $x:expr ),* ) => {
            {
                use std::fmt::Write as _;
                let mut message = String::from(concat!(
                    "The solver encountered an internal error.\n",
                    "Please submit a bug report.\n\n",
                    "--- Description ---\n\n",
                ));
                write!(message, "{}\n\n", $desc).unwrap();
                write!(message, "Location: {}:{}:{}\n\n", file!(), line!(), column!()).unwrap();
                message += "--- Debug info ---\n";
                $(
                    write!(message, "\n{} = {:#?}\n", stringify!($x), $x).unwrap();
                )*
                crate::SolverException::InternalError(message)
            }
        };
    }
    pub(crate) use internal_error;
}

#[derive(Clone, Copy, Debug)]
pub struct SolverSettings {
    pub simulator_settings: raphael_sim::Settings,
    pub allow_non_max_quality_solutions: bool,
}

impl SolverSettings {
    pub fn max_durability(&self) -> u16 {
        self.simulator_settings.max_durability
    }

    pub fn max_cp(&self) -> u16 {
        self.simulator_settings.max_cp
    }

    pub fn max_progress(&self) -> u16 {
        self.simulator_settings.max_progress
    }

    pub fn max_quality(&self) -> u16 {
        self.simulator_settings.max_quality
    }

    pub fn base_progress(&self) -> u16 {
        self.simulator_settings.base_progress
    }

    pub fn base_quality(&self) -> u16 {
        self.simulator_settings.base_quality
    }
}
