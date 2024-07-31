use simulator::{Action, ActionMask, Settings, SimulationState};
use solvers::MacroSolver;

fn main() {
    dbg!(std::mem::size_of::<SimulationState>());
    dbg!(std::mem::align_of::<SimulationState>());

    let settings = Settings {
        max_cp: 553,
        max_durability: 70,
        max_progress: 2400,
        max_quality: 20000,
        base_progress: 100,
        base_quality: 100,
        job_level: 90,
        allowed_actions: ActionMask::from_level(90)
            .remove(Action::TrainedEye)
            .remove(Action::QuickInnovation),
        adversarial: true,
    };

    let state = SimulationState::new(&settings);
    let mut solver = MacroSolver::new(settings, Box::new(|_| {}), Box::new(|_| {}));
    solver.solve(state, false);
}
