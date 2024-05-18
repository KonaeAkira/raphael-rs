use raphael::game::{
    units::{Progress, Quality}, ActionMask, Settings, State
};

use raphael::solvers::MacroSolver;

fn main() {
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: Progress::from(2500.00),
        max_quality: Quality::from(40000.00),
        allowed_actions: ActionMask::from_level(90, false),
    };
    
    let state = State::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state);
}
