mod constants;

mod progress_bound_solver;
use progress_bound_solver::ProgressBoundSolver;

mod quality_bound_solver;
use quality_bound_solver::QualityBoundSolver;

use crate::game::{
    state::InProgress,
    units::{Quality, CP},
    Settings,
};

use self::constants::{
    DURABILITY_COST, MANIPULATION_COST, WASTE_NOT_COST,
};

pub struct UpperBoundSolver {
    settings: Settings,
    progress_bound_solver: ProgressBoundSolver,
    quality_bound_solver: QualityBoundSolver,
}

impl UpperBoundSolver {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            progress_bound_solver: ProgressBoundSolver::new(settings),
            quality_bound_solver: QualityBoundSolver::new(settings),
        }
    }

    pub fn quality_upper_bound(&mut self, state: &InProgress) -> Quality {
        let cp_budget = Self::_get_cp_budget(&state);
        let cp_for_progress = self._calculate_cp_for_progress(state, cp_budget);
        let existing_quality = self
            .settings
            .max_quality
            .saturating_sub(state.missing_quality);
        self.quality_bound_solver
            .quality_upper_bound(
                cp_budget - cp_for_progress,
                state.effects.inner_quiet,
                state.effects.innovation,
                state.effects.great_strides,
            )
            .saturating_add(existing_quality)
    }

    fn _calculate_cp_for_progress(&mut self, state: &InProgress, cp_budget: CP) -> CP {
        let mut lo: CP = 0;
        let mut hi: CP = cp_budget;
        while lo < hi {
            let mean = (lo + hi) / 2;
            if self.progress_bound_solver.progress_upper_bound(
                mean,
                state.effects.muscle_memory,
                state.effects.veneration,
            ) >= state.missing_progress
            {
                hi = mean;
            } else {
                lo = mean + 1;
            }
        }
        lo
    }

    const fn _get_cp_budget(state: &InProgress) -> CP {
        state.cp
            + (state.durability as CP / 5) * DURABILITY_COST
            + state.effects.waste_not as CP * WASTE_NOT_COST
            + state.effects.manipulation as CP * MANIPULATION_COST
    }
}
