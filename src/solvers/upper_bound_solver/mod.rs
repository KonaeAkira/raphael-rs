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
    DURABILITY_COST, GREAT_STRIDES_COST, INNOVATION_COST, MANIPULATION_COST, WASTE_NOT_COST,
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
        for cp_for_progress in 0..=cp_budget {
            if self
                .progress_bound_solver
                .progress_upper_bound(cp_for_progress)
                >= state.missing_progress
            {
                let cp_for_quality = cp_budget - cp_for_progress;
                let existing_quality = self
                    .settings
                    .max_quality
                    .saturating_sub(state.missing_quality);
                return self
                    .quality_bound_solver
                    .quality_upper_bound(cp_for_quality, state.effects.inner_quiet)
                    .saturating_add(existing_quality);
            }
        }
        Quality::new(0)
    }

    const fn _get_cp_budget(state: &InProgress) -> CP {
        state.cp
            + (state.durability as CP / 5) * DURABILITY_COST
            + state.effects.waste_not as CP * WASTE_NOT_COST
            + state.effects.innovation as CP * INNOVATION_COST
            + state.effects.veneration as CP * WASTE_NOT_COST
            + state.effects.manipulation as CP * MANIPULATION_COST
            + if state.effects.great_strides != 0 {
                GREAT_STRIDES_COST
            } else {
                0
            }
    }
}
