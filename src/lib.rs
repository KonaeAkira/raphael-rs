#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_rayon::init_thread_pool;

mod app;
pub use app::MacroSolverApp;

mod config;
mod widgets;

#[cfg(target_arch = "wasm32")]
pub static OOM_PANIC_OCCURED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(target_arch = "wasm32")]
pub static THREAD_POOL_IS_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
