mod actions;
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
    use simulator::{Action, Condition, Settings, SimulationState};

    use crate::MacroSolver;

    pub fn solve(
        settings: &Settings,
        backload_progress: bool,
        minimize_steps: bool,
    ) -> Option<Vec<Action>> {
        MacroSolver::new(settings.clone(), Box::new(|_| {}), Box::new(|_| {})).solve(
            SimulationState::new(settings),
            backload_progress,
            minimize_steps,
        )
    }

    pub fn get_score_triple(settings: &Settings, actions: &[Action]) -> (u16, u8, u8) {
        let quality = get_quality(settings, actions);
        let steps = actions.len() as u8;
        let duration: u8 = actions
            .into_iter()
            .map(|action| action.time_cost() as u8)
            .sum();
        (quality, steps, duration)
    }

    fn get_quality(settings: &Settings, actions: &[Action]) -> u16 {
        let mut state = SimulationState::new(&settings);
        for action in actions {
            state = state
                .use_action(action.clone(), Condition::Normal, &settings)
                .unwrap();
        }
        assert!(state.progress >= settings.max_progress);
        state.get_quality()
    }
}
