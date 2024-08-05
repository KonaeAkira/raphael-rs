
use crate::app::{SolverConfig, SolverEvent};
use simulator::{Action, Settings, SimulationState};

#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{self, Receiver};
use std::sync::mpsc::{Sender};

#[cfg(target_arch = "wasm32")]
type Message = u64;
type Input = (Settings, SolverConfig);
type Output = SolverEvent;


#[cfg(target_arch = "wasm32")]
type Scope = gloo_worker::WorkerScope<Worker>;
#[cfg(target_arch = "wasm32")]
type Id = gloo_worker::HandlerId;


#[cfg(not(target_arch = "wasm32"))]
type Scope = DummyScope;
#[cfg(not(target_arch = "wasm32"))]
type Id = ();

#[cfg(not(target_arch = "wasm32"))]
pub struct DummyScope;
#[cfg(not(target_arch = "wasm32"))]
impl DummyScope {
    pub fn respond(&self, _id: Id, _event: Output){}
}

#[cfg(not(target_arch = "wasm32"))]
pub struct NativeBridge {
    pub(crate) rx: Option<Receiver<Output>>
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeBridge {
    pub fn new() -> Self {
        Self { rx : None }
    }

    pub fn send(&mut self, input: Input) {
        let (tx, rx) = mpsc::channel::<Output>();

        let worker = Worker::new(input, tx);
        std::thread::spawn(move || {
            worker.solver_callback(None, None, None);
        });

        self.rx = Some(rx);
    }
}

pub struct Worker {
    input: Option<Input>,
    tx: Option<Sender<Output>>,
}

impl Worker {
    #[cfg(not(target_arch = "wasm32"))]
    fn new(input: Input, tx: Sender<Output>) -> Worker {
        Worker {
            input: Some(input),
            tx: Some(tx),
        }
    }

    pub fn solver_callback(
        &self,
        scope: Option<&Scope>,
        id: Option<Id>,
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
            self.send_event(tx.clone(), scope, id, SolverEvent::IntermediateSolution(actions.to_vec()));
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
                self.send_event(tx.clone(), scope, id, SolverEvent::FinalSolution(Vec::new()));
            }
        }
    }

    // Adapter to unify both implementations
    fn send_event(
        &self,
        tx: Option<Sender<SolverEvent>>,
        scope: Option<&Scope>,
        id: Option<Id>,
        event: SolverEvent,
    ) {
        if cfg!(target_arch = "wasm32") {
            scope.unwrap().respond(id.unwrap(), event);
        } else {
            tx.unwrap().send(event).unwrap();
        }
    }
}


#[cfg(target_arch = "wasm32")]
impl gloo_worker::Worker for Worker {
    type Message = Message;
    type Input = Input;
    type Output = Output;

    fn create(_scope: &gloo_worker::WorkerScope<Self>) -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        Self {
            input: None,
            tx: None,
        }
    }

    fn update(&mut self, _scope: &gloo_worker::WorkerScope<Self>, _msg: Self::Message) {}

    fn received(
        &mut self,
        scope: &gloo_worker::WorkerScope<Self>,
        msg: Self::Input,
        id: gloo_worker::HandlerId,
    ) {
        self.solver_callback(Some(scope), Some(id), Some(msg));
    }
}
