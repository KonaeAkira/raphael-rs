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

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SolverException {
    NoSolution,
    Interrupted,
    InternalError(String),
    #[cfg(target_arch = "wasm32")]
    AllocError,
}

mod macros {
    macro_rules! internal_error {
        ( $desc:expr, $( $x:expr ),* ) => {
            {
                let mut message = String::from(concat!(
                    "The solver encountered an internal error.\n",
                    "Please submit a bug report.\n\n",
                    "--- Description ---\n\n",
                ));
                message += &format!("{}\n\n", $desc);
                message += &format!("Location: {}:{}\n\n", file!(), line!());
                message += "--- Variables ---\n";
                $(
                    message += &format!("\n{} = {:#?}\n", stringify!($x), $x);
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
