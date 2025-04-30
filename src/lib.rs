#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_rayon::init_thread_pool;

mod app;
pub use app::MacroSolverApp;

mod config;
mod widgets;

#[cfg(target_arch = "wasm32")]
pub const OOM_STATUS: usize = usize::MAX;
#[cfg(target_arch = "wasm32")]
pub static ATOMIC_STATUS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
