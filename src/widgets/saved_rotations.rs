use std::collections::VecDeque;

use raphael_data::{Consumable, CrafterStats, Locale, Recipe};
use raphael_sim::*;
use serde::{Deserialize, Serialize};

use crate::{app::SolverConfig, config::CrafterConfig};

use super::util;

#[derive(Debug, Serialize, Deserialize)]
pub struct Rotation {
    pub unique_id: u32,
    pub name: String,
    pub solver: String,
    pub actions: Vec<Action>,
    pub item: u32,
    pub food: Option<(u32, bool)>,
    pub potion: Option<(u32, bool)>,
    pub crafter_stats: CrafterStats,
    pub job_id: u8,
}

impl Rotation {
    pub fn new(
        name: impl Into<String>,
        actions: Vec<Action>,
        recipe: &Recipe,
        food: Option<Consumable>,
        potion: Option<Consumable>,
        crafter_config: &CrafterConfig,
        solver_config: &SolverConfig,
    ) -> Self {
        let solver_params = format!(
            "Raphael v{}{}{}{}",
            env!("CARGO_PKG_VERSION"),
            match solver_config.backload_progress {
                true => " +backload",
                false => "",
            },
            match solver_config.adversarial {
                true => " +adversarial",
                false => "",
            },
            match solver_config.minimize_steps {
                true => " +min_step",
                false => "",
            }
        );
        Self {
            unique_id: rand::random(),
            name: name.into(),
            solver: solver_params,
            actions,
            item: recipe.item_id,
            food: food.map(|consumable| (consumable.item_id, consumable.hq)),
            potion: potion.map(|consumable| (consumable.item_id, consumable.hq)),
            crafter_stats: *crafter_config.active_stats(),
            job_id: crafter_config.selected_job,
        }
    }
}

impl Clone for Rotation {
    fn clone(&self) -> Self {
        Self {
            unique_id: rand::random(),
            name: self.name.clone(),
            solver: self.solver.clone(),
            actions: self.actions.clone(),
            item: self.item,
            food: self.food,
            potion: self.potion,
            crafter_stats: self.crafter_stats,
            job_id: self.job_id,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavedRotationsData {
    pinned: Vec<Rotation>,
    solve_history: VecDeque<Rotation>,
}

impl SavedRotationsData {
    const MAX_HISTORY_SIZE: usize = 50;

    pub fn add_solved_rotation(&mut self, rotation: Rotation) {
        while self.solve_history.len() == Self::MAX_HISTORY_SIZE {
            self.solve_history.pop_back();
        }
        self.solve_history.push_front(rotation);
    }
}

struct RotationWidget<'a> {
    locale: Locale,
    pinned: &'a mut bool,
    deleted: &'a mut bool,
    rotation: &'a Rotation,
    actions: &'a mut Vec<Action>,
}

impl<'a> RotationWidget<'a> {
    pub fn new(
        locale: Locale,
        pinned: &'a mut bool,
        deleted: &'a mut bool,
        rotation: &'a Rotation,
        actions: &'a mut Vec<Action>,
    ) -> Self {
        Self {
            locale,
            pinned,
            deleted,
            rotation,
            actions,
        }
    }

    fn id_salt(&self, salt: &str) -> String {
        format!("{}_{}", self.rotation.unique_id, salt)
    }

    fn show_rotation_title(&mut self, ui: &mut egui::Ui, collapsed: &mut bool) {
        ui.horizontal(|ui| {
            util::collapse_temporary(ui, self.id_salt("collapsed").into(), collapsed);
            ui.label(egui::RichText::new(&self.rotation.name).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new("🗑")).clicked() {
                    *self.deleted = true;
                }
                ui.add_space(-3.0);
                if ui
                    .add_enabled(!*self.pinned, egui::Button::new("📌"))
                    .clicked()
                {
                    *self.pinned = true;
                }
                ui.add_space(-3.0);
                if ui.button("Load").clicked() {
                    self.actions.clone_from(&self.rotation.actions);
                }
                let duration = self
                    .rotation
                    .actions
                    .iter()
                    .map(|action| action.time_cost())
                    .sum::<u8>();
                ui.label(format!(
                    "{} steps, {} seconds",
                    self.rotation.actions.len(),
                    duration
                ));
            });
        });
    }

