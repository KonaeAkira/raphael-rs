use egui::Widget;
use game_data::{get_job_name, Locale};

use crate::config::CrafterConfig;

pub struct StatsEdit<'a> {
    locale: Locale,
    crafter_config: &'a mut CrafterConfig,
}

impl<'a> StatsEdit<'a> {
    pub fn new(locale: Locale, crafter_config: &'a mut CrafterConfig) -> Self {
        Self {
            locale,
            crafter_config,
        }
    }
}

impl<'a> Widget for StatsEdit<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            for job_id in 0..8 {
                if job_id != 0 {
                    ui.separator();
                }
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(get_job_name(job_id, self.locale)).strong());
                    if ui.button("Copy to all").clicked() {
                        let stats = self.crafter_config.crafter_stats[job_id as usize];
                        self.crafter_config.crafter_stats = [stats; 8];
                    }
                });
                let stats = &mut self.crafter_config.crafter_stats[job_id as usize];
                ui.horizontal(|ui| {
                    ui.label("Craftsmanship:");
                    ui.add(egui::DragValue::new(&mut stats.craftsmanship));
                    ui.label("Control:");
                    ui.add(egui::DragValue::new(&mut stats.control));
                    ui.label("CP:");
                    ui.add(egui::DragValue::new(&mut stats.cp));
                    ui.label("Level:");
                    ui.add(egui::DragValue::new(&mut stats.level).clamp_range(1..=100));
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut stats.manipulation, "Manipulation");
                    ui.checkbox(&mut stats.heart_and_soul, "Heart and Soul");
                    ui.checkbox(&mut stats.quick_innovation, "Quick Innovation");
                });
            }

            ui.separator().rect.width();
            ui.horizontal(|ui| {
                let button_text = "🗐 Copy Crafter Config";
                let button_response;
                if ui
                    .ctx()
                    .animate_bool_with_time(egui::Id::new(button_text), false, 0.25)
                    == 0.0
                {
                    button_response = ui.button(button_text);
                } else {
                    button_response = ui.add_enabled(false, egui::Button::new(button_text));
                }
                if button_response.clicked() {
                    ui.output_mut(|output| {
                        output.copied_text = ron::to_string(self.crafter_config).unwrap()
                    });
                    ui.ctx()
                        .animate_bool_with_time(egui::Id::new(button_text), true, 0.0);
                }

                ui.add_space(button_response.rect.width() * 0.5);
                let selected_job = self.crafter_config.selected_job;
                let hint_text = "📋 Paste Config here to Load";
                let input_string = &mut String::new();
                let input_response;
                if ui
                    .ctx()
                    .animate_bool_with_time(egui::Id::new(hint_text), false, 0.25)
                    == 0.0
                {
                    input_response =
                        ui.add(egui::TextEdit::singleline(input_string).hint_text(hint_text));
                } else {
                    input_response = ui.add_enabled(
                        false,
                        egui::TextEdit::singleline(input_string).hint_text(hint_text),
                    );
                }
                if input_response.changed() {
                    match ron::from_str(&input_string) {
                        Ok(crafter_config) => {
                            *self.crafter_config = crafter_config;
                            self.crafter_config.selected_job = selected_job;
                            ui.ctx()
                                .animate_bool_with_time(egui::Id::new(hint_text), true, 0.0);
                        }
                        Err(_) => {}
                    }
                }
            });
        })
        .response
    }
}
