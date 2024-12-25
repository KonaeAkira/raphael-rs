use egui::{Align, Color32, Id, Layout, Rounding, Widget};
use game_data::{action_name, Item, Locale};
use simulator::{Action, Settings, SimulationState};

use crate::{
    app::SolverConfig,
    config::{CrafterConfig, QualityTarget},
};

use super::HelpText;

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
                            let image = get_action_icon(*action, self.crafter_config.selected_job)
                                .fit_to_exact_size(egui::Vec2::new(30.0, 30.0))
                                .rounding(4.0)
                                .tint(match error {
                                    Ok(_) => Color32::WHITE,
                                    Err(_) => Color32::from_rgb(255, 96, 96),
                                });
                            ui.add(image)
                                .on_hover_text(action_name(*action, self.locale));
                        }
                    });
                });
            });
        })
        .response
    }
}

#[cfg(target_arch = "wasm32")]
fn get_action_icon(action: Action, job_id: u8) -> egui::Image<'static> {
    let image_path = format!(
        "{}/action-icons/{}/{}.webp",
        env!("BASE_URL"),
        game_data::get_job_name(job_id, Locale::EN),
        game_data::action_name(action, Locale::EN)
    );
    egui::Image::new(image_path)
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! action_icon {
    ( $name:literal, $job_id:expr ) => {
        match $job_id {
            0 => egui::include_image!(concat!("../../assets/action-icons/CRP/", $name, ".webp")),
            1 => egui::include_image!(concat!("../../assets/action-icons/BSM/", $name, ".webp")),
            2 => egui::include_image!(concat!("../../assets/action-icons/ARM/", $name, ".webp")),
            3 => egui::include_image!(concat!("../../assets/action-icons/GSM/", $name, ".webp")),
            4 => egui::include_image!(concat!("../../assets/action-icons/LTW/", $name, ".webp")),
            5 => egui::include_image!(concat!("../../assets/action-icons/WVR/", $name, ".webp")),
            6 => egui::include_image!(concat!("../../assets/action-icons/ALC/", $name, ".webp")),
            7 => egui::include_image!(concat!("../../assets/action-icons/CUL/", $name, ".webp")),
            _ => {
                log::warn!("Unknown job id {}. Falling back to job id 0.", $job_id);
                egui::include_image!(concat!("../../assets/action-icons/CRP/", $name, ".webp"))
            }
        }
    };
}

#[cfg(not(target_arch = "wasm32"))]
fn get_action_icon(action: Action, job_id: u8) -> egui::Image<'static> {
    egui::Image::new(match action {
        Action::BasicSynthesis => action_icon!("Basic Synthesis", job_id),
        Action::BasicTouch => action_icon!("Basic Touch", job_id),
        Action::MasterMend => action_icon!("Master's Mend", job_id),
        Action::Observe => action_icon!("Observe", job_id),
        Action::TricksOfTheTrade => action_icon!("Tricks of the Trade", job_id),
        Action::WasteNot => action_icon!("Waste Not", job_id),
        Action::Veneration => action_icon!("Veneration", job_id),
        Action::StandardTouch => action_icon!("Standard Touch", job_id),
        Action::GreatStrides => action_icon!("Great Strides", job_id),
        Action::Innovation => action_icon!("Innovation", job_id),
        Action::WasteNot2 => action_icon!("Waste Not II", job_id),
        Action::ByregotsBlessing => action_icon!("Byregot's Blessing", job_id),
        Action::PreciseTouch => action_icon!("Precise Touch", job_id),
        Action::MuscleMemory => action_icon!("Muscle Memory", job_id),
        Action::CarefulSynthesis => action_icon!("Careful Synthesis", job_id),
        Action::Manipulation => action_icon!("Manipulation", job_id),
        Action::PrudentTouch => action_icon!("Prudent Touch", job_id),
        Action::AdvancedTouch => action_icon!("Advanced Touch", job_id),
        Action::Reflect => action_icon!("Reflect", job_id),
        Action::PreparatoryTouch => action_icon!("Preparatory Touch", job_id),
        Action::Groundwork => action_icon!("Groundwork", job_id),
        Action::DelicateSynthesis => action_icon!("Delicate Synthesis", job_id),
        Action::IntensiveSynthesis => action_icon!("Intensive Synthesis", job_id),
        Action::TrainedEye => action_icon!("Trained Eye", job_id),
        Action::HeartAndSoul => action_icon!("Heart and Soul", job_id),
        Action::PrudentSynthesis => action_icon!("Prudent Synthesis", job_id),
        Action::TrainedFinesse => action_icon!("Trained Finesse", job_id),
        Action::RefinedTouch => action_icon!("Refined Touch", job_id),
        Action::QuickInnovation => action_icon!("Quick Innovation", job_id),
        Action::ImmaculateMend => action_icon!("Immaculate Mend", job_id),
        Action::TrainedPerfection => action_icon!("Trained Perfection", job_id),
    })
}
