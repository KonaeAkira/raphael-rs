use std::collections::HashMap;

use egui::{Align, Color32, Layout, Rounding, TextureHandle, Widget};
use game_data::{action_name, Item, Locale};
use simulator::{Action, Settings, SimulationState};

use crate::config::QualityTarget;

use super::HelpText;

pub struct Simulator<'a> {
    settings: &'a Settings,
    actions: &'a [Action],
    item: &'a Item,
    action_icons: &'a HashMap<Action, TextureHandle>,
    locale: Locale,
}

impl<'a> Simulator<'a> {
    pub fn new(
        settings: &'a Settings,
        actions: &'a [Action],
        item: &'a Item,
        action_icons: &'a HashMap<Action, TextureHandle>,
        locale: Locale,
    ) -> Self {
        Self {
            settings,
            actions,
            item,
            action_icons,
            locale,
        }
    }
}

impl<'a> Widget for Simulator<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (game_state, errors) = SimulationState::from_macro_continue_on_error(
            &Settings {
                max_quality: u16::MAX,
                ..*self.settings
            },
            self.actions,
        );

        let max_progress = self.settings.max_progress;
        let clamped_progress = self.settings.max_progress - game_state.missing_progress;

        let max_quality = self.settings.max_quality;
        let quality = u16::MAX - game_state.missing_quality;
        let clamped_quality = std::cmp::min(max_quality, quality);

        let prog_qual_dbg_text = format!(
            "Progress per 100% efficiency: {}\nQuality per 100% efficiency: {}",
            self.settings.base_progress, self.settings.base_quality
        );

        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Simulation").strong());
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Progress:");
                        ui.add(
                            egui::ProgressBar::new(clamped_progress as f32 / max_progress as f32)
                                .text(format!("{} / {}", clamped_progress, max_progress))
                                .rounding(Rounding::ZERO),
                        )
                        .on_hover_text_at_pointer(&prog_qual_dbg_text);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Quality:");
                        match quality == u16::MAX {
                            true => ui.add(
                                egui::ProgressBar::new(clamped_quality as f32 / max_quality as f32)
                                    .text(format!(
                                        "{} / {}  (max quality guaranteed)",
                                        clamped_quality, max_quality,
                                    ))
                                    .rounding(Rounding::ZERO),
                            ),
                            false => ui.add(
                                egui::ProgressBar::new(clamped_quality as f32 / max_quality as f32)
                                    .text(format!(
                                        "{} / {}  (+{} overflow)",
                                        clamped_quality,
                                        max_quality,
                                        quality.saturating_sub(max_quality)
                                    ))
                                    .rounding(Rounding::ZERO),
                            ),
                        }
                        .on_hover_text_at_pointer(&prog_qual_dbg_text);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Durability:");
                        let max_durability = self.settings.max_durability;
                        let durability = game_state.durability;
                        ui.add(
                            egui::ProgressBar::new(durability as f32 / max_durability as f32)
                                .text(format!("{} / {}", durability, max_durability))
                                .rounding(Rounding::ZERO)
                                .desired_width(120.0),
                        );
                        ui.label("CP:");
                        let max_cp = self.settings.max_cp;
                        let cp = game_state.cp;
                        ui.add(
                            egui::ProgressBar::new(cp as f32 / max_cp as f32)
                                .text(format!("{} / {}", cp, max_cp))
                                .rounding(Rounding::ZERO)
                                .desired_width(120.0),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add(HelpText::new(
                                "Calculated assuming Normal conditon on every step",
                            ));
                            if self.item.is_collectable {
                                let t1 = QualityTarget::CollectableT1
                                    .get_target(self.settings.max_quality);
                                let t2 = QualityTarget::CollectableT2
                                    .get_target(self.settings.max_quality);
                                let t3 = QualityTarget::CollectableT3
                                    .get_target(self.settings.max_quality);
                                let tier = match quality {
                                    quality if quality >= t3 => 3,
                                    quality if quality >= t2 => 2,
                                    quality if quality >= t1 => 1,
                                    _ => 0,
                                };
                                ui.label(
                                    egui::RichText::new(format!("Tier {tier} collectable reached"))
                                        .strong(),
                                );
                            } else {
                                let hq = match game_state.missing_progress {
                                    0 => game_data::hq_percentage(clamped_quality, max_quality),
                                    _ => 0,
                                };
                                ui.label(egui::RichText::new(format!("{hq}% HQ")).strong());
                            }
                        });
                    });
                });
            });
            ui.add_space(5.5);
            ui.group(|ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.set_height(30.0);
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        for (action, error) in self.actions.iter().zip(errors.into_iter()) {
                            ui.add(
                                egui::Image::new(self.action_icons.get(action).unwrap())
                                    .max_height(30.0)
                                    .rounding(4.0)
                                    .tint(match error {
                                        Ok(_) => Color32::WHITE,
                                        Err(_) => Color32::from_rgb(255, 96, 96),
                                    }),
                            )
                            .on_hover_text(action_name(*action, self.locale));
                        }
                    });
                });
            });
        })
        .response
    }
}
