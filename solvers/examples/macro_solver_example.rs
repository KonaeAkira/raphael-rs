use simulator::{state::InProgress, ActionMask, Settings};
use solvers::MacroSolver;

fn main() {
    let settings = Settings {
        max_cp: 720,
        max_durability: 80,
        max_progress: 5700,
        max_quality: 10600,
        base_progress: 241,
        base_quality: 322,
        initial_quality: 0,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100, true),
    };

    let state = InProgress::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state);
}
