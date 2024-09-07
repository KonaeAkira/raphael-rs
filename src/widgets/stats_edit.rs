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
                        let mut non_specialist_stats = stats.clone();
                        let mut specialist_stats = stats.clone();
                        if self.crafter_config.specialists[job_id as usize] {
                            non_specialist_stats.craftsmanship =
                                stats.craftsmanship.saturating_sub(20);
                            non_specialist_stats.control = stats.control.saturating_sub(20);
                            non_specialist_stats.cp = stats.cp.saturating_sub(15);
                        } else {
                            specialist_stats.craftsmanship = stats.craftsmanship.saturating_add(20);
                            specialist_stats.control = stats.control.saturating_add(20);
                            specialist_stats.cp = stats.cp.saturating_add(15);
                        }

                        for i in 0..8 {
                            self.crafter_config.crafter_stats[i] =
                                match self.crafter_config.specialists[i] {
                                    true => specialist_stats,
                                    false => non_specialist_stats,
                                }
                        }
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
                    if ui
                        .checkbox(
                            &mut self.crafter_config.specialists[job_id as usize],
                            "Specialist",
                        )
                        .changed()
                    {
                        let change = match self.crafter_config.specialists[job_id as usize] {
                            true => (20, 15),
                            false => (-20, -15),
                        };

                        stats.craftsmanship = stats.craftsmanship.saturating_add_signed(change.0);
                        stats.control = stats.control.saturating_add_signed(change.0);
                        stats.cp = stats.cp.saturating_add_signed(change.1);
                    }
                });
            }
        })
        .response
    }
}
