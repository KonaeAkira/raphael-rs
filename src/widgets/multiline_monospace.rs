/// Copyable multiline monospace text box
pub struct MultilineMonospace {
    text: String,
    max_height: f32,
    scrollable: bool,
}

impl MultilineMonospace {
    pub fn new(text: String) -> Self {
        Self {
            text,
            max_height: f32::INFINITY,
            scrollable: false,
        }
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn scrollable(mut self, scrollable: bool) -> Self {
        self.scrollable = scrollable;
        self
    }
}

impl egui::Widget for MultilineMonospace {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = egui::Id::new(&self.text);
        ui.push_id(id, |ui| {
            ui.group(|ui| {
                ui.horizontal_top(|ui| {
                    egui::ScrollArea::new([self.scrollable, self.scrollable])
                        .max_height(self.max_height)
                        .show(ui, |ui| {
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
                });
            });
        })
        .response
    }
}
