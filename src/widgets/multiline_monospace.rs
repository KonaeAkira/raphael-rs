/// Copyable multiline monospace text box
pub struct MultilineMonospace {
    id: egui::Id,
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
    pub fn new(id: egui::Id, text: String) -> Self {
        Self {
            id,
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
        let text_changed = ui.ctx().data_mut(|mem| {
            let previous_text_mem_id = self.id.with("previous_text");
            let text_changed = mem
                .get_temp::<String>(previous_text_mem_id)
                .is_some_and(|previous_text| previous_text != self.text);
            mem.insert_temp(previous_text_mem_id, self.text.clone());
            text_changed
        });
        ui.group(|ui| {
            ui.set_max_size(ui.available_size());
            ui.horizontal_top(|ui| {
                let mut scroll_area = egui::ScrollArea::new(self.scroll_direction_enabled)
                    .max_height(self.max_height);
                if text_changed {
                    // Reset scroll if text has changed.
                    scroll_area = scroll_area.scroll_offset([0.0, 0.0].into());
                }
                scroll_area.show(ui, |ui| {
                    ui.monospace(&self.text);
                });
                ui.put(
                    egui::Rect::from_min_max(ui.max_rect().left_top(), ui.max_rect().right_top()),
                    CopyTextButton::new(&self.text),
                );
            });
        })
        .response
    }
}
