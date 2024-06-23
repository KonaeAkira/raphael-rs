use simulator::{state::InProgress, ActionMask, Settings};
use solvers::MacroSolver;

fn main() {
    let settings = Settings {
        max_cp: 600,
        max_durability: 70,
        max_progress: 4300,
        max_quality: 12800,
        base_progress: 200,
        base_quality: 215,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, true),
    };

    let state = InProgress::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state);
}
