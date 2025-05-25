// code borrows heavily from the `egui_demo_app`, specifically `backend_panel.rs` & `frame_history.rs`

use egui::util::History;

#[derive(Debug, Clone, PartialEq)]
enum RunMode {
    Reactive,
    Continuous,
}

#[derive(Debug, Clone, Default)]
struct EguiWindows {
    settings: bool,
    inspection: bool,
    memory: bool,
    // `output_events` is skipped
}

impl EguiWindows {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.settings, "🔧 Settings");
        ui.checkbox(&mut self.inspection, "🔍 Inspection");
        ui.checkbox(&mut self.memory, "📝 Memory");

        let ctx = ui.ctx();
        egui::Window::new("🔧 Settings")
            .open(&mut self.settings)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.settings_ui(ui);
            });

        egui::Window::new("🔍 Inspection")
            .open(&mut self.inspection)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.inspection_ui(ui);
            });

        egui::Window::new("📝 Memory")
            .open(&mut self.memory)
            .resizable(false)
            .show(ctx, |ui| {
                ctx.memory_ui(ui);
            });
    }
}

#[derive(Debug, Clone)]
pub struct RenderInfoState {
    run_mode: RunMode,
    cpu_usage_history: History<f32>,
    frame_time_history: History<f32>,
    last_frames_time: Option<f64>,

    y_lim: f32,
    plot_cpu_usage: bool,
    plot_frame_time: bool,

    egui_windows: EguiWindows,
}

impl RenderInfoState {
    pub fn update(&mut self, ctx: &egui::Context, frame: &eframe::Frame) {
        let now = ctx.input(|i| i.time);
        if let Some(cpu_usage) = frame.info().cpu_usage {
            let last_frames_time = self.last_frames_time.unwrap_or_default();
            self.cpu_usage_history.add(last_frames_time, cpu_usage);
            self.frame_time_history
                .add(last_frames_time, (now - last_frames_time) as f32);
        }
        self.last_frames_time = Some(now);

        match self.run_mode {
            RunMode::Continuous => {
                ctx.request_repaint();
            }
            RunMode::Reactive => {}
        }
    }

    pub fn mean_cpu_usage(&self) -> f32 {
        self.cpu_usage_history.average().unwrap_or_default()
    }

    pub fn theoretical_fps(&self) -> f32 {
        self.cpu_usage_history
            .average()
            .map_or(0.0, |average_frame_time| 1.0 / average_frame_time)
    }

    pub fn mean_frame_time(&self) -> f32 {
        self.cpu_usage_history
            .mean_time_interval()
            .unwrap_or_default()
    }

    pub fn average_fps(&self) -> f32 {
        self.cpu_usage_history
            .mean_time_interval()
            .map_or(0.0, |mean_time_interval| 1.0 / mean_time_interval)
    }
}

impl Default for RenderInfoState {
    fn default() -> Self {
        Self {
            run_mode: RunMode::Reactive,
            cpu_usage_history: History::new(0..300, 1.0),
            frame_time_history: History::new(0..300, 1.0),
            last_frames_time: None,

            y_lim: 0.010,
            plot_cpu_usage: true,
            plot_frame_time: false,

            egui_windows: EguiWindows::default(),
        }
    }
}

pub struct RenderInfo<'a> {
    state: &'a mut RenderInfoState,
}

