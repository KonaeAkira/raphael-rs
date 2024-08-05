#[cfg(target_arch = "wasm32")]
use gloo_worker::Registrable;

#[cfg(target_arch = "wasm32")]
fn main() {
    raphael_xiv::Worker::registrar().register();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
