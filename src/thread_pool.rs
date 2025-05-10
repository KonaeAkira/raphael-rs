use std::num::NonZeroUsize;

static THREAD_POOL_INIT: std::sync::Once = std::sync::Once::new();

#[cfg(target_arch = "wasm32")]
static THREAD_POOL_IS_INITIALIZED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

pub fn attempt_initialization(num_threads: Option<NonZeroUsize>) {
    THREAD_POOL_INIT.call_once(|| {
        initialize(num_threads);
    });
}

pub fn initialization_attempted() -> bool {
    THREAD_POOL_INIT.is_completed()
}

#[cfg(not(target_arch = "wasm32"))]
fn initialize(num_threads: Option<NonZeroUsize>) {
    let num_threads = match num_threads {
        Some(num_threads) => num_threads,
        None => default_thread_count(),
    };
    match rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads.get())
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
fn initialize(num_threads: Option<NonZeroUsize>) {
    let num_threads = match num_threads {
        Some(num_threads) => num_threads,
        None => default_thread_count(),
    };
    let future = wasm_bindgen_futures::JsFuture::from(crate::init_thread_pool(num_threads.get()));
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
pub fn default_thread_count() -> NonZeroUsize {
    std::thread::available_parallelism().map_or(NonZeroUsize::new(4).unwrap(), |detected| {
        let num_threads = std::cmp::max(2, detected.get() / 2);
        NonZeroUsize::new(num_threads).unwrap()
    })
}

#[cfg(target_arch = "wasm32")]
pub fn default_thread_count() -> NonZeroUsize {
    let window = web_sys::window().unwrap();
    let detected = window.navigator().hardware_concurrency() as usize;
    // See https://github.com/KonaeAkira/raphael-rs/issues/169
    NonZeroUsize::new((detected / 2).clamp(2, 8)).unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn is_initialized() -> bool {
    THREAD_POOL_INIT.is_completed()
}

#[cfg(target_arch = "wasm32")]
pub fn is_initialized() -> bool {
    THREAD_POOL_IS_INITIALIZED.load(std::sync::atomic::Ordering::Relaxed)
}
