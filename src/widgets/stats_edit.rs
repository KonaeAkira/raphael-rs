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
                    ui.add(egui::DragValue::new(&mut stats.level));
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut stats.manipulation, "Manipulation");
                });
            }
        })
        .response
    }
}
