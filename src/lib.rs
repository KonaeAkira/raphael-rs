#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_rayon::init_thread_pool;

mod app;
pub use app::MacroSolverApp;

mod config;
mod context;
mod elements;
mod fonts;
mod solve;
mod thread_pool;

#[cfg(not(target_arch = "wasm32"))]
mod update;

#[cfg(target_arch = "wasm32")]
pub static OOM_PANIC_OCCURED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
