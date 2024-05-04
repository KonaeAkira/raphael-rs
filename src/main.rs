use raphael::game::{
    units::{Progress, Quality},
    Settings, State,
};

use raphael::solvers::MacroSolver;

fn main() {
    dbg!(std::mem::size_of::<State>());
    dbg!(std::mem::align_of::<State>());

    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: Progress::from(2500.00),
        max_quality: Quality::from(40000.00),
    };
    
    let state = State::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state);
}
