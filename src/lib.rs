#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_rayon::init_thread_pool;

mod app;
pub use app::MacroSolverApp;

mod config;
mod widgets;
