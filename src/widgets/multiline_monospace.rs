/// Copyable multiline monospace text box
pub struct MultilineMonospace {
    text: String,
    max_height: f32,
    scroll_direction_enabled: egui::Vec2b,
}

struct CopyTextButton<'a> {
    text: &'a str,
}

impl<'a> CopyTextButton<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }
}

impl egui::Widget for CopyTextButton<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
            if ui.ctx().animate_bool_with_time(ui.id(), false, 1.5) == 0.0 {
                if ui.button("🗐").clicked() {
                    ui.ctx().copy_text(self.text.to_owned());
                    ui.ctx().animate_bool_with_time(ui.id(), true, 0.0);
                }
            } else {
                ui.add_enabled(false, egui::Button::new("🗐"));
            }
        })
        .response
    }
}

impl MultilineMonospace {
    pub fn new(text: String) -> Self {
        Self {
            text,
            max_height: f32::INFINITY,
            scroll_direction_enabled: egui::Vec2b::default(),
        }
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn scrollable(mut self, direction_enabled: impl Into<egui::Vec2b>) -> Self {
        self.scroll_direction_enabled = direction_enabled.into();
        self
    }
}

impl egui::Widget for MultilineMonospace {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = egui::Id::new(&self.text);
        ui.push_id(id, |ui| {
            ui.group(|ui| {
                ui.set_max_size(ui.available_size());
                ui.horizontal_top(|ui| {
                    egui::ScrollArea::new(self.scroll_direction_enabled)
                        .max_height(self.max_height)
                        .show(ui, |ui| {
                            ui.monospace(&self.text);
                        });
                    ui.put(
                        egui::Rect::from_min_max(
                            ui.max_rect().left_top(),
                            ui.max_rect().right_top(),
                        ),
                        CopyTextButton::new(&self.text),
                    );
                });
            });
        })
        .response
    }
}
