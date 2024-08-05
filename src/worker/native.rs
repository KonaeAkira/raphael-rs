use crate::worker::Input;
use crate::worker::Output;
use crate::Worker;
use std::sync::mpsc::{self, Receiver, Sender};

pub(crate) type Scope = DummyScope;
pub(crate) type Id = ();

pub struct DummyScope;
impl DummyScope {
    pub fn respond(&self, _id: Id, _event: Output) {}
}

pub struct NativeBridge {
    pub(crate) rx: Option<Receiver<Output>>,
}

impl NativeBridge {
    pub fn new() -> Self {
        Self { rx: None }
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

impl Worker {
    fn new(input: Input, tx: Sender<Output>) -> Worker {
        Worker {
            input: Some(input),
            tx: Some(tx),
        }
    }
}
