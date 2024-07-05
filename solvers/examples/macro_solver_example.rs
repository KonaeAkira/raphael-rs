use simulator::{state::InProgress, ActionMask, Settings, SimulationState};
use solvers::MacroSolver;

fn main() {
    dbg!(std::mem::size_of::<SimulationState>());
    dbg!(std::mem::align_of::<SimulationState>());

    let settings = Settings {
        max_cp: 703,
        max_durability: 80,
        max_progress: 6600,
        max_quality: 12000,
        base_progress: 214,
        base_quality: 231,
        initial_quality: 0,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100, true),
    };

    let state = InProgress::new(&settings);
    let mut solver = MacroSolver::new(settings);
    solver.solve(state, false);
}
