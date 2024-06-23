#[cfg(not(target_arch = "wasm32"))]
compile_error!("This binary can only be built for WASM");

#[cfg(target_arch = "wasm32")]
fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    Box::new(raphael_gui::MacroSolverApp::new(cc))
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}
