use simulator::{state::InProgress, ActionMask, Settings, SimulationState};
use solvers::MacroSolver;

fn main() {
    dbg!(std::mem::size_of::<SimulationState>());
    dbg!(std::mem::align_of::<SimulationState>());

    let settings = Settings {
        max_cp: 600,
        max_durability: 40,
        max_progress: 2400,
        max_quality: 9400,
        base_progress: 266,
        base_quality: 331,
        initial_quality: 0,
        job_level: 94,
        allowed_actions: ActionMask::from_level(90, true),
    };

    let state = InProgress::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state);
}
