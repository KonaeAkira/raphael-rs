// Prevents a console from being opened on Windows
// This attribute is ignored for all other platforms
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![cfg_attr(target_arch = "wasm32", feature(alloc_error_hook))]

#[cfg(all(target_os = "windows", not(debug_assertions)))]
fn init_logging() {
    // Ensure app storage folder exists
    let mut file_path = eframe::storage_dir("Raphael XIV").unwrap();
    if !std::fs::exists(&file_path).unwrap() {
        let creation_result = std::fs::create_dir_all(&file_path);
        assert!(creation_result.is_ok());
    }

    // Get log file target. File is truncated if it already exists
    file_path.push("log.txt");
    let log_file_target = Box::new(std::fs::File::create(file_path).unwrap());

    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .target(env_logger::Target::Pipe(log_file_target))
        .init();

    // Ensure panics are logged when detached, since the default hook outputs to stderr
    // Backtraces are currently not generated
    std::panic::set_hook(Box::new(|info| {
        log::error!("{}", info);
    }));
}

#[cfg(target_arch = "wasm32")]
fn init_logging() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
}

#[cfg(not(any(
    all(target_os = "windows", not(debug_assertions)),
    target_arch = "wasm32"
)))]
fn init_logging() {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    init_logging();

    let present_mode =
        if let Some(present_mode_env_var) = std::env::var("RAPHAEL_PRESENT_MODE").ok() {
            match present_mode_env_var.as_str() {
                "AutoVsync" => eframe::wgpu::PresentMode::AutoVsync,
                "AutoNoVsync" => eframe::wgpu::PresentMode::AutoNoVsync,
                "Fifo" => eframe::wgpu::PresentMode::Fifo,
                "FifoRelaxed" => eframe::wgpu::PresentMode::FifoRelaxed,
                "Immediate" => eframe::wgpu::PresentMode::Immediate,
                "Mailbox" => eframe::wgpu::PresentMode::Mailbox,
                _ => panic!("Unknown present mode: {}", present_mode_env_var),
            }
        } else {
            eframe::wgpu::PresentMode::default()
        };
    let desired_maximum_frame_latency = if let Some(desired_maximum_frame_latency_env_var) =
        std::env::var("RAPHAEL_DESIRED_MAXIMUM_FRAME_LATENCY").ok()
    {
        match desired_maximum_frame_latency_env_var.parse() {
            Ok(value) => Some(value),
            Err(e) => panic!(
                "Failed to parse desired maximum frame latency with error: {}",
                e
            ),
        }
    } else {
        None
    };
    let wgpu_options = eframe::egui_wgpu::WgpuConfiguration {
        present_mode,
        desired_maximum_frame_latency,
        ..Default::default()
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        wgpu_options,
        ..Default::default()
    };
    eframe::run_native(
        "Raphael XIV",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(raphael_xiv::MacroSolverApp::new(cc)))
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    fn custom_alloc_error_hook(_layout: std::alloc::Layout) {
        raphael_xiv::OOM_PANIC_OCCURED.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    std::alloc::set_alloc_error_hook(custom_alloc_error_hook);

    init_logging();

    fn get_canvas() -> Option<web_sys::HtmlCanvasElement> {
        use web_sys::wasm_bindgen::JsCast;
        let document = web_sys::window()?.document()?;
        let canvas = document.get_element_by_id("the_canvas_id")?;
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
    }

    fn remove_loading_spinner() -> Option<()> {
        let document = web_sys::window()?.document()?;
        let spinner = document.get_element_by_id("spinner")?;
        spinner.remove();
        Some(())
    }

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                get_canvas().unwrap(),
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    Ok(Box::new(raphael_xiv::MacroSolverApp::new(cc)))
                }),
            )
            .await;
        remove_loading_spinner();
        if let Err(error) = start_result {
            let mut message = format!("Failed to start app: {error:?}");
            log::error!("{message}");
            message.push_str("\n\nIn case the error is \"WebGL not supported\":");
            message.push_str("\nPlease enable WebGL support in your browser.");
            web_sys::window()
                .unwrap()
                .alert_with_message(&message)
                .unwrap();
        }
    });
}
