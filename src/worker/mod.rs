use crate::app::{SolverConfig, SolverEvent};
use simulator::{Action, Settings, SimulationState};
use solvers::test_utils;
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
    #[allow(unused)]
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
        let progress_callback = move |progress: usize| {
            self.send_event(tx.clone(), scope, id, SolverEvent::Progress(progress));
        };

        let mut solution = if config.minimize_steps {
            None // skip unsound solver
        } else {
            solvers::MacroSolver::new(
                settings,
                true,
                true,
                Box::new(solution_callback.clone()),
                Box::new(progress_callback.clone()),
            )
            .solve(SimulationState::new(&settings))
        };

        if solution.is_none()
            || test_utils::get_quality(&settings, solution.as_ref().unwrap().as_slice())
                < settings.max_quality
        {
            progress_callback(0); // reset solver progress
            solution = solvers::MacroSolver::new(
                settings,
                config.backload_progress,
                false,
                Box::new(solution_callback),
                Box::new(progress_callback),
            )
            .solve(SimulationState::new(&settings));
        }

        let tx = self.tx.clone();
        match solution {
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
    #[allow(unused)]
    fn send_event(
        &self,
        tx: Option<Sender<SolverEvent>>,
        scope: Option<&worker::Scope>,
        id: Option<worker::Id>,
        event: SolverEvent,
    ) {
        #[cfg(target_arch = "wasm32")]
        scope.unwrap().respond(id.unwrap(), event);
        #[cfg(not(target_arch = "wasm32"))]
        tx.unwrap().send(event).unwrap();
    }
}
