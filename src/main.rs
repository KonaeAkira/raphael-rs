use raphael_rs::{
    config::Settings,
    game::{
        state::State,
        units::{progress::Progress, quality::Quality},
    },
    solvers::macro_solver::MacroSolver,
};

fn main() {
    env_logger::init();
    let settings = Settings {
        max_cp: 700,
        max_durability: 70,
        max_progress: Progress::from(2500),
        max_quality: Quality::from(40000),
    };
    let state = State::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state);
}
