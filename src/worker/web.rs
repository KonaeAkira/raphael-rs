use crate::worker::Input;
use crate::worker::Output;
use crate::Worker;

type Message = u64;
pub(crate) type Scope = gloo_worker::WorkerScope<Worker>;
pub(crate) type Id = gloo_worker::HandlerId;

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
