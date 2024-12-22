use egui::Widget;
use game_data::{action_name, get_job_name, Locale};
use simulator::Action;

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

impl Widget for StatsEdit<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            for job_id in 0..8 {
                if job_id != 0 {
                    ui.separator();
                }
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(get_job_name(job_id, self.locale)).strong());
                    if ui.button(t!("label.copy_to_all")).clicked() {
                        let stats = self.crafter_config.crafter_stats[job_id as usize];
                        self.crafter_config.crafter_stats = [stats; 8];
                    }
                });
                let stats = &mut self.crafter_config.crafter_stats[job_id as usize];
                ui.horizontal(|ui| {
                    ui.label(format!("{}:", t!("craftsmanship")));
                    ui.add(egui::DragValue::new(&mut stats.craftsmanship).range(1..=9999));
                    ui.label(format!("{}:", t!("control")));
                    ui.add(egui::DragValue::new(&mut stats.control).range(1..=9999));
                    ui.label(format!("{}:", t!("cp")));
                    ui.add(egui::DragValue::new(&mut stats.cp).range(1..=999));
                    ui.label(format!("{}:", t!("job_level")));
                    ui.add(egui::DragValue::new(&mut stats.level).range(1..=100));
                });
                ui.horizontal(|ui| {
                    ui.checkbox(
                        &mut stats.manipulation,
                        action_name(Action::Manipulation, self.locale),
                    );
                    ui.checkbox(
                        &mut stats.heart_and_soul,
                        action_name(Action::HeartAndSoul, self.locale),
                    );
                    ui.checkbox(
                        &mut stats.quick_innovation,
                        action_name(Action::QuickInnovation, self.locale),
                    );
                });
            }

            ui.separator().rect.width();
            ui.horizontal(|ui| {
                let button_text = t!("label.copy_crafter_config");
                let copy_id = egui::Id::new("config_copy");
                let button_response =
                    if ui.ctx().animate_bool_with_time(copy_id, false, 0.25) == 0.0 {
                        ui.button(button_text)
                    } else {
                        ui.add_enabled(false, egui::Button::new(button_text))
                    };
                if button_response.clicked() {
                    ui.output_mut(|output| {
                        output.copied_text = ron::to_string(self.crafter_config).unwrap()
                    });
                    ui.ctx().animate_bool_with_time(copy_id, true, 0.0);
                }

                ui.add_space(button_response.rect.width() * 0.5);
                let selected_job = self.crafter_config.selected_job;
                let hint_text = t!("label.paste_crafter_config");
                let paste_id = egui::Id::new("config_paste");
                let input_string = &mut String::new();
                let input_response =
                    if ui.ctx().animate_bool_with_time(paste_id, false, 0.25) == 0.0 {
                        ui.add(egui::TextEdit::singleline(input_string).hint_text(hint_text))
                    } else {
                        ui.add_enabled(
                            false,
                            egui::TextEdit::singleline(input_string).hint_text(hint_text),
                        )
                    };
                if input_response.changed() {
                    if let Ok(crafter_config) = ron::from_str(input_string) {
                        *self.crafter_config = crafter_config;
                        self.crafter_config.selected_job = selected_job;
                        ui.ctx().animate_bool_with_time(paste_id, true, 0.0);
                    }
                }
            });
        })
        .response
    }
}
