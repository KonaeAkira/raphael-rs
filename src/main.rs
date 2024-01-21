use raphael_rs::{
    config::Settings,
    game::{
        actions::{PROG_DENOM, QUAL_DENOM},
        state::State,
    },
    progress, quality,
    solvers::macro_solver::MacroSolver,
};

fn main() {
    env_logger::init();
    let settings = Settings {
        max_cp: 400,
        max_durability: 60,
        max_progress: progress!(2000),
        max_quality: quality!(40000),
    };
    let state = State::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state);
}
