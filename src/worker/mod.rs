use crate::app::{SolverConfig, SolverEvent};
use simulator::{Action, Settings, SimulationState};
use std::sync::mpsc::Sender;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(not(target_arch = "wasm32"))]
use crate::worker::native as worker;
#[cfg(not(target_arch = "wasm32"))]
use crate::worker::native::NativeBridge;
#[cfg(not(target_arch = "wasm32"))]
pub type BridgeType = NativeBridge;

#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(target_arch = "wasm32")]
use crate::worker::web as worker;
#[cfg(target_arch = "wasm32")]
use gloo_worker::WorkerBridge;
#[cfg(target_arch = "wasm32")]
pub type BridgeType = WorkerBridge<Worker>;

type Input = (Settings, SolverConfig);
type Output = SolverEvent;

pub struct Worker {
    input: Option<Input>,
    tx: Option<Sender<Output>>,
}

impl Worker {
    pub fn solver_callback(
        &self,
        scope: Option<&worker::Scope>,
        id: Option<worker::Id>,
        input: Option<Input>,
    ) {
        let input = if cfg!(not(target_arch = "wasm32")) {
            self.input.unwrap()
        } else {
            input.unwrap()
        };

        let settings = input.0;
        let config = input.1;

        let tx = self.tx.clone();
        let solution_callback = move |actions: &[Action]| {
            self.send_event(
                tx.clone(),
                scope,
                id,
                SolverEvent::IntermediateSolution(actions.to_vec()),
            );
        };

        let tx = self.tx.clone();
        let progress_callback = move |progress: f32| {
            self.send_event(tx.clone(), scope, id, SolverEvent::Progress(progress));
        };

        let final_solution = solvers::MacroSolver::new(
            settings,
            Box::new(solution_callback),
            Box::new(progress_callback),
        )
        .solve(
            SimulationState::new(&settings),
            config.backload_progress,
            config.minimize_steps,
        );

        let tx = self.tx.clone();
        match final_solution {
            Some(actions) => {
                self.send_event(tx.clone(), scope, id, SolverEvent::FinalSolution(actions));
            }
            None => {
                self.send_event(
                    tx.clone(),
                    scope,
                    id,
                    SolverEvent::FinalSolution(Vec::new()),
                );
            }
        }
    }

    // Adapter to unify both implementations
    fn send_event(
        &self,
        tx: Option<Sender<SolverEvent>>,
        scope: Option<&worker::Scope>,
        id: Option<worker::Id>,
        event: SolverEvent,
    ) {
        if cfg!(target_arch = "wasm32") {
            scope.unwrap().respond(id.unwrap(), event);
        } else {
            tx.unwrap().send(event).unwrap();
        }
    }
}
