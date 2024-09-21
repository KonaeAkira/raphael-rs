use game_data::{get_item_name, Locale};

pub struct ItemNameLabel {
    item_id: u32,
    text: String,
}

impl ItemNameLabel {
    pub fn new(item_id: u32, hq: bool, locale: Locale) -> Self {
        Self {
            item_id,
            text: get_item_name(item_id, hq, locale),
        }
    }
}

impl egui::Widget for ItemNameLabel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = egui::Id::new(ui.id().value() ^ u64::from(self.item_id));

        let response;
        if ui.ctx().animate_bool_with_time(id, false, 0.25) == 0.0 {
            response = ui
                .add(egui::Label::new(egui::RichText::new(&self.text)).sense(egui::Sense::click()));
        } else {
            response = ui.add(
                egui::Label::new(
                    egui::RichText::new(&self.text).color(ui.style().visuals.weak_text_color()),
                )
                .sense(egui::Sense::click()),
            );
        }

        response.context_menu(|ui| {
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                ui.close_menu();
            }
            let mut selection_made = false;
            if ui.button(t!("label.copy_item_name")).clicked() {
                let copy_item_name: String = self
                    .text
                    .trim_end_matches(&[' ', game_data::HQ_ICON_CHAR, game_data::CL_ICON_CHAR])
                    .to_string();
                ui.output_mut(|output| output.copied_text = copy_item_name);
                ui.close_menu();
                selection_made = true;
            }
            ui.separator();
            if ui.button(t!("label.copy_item_id")).clicked() {
                ui.output_mut(|output| output.copied_text = self.item_id.to_string());
                ui.close_menu();
                selection_made = true;
            }

            if selection_made {
                ui.ctx().animate_bool_with_time(id, true, 0.0);
            }
        });
        response
    }
}