    fn show_info_row(
        &self,
        ui: &mut egui::Ui,
        key: impl Into<egui::WidgetText>,
        value: impl Into<egui::WidgetText>,
    ) {
        ui.horizontal(|ui| {
            let used_width = ui.label(key).rect.width();
            ui.add_space(96.0 - used_width);
            ui.label(value);
        });
    }

    fn get_consumable_name(&self, consumable: Option<(u32, bool)>) -> String {
        match consumable {
            Some((item_id, hq)) => raphael_data::get_item_name(item_id, hq, self.locale),
            None => "None".to_string(),
        }
    }

    fn show_rotation_info(&self, ui: &mut egui::Ui) {
        let stats_string = format!(
            "{} CMS, {} Control, {} CP",
            self.rotation.crafter_stats.craftsmanship,
            self.rotation.crafter_stats.control,
            self.rotation.crafter_stats.cp,
        );
        let job_string = format!(
            "Level {} {}",
            self.rotation.crafter_stats.level,
            raphael_data::get_job_name(self.rotation.job_id, self.locale)
        );
        self.show_info_row(
            ui,
            "Recipe",
            raphael_data::get_item_name(self.rotation.item, false, self.locale),
        );
        self.show_info_row(ui, "Crafter stats", stats_string);
        self.show_info_row(ui, "Job", job_string);
        self.show_info_row(ui, "Food", self.get_consumable_name(self.rotation.food));
        self.show_info_row(ui, "Potion", self.get_consumable_name(self.rotation.potion));
        self.show_info_row(ui, "Solver", &self.rotation.solver);
    }

    fn show_rotation_actions(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::horizontal()
            .id_salt(self.id_salt("scroll_area"))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for action in &self.rotation.actions {
                        let image = util::get_action_icon(*action, self.rotation.job_id)
                            .fit_to_exact_size(egui::Vec2::new(30.0, 30.0))
                            .corner_radius(4.0);
                        ui.add(image)
                            .on_hover_text(raphael_data::action_name(*action, self.locale));
                    }
                });
            });
    }
}

impl egui::Widget for RotationWidget<'_> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.vertical(|ui| {
                let mut collapsed = true;
                self.show_rotation_title(ui, &mut collapsed);
                if !collapsed {
                    ui.separator();
                    self.show_rotation_info(ui);
                }
                ui.separator();
                self.show_rotation_actions(ui);
            });
        })
        .response
    }
}

pub struct SavedRotationsWidget<'a> {
    locale: Locale,
    rotations: &'a mut SavedRotationsData,
    actions: &'a mut Vec<Action>,
}

impl<'a> SavedRotationsWidget<'a> {
    pub fn new(
        locale: Locale,
        rotations: &'a mut SavedRotationsData,
        actions: &'a mut Vec<Action>,
    ) -> Self {
        Self {
            locale,
            rotations,
            actions,
        }
    }
}

impl egui::Widget for SavedRotationsWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.group(|ui| {
                    ui.label(egui::RichText::new("Saved macros").strong());
                    ui.separator();
                    if self.rotations.pinned.is_empty() {
                        ui.label("No saved macros");
                    }
                    self.rotations.pinned.retain(|rotation| {
                        let mut deleted = false;
                        ui.add(RotationWidget::new(
                            self.locale,
                            &mut true,
                            &mut deleted,
                            rotation,
                            self.actions,
                        ));
                        !deleted
                    });
                });

                ui.add_space(5.0);

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Solve history").strong());
                        ui.label(format!(
                            "({}/{})",
                            self.rotations.solve_history.len(),
                            SavedRotationsData::MAX_HISTORY_SIZE
                        ));
                    });
                    ui.separator();
                    if self.rotations.solve_history.is_empty() {
                        ui.label("No solve history");
                    }
                    self.rotations.solve_history.retain(|rotation| {
                        let mut pinned = false;
                        let mut deleted = false;
                        ui.add(RotationWidget::new(
                            self.locale,
                            &mut pinned,
                            &mut deleted,
                            rotation,
                            self.actions,
                        ));
                        if pinned {
                            self.rotations.pinned.push(rotation.clone());
                        }
                        !pinned && !deleted
                    });
                });
            });
        })
        .response
    }
}
