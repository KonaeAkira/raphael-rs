#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
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
fn get_canvas() -> Option<web_sys::HtmlCanvasElement> {
    use web_sys::wasm_bindgen::JsCast;
    let document = web_sys::window()?.document()?;
    let canvas = document.get_element_by_id("the_canvas_id")?;
    canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                get_canvas().expect("Failed to get HTML canvas"),
                eframe::WebOptions::default(),
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    Ok(Box::new(raphael_xiv::MacroSolverApp::new(cc)))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
