/// Copyable multiline monospace text box
pub struct MultilineMonospace {
    text: String,
}

impl MultilineMonospace {
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl egui::Widget for MultilineMonospace {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = egui::Id::new(&self.text);
        ui.group(|ui| {
            ui.horizontal_top(|ui| {
                ui.monospace(&self.text);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.ctx().animate_bool_with_time(id, false, 1.5) == 0.0 {
                        if ui.button("🗐").clicked() {
                            ui.ctx().copy_text(self.text);
                            ui.ctx().animate_bool_with_time(id, true, 0.0);
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("🗐"));
                    }
                });
            });
        })
        .response
    }
}
