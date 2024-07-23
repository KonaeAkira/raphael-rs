use simulator::{state::InProgress, Action, ActionMask, Settings, SimulationState};
use solvers::MacroSolver;

fn main() {
    dbg!(std::mem::size_of::<SimulationState>());
    dbg!(std::mem::align_of::<SimulationState>());

    let settings = Settings {
        max_cp: 699,
        max_durability: 80,
        max_progress: 5700,
        max_quality: 20000,
        base_progress: 295,
        base_quality: 310,
        job_level: 100,
        allowed_actions: ActionMask::from_level(100).remove(Action::TrainedEye),
        adversarial: false,
    };

    let state = InProgress::new(&settings);
    let mut solver = MacroSolver::new(settings, Box::new(|_| {}));
    solver.solve(state, false);
}
