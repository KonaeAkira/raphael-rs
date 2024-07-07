use gloo_worker::Registrable;
fn main() {
    raphael_xiv::WebWorker::registrar().register();
}
