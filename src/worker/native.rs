use crate::Worker;
use crate::worker::Input;
use crate::worker::Output;
use std::sync::mpsc::{self, Receiver, Sender};

pub(crate) type Scope = DummyScope;
pub(crate) type Id = ();

pub struct DummyScope;
impl DummyScope {
    pub fn respond(&self, _id: Id, _event: Output) {}
}

pub struct NativeBridge {
    pub(crate) tx: Sender<Output>,
    pub(crate) rx: Receiver<Output>,
}

impl NativeBridge {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Output>();
        Self { tx, rx }
    }

    pub fn send(&mut self, input: Input) {
        let worker = Worker::new(input, self.tx.clone());
        std::thread::spawn(move || {
            worker.solver_callback(None, None, None);
        });
    }
}

impl Worker {
    fn new(input: Input, tx: Sender<Output>) -> Self {
        Self {
            input: Some(input),
            tx: Some(tx),
        }
    }
}
