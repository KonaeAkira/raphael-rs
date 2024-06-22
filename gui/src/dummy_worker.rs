use gloo_worker::Registrable;
fn main() {
    raphael_gui::WebWorker::registrar().register();
}
