static THREAD_POOL_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(target_arch = "wasm32")]
static THREAD_POOL_IS_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn attempt_initialization(num_threads: usize) {
    THREAD_POOL_INIT.call_once(|| {
        initialize(num_threads);
    });
}

pub fn initialization_attempted() -> bool {
    THREAD_POOL_INIT.is_completed()
}

#[cfg(not(target_arch = "wasm32"))]
fn initialize(num_threads: usize) {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
    {
        Ok(()) => log::debug!(
            "Created global thread pool with num_threads = {}",
            num_threads
        ),
        Err(error) => log::debug!(
            "Creation of global thread pool failed with error = {:?}",
            error
        ),
    }
}

#[cfg(target_arch = "wasm32")]
fn initialize(num_threads: usize) {
    let num_threads = if num_threads == 0 {
        default_size()
    } else {
        num_threads
    };
    let future = wasm_bindgen_futures::JsFuture::from(crate::init_thread_pool(num_threads));
    wasm_bindgen_futures::spawn_local(async move {
        let result = future.await;
        log::debug!(
            "Initialized Pool with num_threads = {}, result = {:?}",
            num_threads,
            result
        );
        THREAD_POOL_IS_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn default_size() -> usize {
    std::thread::available_parallelism()
        .unwrap_or(std::num::NonZero::new(8).unwrap())
        .into()
}

#[cfg(target_arch = "wasm32")]
pub fn default_size() -> usize {
    let window = web_sys::window().unwrap();
    window.navigator().hardware_concurrency() as usize
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_initialized() -> bool {
    THREAD_POOL_INIT.is_completed()
}

#[cfg(target_arch = "wasm32")]
pub fn is_initialized() -> bool {
    THREAD_POOL_IS_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed)
}