impl<'a> RenderInfo<'a> {
    pub fn new(state: &'a mut RenderInfoState) -> Self {
        Self { state }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.state.update(ui.ctx(), frame);

        ui.vertical_centered(|ui| {
            ui.heading("Rendering / UI");
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.radio_value(&mut self.state.run_mode, RunMode::Reactive, "Reactive");
            ui.radio_value(&mut self.state.run_mode, RunMode::Continuous, "Continuous");
        });
        ui.separator();

        ui.label("Stats:");
        egui::Grid::new("adapter_info").show(ui, |ui| {
            ui.label("Mean CPU usage:");
            ui.label(
                egui::RichText::new(format!("{:5.2}", self.state.mean_cpu_usage() * 1000.0))
                    .monospace(),
            );
            ui.label("ms");
            ui.end_row();

            ui.label("Mean frame time:");
            ui.label(
                egui::RichText::new(format!("{:5.2}", self.state.mean_frame_time() * 1000.0))
                    .monospace(),
            );
            ui.label("ms");
            ui.end_row();

            match self.state.run_mode {
                RunMode::Reactive => {
                    ui.label("Theoretical FPS:");
                    ui.label(
                        egui::RichText::new(format!("{:6.1}", self.state.theoretical_fps()))
                            .monospace(),
                    );
                }
                RunMode::Continuous => {
                    ui.label("FPS:");
                    ui.label(
                        egui::RichText::new(format!("{:6.1}", self.state.average_fps()))
                            .monospace(),
                    );
                }
            }

            ui.end_row();
        });
        ui.collapsing("📊 History", |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.state.plot_cpu_usage, "CPU usage");
                ui.checkbox(&mut self.state.plot_frame_time, "Frame time");
            });
            ui.horizontal(|ui| {
                ui.label("Y axis limit:");
                let mut y_lim = self.state.y_lim * 1000.0;
                ui.add(
                    egui::DragValue::new(&mut y_lim)
                        .range(1.0..=30.0)
                        .suffix(" ms"),
                );
                self.state.y_lim = y_lim * 0.001;
            });

            self.graph(ui);
        });
        ui.separator();

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(wgpu_render_state) = frame.wgpu_render_state() {
            let adapter = &wgpu_render_state.adapter;
            let eframe::wgpu::AdapterInfo {
                name,
                vendor,
                device,
                device_type,
                driver,
                driver_info,
                backend,
            } = &adapter.get_info();

            ui.horizontal(|ui| {
                ui.label("Backend:");
                ui.label(format!("{backend:?}")).on_hover_ui(|ui| {
                    egui::Grid::new("adapter_info").show(ui, |ui| {
                        ui.label("Device Type:");
                        ui.label(format!("{device_type:?}"));
                        ui.end_row();

                        if !name.is_empty() {
                            ui.label("Name:");
                            ui.label(format!("{name:?}"));
                            ui.end_row();
                        }
                        if !driver.is_empty() {
                            ui.label("Driver:");
                            ui.label(format!("{driver:?}"));
                            ui.end_row();
                        }
                        if !driver_info.is_empty() {
                            ui.label("Driver info:");
                            ui.label(format!("{driver_info:?}"));
                            ui.end_row();
                        }
                        if *vendor != 0 {
                            ui.label("Vendor:");
                            ui.label(format!("0x{vendor:04X}"));
                            ui.end_row();
                        }
                        if *device != 0 {
                            ui.label("Device:");
                            ui.label(format!("0x{device:02X}"));
                            ui.end_row();
                        }
                    });
                });
            });
        }
        ui.separator();

        ui.label("egui windows:");
        ui.add(egui::Label::new(egui::RichText::new("⚠ Changing settings in these windows may not allow for them to be reset unless the entire stored state is deleted").color(ui.visuals().warn_fg_color).small()).wrap());
        self.state.egui_windows.ui(ui);
    }

    fn graph(&self, ui: &mut egui::Ui) -> egui::Response {
        use egui::{Rect, Sense, Shape, Stroke, TextStyle, emath, epaint, pos2, vec2};

        let cpu_usage_history = &self.state.cpu_usage_history;

        let size = vec2(ui.available_size_before_wrap().x, 100.0);
        let (rect, response) = ui.allocate_at_least(size, Sense::hover());
        let style = ui.style().noninteractive();

        let graph_top_cpu_usage = self.state.y_lim; // 10.0 ms
        let graph_rect =
            Rect::from_x_y_ranges(cpu_usage_history.max_age()..=0.0, graph_top_cpu_usage..=0.0);
        let to_screen = emath::RectTransform::from_to(graph_rect, rect);

        // Preallocates space for box + hover line & text + samples
        let number_of_shapes = 3 + 2 * cpu_usage_history.len();
        let mut shapes = Vec::with_capacity(number_of_shapes);
        shapes.push(Shape::Rect(epaint::RectShape::new(
            rect,
            style.corner_radius,
            ui.visuals().extreme_bg_color,
            ui.style().noninteractive().bg_stroke,
            egui::StrokeKind::Middle,
        )));

        if self.state.plot_cpu_usage {
            let color = ui.visuals().text_color();
            let line_stroke = Stroke::new(1.0, color);
            self.add_samples(
                &mut shapes,
                cpu_usage_history,
                &to_screen,
                line_stroke,
                true,
            );
        }
        if self.state.plot_frame_time {
            let color = ui.visuals().weak_text_color();
            let line_stroke = Stroke::new(1.0, color);
            self.add_samples(
                &mut shapes,
                &self.state.frame_time_history,
                &to_screen,
                line_stroke,
                false,
            );
        }

        // Add hover line & text
        let rect = rect.shrink(4.0);
        let color = ui.visuals().strong_text_color();
        let line_stroke = Stroke::new(1.0, color);
        if let Some(pointer_pos) = response.hover_pos() {
            let y = pointer_pos.y;
            shapes.push(Shape::line_segment(
                [pos2(rect.left(), y), pos2(rect.right(), y)],
                line_stroke,
            ));
            let cpu_usage = to_screen.inverse().transform_pos(pointer_pos).y;
            let text = format!("{:4.1} ms", 1000.0 * cpu_usage);
            shapes.push(ui.fonts(|f| {
                Shape::text(
                    f,
                    pos2(rect.left(), y),
                    egui::Align2::LEFT_BOTTOM,
                    text,
                    TextStyle::Monospace.resolve(ui.style()),
                    color,
                )
            }));
        }

        ui.painter().extend(shapes);

        response
    }

    fn add_samples(
        &self,
        shapes: &mut Vec<egui::Shape>,
        history: &History<f32>,
        to_screen: &egui::emath::RectTransform,
        line_stroke: egui::Stroke,
        assume_value_continuity: bool,
    ) {
        let right_side_time = self.state.last_frames_time.unwrap_or_default();
        let mut history_peekable_iter = history.iter().peekable();
        for _ in 0..history.len() {
            if let Some((time, value)) = history_peekable_iter.next() {
                let age = (right_side_time - time) as f32;
                let pos = to_screen.transform_pos_clamped(egui::Pos2::new(age, value));

                let mut draw_disconnected = true;
                if let Some((following_time, following_value)) = history_peekable_iter.peek() {
                    let time_distance = following_time - time;
                    let value_difference = following_value - value;
                    // connect samples with line if they are in a window of rougly one frame at 60 fps
                    // rounded and with additional ~1ms, since normal debug builds can miss this window quite often
                    if time_distance < 0.018
                        && (assume_value_continuity || value_difference.abs() < 0.020)
                    {
                        let following_age = (right_side_time - following_time) as f32;
                        let following_pos = to_screen.transform_pos_clamped(egui::Pos2::new(
                            following_age,
                            *following_value,
                        ));

                        shapes.push(egui::Shape::line_segment([pos, following_pos], line_stroke));
                        draw_disconnected = false;
                    }
                }

                if draw_disconnected {
                    shapes.push(egui::Shape::circle_filled(pos, 1.0, line_stroke.color));
                }
            } else {
                break;
            }
        }
    }
}
