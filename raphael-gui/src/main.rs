// Prevents a console from being opened on Windows
// This attribute is ignored for all other platforms
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(target_arch = "wasm32", feature(alloc_error_hook))]

mod config;
mod context;
mod macros;
mod platform;
mod saved_rotations;
mod solve;
mod storage;
mod thread_pool;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
mod native;

/// Set when an allocation fails on the web, where memory is limited to 4GB.
#[cfg(target_arch = "wasm32")]
pub static OOM_PANIC_OCCURED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(target_arch = "wasm32")]
fn main() {
    fn custom_alloc_error_hook(_layout: std::alloc::Layout) {
        OOM_PANIC_OCCURED.store(true, std::sync::atomic::Ordering::Relaxed);
        wasm_bindgen::throw_str("OOM panic");
    }
    std::alloc::set_alloc_error_hook(custom_alloc_error_hook);
    console_error_panic_hook::set_once();
    let _ = console_log::init_with_level(log::Level::Debug);

    // Remove the loading screen from `index.html`.
    if let Some(spinner) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("loading"))
    {
        spinner.remove();
    }

    dioxus::LaunchBuilder::web()
        .with_cfg(dioxus::web::Config::new().rootname("main"))
        .launch(ui::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn init_logging() {
    let mut builder = env_logger::builder();
    builder.format_timestamp(None).format_target(false);
    if std::env::var_os("RUST_LOG").is_none() {
        builder.filter_level(log::LevelFilter::Info);
    }

    // Release builds on Windows have no console, so log to a file instead.
    #[cfg(all(target_os = "windows", not(debug_assertions)))]
    if let Some(dir) = storage::data_dir()
        && std::fs::create_dir_all(&dir).is_ok()
        && let Ok(file) = std::fs::File::create(dir.join("log.txt"))
    {
        builder.target(env_logger::Target::Pipe(Box::new(file)));
        // Ensure panics are logged when detached, since the default hook outputs to stderr.
        std::panic::set_hook(Box::new(|info| {
            log::error!("{info}");
        }));
    }

    builder.init();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

    init_logging();
    native::check_for_update();

    let window = WindowBuilder::new()
        .with_title("Raphael XIV")
        .with_inner_size(LogicalSize::new(1360.0, 900.0))
        .with_min_inner_size(LogicalSize::new(360.0, 400.0));
    let config = Config::new()
        .with_window(window)
        .with_menu(None)
        .with_background_color((34, 31, 34, 255))
        .with_disable_context_menu(!cfg!(debug_assertions));
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(ui::App);
}
