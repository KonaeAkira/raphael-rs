use raphael::game::{ActionMask, Settings, State};

use raphael::solvers::MacroSolver;

fn main() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: 6600,
        max_quality: 14040,
        base_progress: 248,
        base_quality: 270,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90, false),
    };

    let state = State::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state);
}
