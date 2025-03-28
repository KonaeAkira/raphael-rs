#[cfg(target_arch = "wasm32")]
use gloo_worker::Registrable;

#[cfg(target_arch = "wasm32")]
fn main() {
    raphael_xiv::Worker::registrar().register();
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
