use egui::{Align, Color32, Id, Layout, Rounding, Widget};
use game_data::{action_name, get_job_name, Item, Locale};
use simulator::{Action, Settings, SimulationState};

use crate::{
    app::SolverConfig,
    config::{CrafterConfig, QualityTarget},
};

use super::HelpText;

#[cfg(target_arch = "wasm32")]
const BASE_ASSET_PATH: &str = env!("BASE_URL");
#[cfg(not(target_arch = "wasm32"))]
const BASE_ASSET_PATH: &str = "file://./assets";

pub struct Simulator<'a> {
    settings: &'a Settings,
    initial_quality: u16,
    solver_config: SolverConfig,
    crafter_config: &'a CrafterConfig,
    actions: &'a [Action],
    item: &'a Item,
    locale: Locale,
}

impl<'a> Simulator<'a> {
    pub fn new(
        settings: &'a Settings,
        initial_quality: u16,
        solver_config: SolverConfig,
        crafter_config: &'a CrafterConfig,
        actions: &'a [Action],
        item: &'a Item,
        locale: Locale,
    ) -> Self {
        Self {
            settings,
            initial_quality,
            solver_config,
            crafter_config,
            actions,
            item,
            locale,
        }
    }
}

impl Widget for Simulator<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (game_state, errors) =
            SimulationState::from_macro_continue_on_error(self.settings, self.actions);

        let max_progress = self.settings.max_progress;
        let progress = game_state.progress;

        let max_quality = self.settings.max_quality;
        let quality = game_state.quality + self.initial_quality;

        let prog_qual_dbg_text = t!(
            "info.base_progress_and_quality",
            progress = self.settings.base_progress,
            quality = self.settings.base_quality
        );

        let mut config_changed_warning = false;
        ui.ctx().data(|data| {
            if let Some((settings, initial_quality, solver_config)) =
                data.get_temp::<(Settings, u16, SolverConfig)>(Id::new("LAST_SOLVE_PARAMS"))
            {
                config_changed_warning = settings != *self.settings
                    || initial_quality != self.initial_quality
                    || solver_config != self.solver_config;
            }
        });
        if self.actions.is_empty() {
            config_changed_warning = false;
        }

        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(t!("label.simulation")).strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add_visible(
                                config_changed_warning,
                                egui::Label::new(
                                    egui::RichText::new(t!("warning.outdated_parameters"))
                                        .small()
                                        .color(ui.visuals().warn_fg_color),
                                ),
                            );
                        });
                    });
                    ui.separator();
                    let mut progress_bar_rect = egui::Rect {
                        min: egui::Pos2 { x: 0.0, y: 0.0 },
                        max: egui::Pos2 { x: 0.0, y: 0.0 },
                    };
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("progress")));
                        let mut text = format!("{: >5} / {}", progress, max_progress);
                        if progress >= max_progress {
                            text.push_str(&format!("  (+{} overflow)", progress - max_progress));
                        }
                        progress_bar_rect = ui
                            .add(
                                egui::ProgressBar::new(progress as f32 / max_progress as f32)
                                    .text(text)
                                    .rounding(Rounding::ZERO),
                            )
                            .on_hover_text_at_pointer(prog_qual_dbg_text.clone())
                            .rect;
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("quality")));
                        let mut text = format!("{: >5} / {}", quality, max_quality);
                        if quality >= max_quality {
                            text.push_str(&t!("label.overflow", overflow = quality - max_quality));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_sized(
                                progress_bar_rect.size(),
                                egui::ProgressBar::new(quality as f32 / max_quality as f32)
                                    .text(text)
                                    .rounding(Rounding::ZERO),
                            )
                            .on_hover_text_at_pointer(prog_qual_dbg_text);
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", t!("durability")));
                        let max_durability = self.settings.max_durability;
                        let durability = game_state.durability;
                        ui.add(
                            egui::ProgressBar::new(durability as f32 / max_durability as f32)
                                .text(format!("{: >2} / {}", durability, max_durability))
                                .rounding(Rounding::ZERO)
                                .desired_width(120.0),
                        );
                        ui.label(format!("{}:", t!("cp")));
                        let max_cp = self.settings.max_cp;
                        let cp = game_state.cp;
                        ui.add(
                            egui::ProgressBar::new(cp as f32 / max_cp as f32)
                                .text(format!("{: >3} / {}", cp, max_cp))
                                .rounding(Rounding::ZERO)
                                .desired_width(120.0),
                        );

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.add(HelpText::new(if self.settings.adversarial {
                                t!("info.adversarial_simulation")
                            } else {
                                t!("info.normal_simulation")
                            }));
                            if game_state.is_final(self.settings) {
                                if progress < max_progress {
                                    ui.label(egui::RichText::new(t!("sim_result.failed")).strong());
                                } else if self.item.always_collectable {
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
                                        egui::RichText::new(t!(
                                            "sim_result.collectable",
                                            tier = tier
                                        ))
                                        .strong(),
                                    );
                                } else {
                                    let hq = game_data::hq_percentage(quality, max_quality);
                                    ui.label(
                                        egui::RichText::new(t!("sim_result.hq", hq = hq)).strong(),
                                    );
                                }
                            }
                        });
                    });
                });
            });
            ui.group(|ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.set_height(30.0);
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        for (action, error) in self.actions.iter().zip(errors.into_iter()) {
                            let image_path = format!(
                                "{}/action-icons/{}/{}.webp",
                                BASE_ASSET_PATH,
                                get_job_name(self.crafter_config.selected_job, Locale::EN),
                                action_name(*action, Locale::EN)
                            );

                            ui.add(
                                egui::Image::new(image_path)
                                    .fit_to_exact_size(egui::Vec2::new(30.0, 30.0))
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
