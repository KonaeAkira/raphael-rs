//! Lazily-initialized global rayon thread pool used by the solver.
//!
//! On the web, the pool is backed by Web Workers sharing the module's memory (via
//! `wasm-bindgen-rayon`), which requires the page to be cross-origin isolated.

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};

static INITIALIZATION_ATTEMPTED: AtomicBool = AtomicBool::new(false);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn initialization_attempted() -> bool {
    INITIALIZATION_ATTEMPTED.load(Ordering::Relaxed)
}

pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Initializes the global thread pool if that hasn't been attempted yet.
/// Resolves once the pool is ready to accept work.
#[cfg(not(target_arch = "wasm32"))]
pub async fn ensure_initialized(num_threads: Option<NonZeroUsize>) {
    if INITIALIZATION_ATTEMPTED.swap(true, Ordering::Relaxed) {
        return;
    }
    let num_threads = num_threads.unwrap_or_else(default_thread_count);
    match rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads.get())
        .build_global()
    {
        Ok(()) => log::debug!("Created global thread pool with num_threads = {num_threads}"),
        Err(error) => log::debug!("Creation of global thread pool failed with error = {error:?}"),
    }
    INITIALIZED.store(true, Ordering::Relaxed);
}

/// Initializes the global thread pool if that hasn't been attempted yet.
/// Resolves once the pool is ready to accept work.
#[cfg(target_arch = "wasm32")]
pub async fn ensure_initialized(num_threads: Option<NonZeroUsize>) {
    if INITIALIZATION_ATTEMPTED.swap(true, Ordering::Relaxed) {
        // Another caller started the initialization. Wait for it to finish.
        while !is_initialized() {
            gloo_timers::future::TimeoutFuture::new(20).await;
        }
        return;
    }
    let num_threads = num_threads.unwrap_or_else(default_thread_count);
    let promise = wasm_bindgen_rayon::init_thread_pool(num_threads.get());
    let result = wasm_bindgen_futures::JsFuture::from(promise).await;
    log::debug!("Initialized thread pool with num_threads = {num_threads}, result = {result:?}");
    INITIALIZED.store(true, Ordering::Relaxed);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn default_thread_count() -> NonZeroUsize {
    std::thread::available_parallelism().map_or(NonZeroUsize::new(4).unwrap(), |detected| {
        let num_threads = std::cmp::max(2, detected.get() / 2);
        NonZeroUsize::new(num_threads).unwrap()
    })
}

#[cfg(target_arch = "wasm32")]
pub fn default_thread_count() -> NonZeroUsize {
    let detected = web_sys::window()
        .map(|window| window.navigator().hardware_concurrency() as usize)
        .unwrap_or(4);
    // See https://github.com/KonaeAkira/raphael-rs/issues/169
    NonZeroUsize::new((detected / 2).clamp(2, 8)).unwrap()
}
