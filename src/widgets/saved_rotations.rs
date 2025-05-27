use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

use raphael_data::{Consumable, CrafterStats, Locale, Recipe};
use raphael_sim::*;
use serde::{Deserialize, Serialize};

use crate::{
    app::SolverConfig,
    config::{
        CrafterConfig, CustomRecipeOverridesConfiguration, QualitySource, RecipeConfiguration,
    },
};

use super::util;

fn generate_unique_rotation_id() -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    web_time::Instant::now().hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecipeInfo {
    NormalRecipe(u32),
    CustomRecipe(Recipe, CustomRecipeOverridesConfiguration),
}

impl RecipeInfo {
    pub fn create_from(
        recipe: &Recipe,
        custom_recipe_overrides_configuration: &CustomRecipeOverridesConfiguration,
    ) -> Self {
        if custom_recipe_overrides_configuration.use_custom_recipe {
            Self::CustomRecipe(*recipe, *custom_recipe_overrides_configuration)
        } else {
            Self::NormalRecipe(
                raphael_data::RECIPES
                    .entries()
                    .find_map(|(recipe_id, recipe_entry)| {
                        if recipe_entry == recipe {
                            Some(*recipe_id)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default(),
            )
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveInfo {
    pub game_settings: Settings,
    pub initial_quality: u16,
    pub solver_config: SolverConfig,
}

impl SolveInfo {
    pub fn new(
        game_settings: &Settings,
        initial_quality: u16,
        solver_config: &SolverConfig,
    ) -> Self {
        Self {
            game_settings: *game_settings,
            initial_quality,
            solver_config: *solver_config,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rotation {
    pub unique_id: u64,
    pub name: String,
    pub solver: String,
    pub actions: Vec<Action>,
    #[serde(default)]
    pub recipe_info: Option<RecipeInfo>,
    #[serde(default)]
    pub solve_info: Option<SolveInfo>,
    pub food: Option<(u32, bool)>,
    pub potion: Option<(u32, bool)>,
    pub crafter_stats: CrafterStats,
}

impl Rotation {
    pub fn new(
        name: impl Into<String>,
        actions: Vec<Action>,
        recipe_config: &RecipeConfiguration,
        custom_recipe_overrides_configuration: &CustomRecipeOverridesConfiguration,
        game_settings: &Settings,
        solver_config: &SolverConfig,
        food: Option<Consumable>,
        potion: Option<Consumable>,
        crafter_config: &CrafterConfig,
    ) -> Self {
        let solver_params = format!(
            "Raphael v{}{}{}",
            env!("CARGO_PKG_VERSION"),
            match solver_config.backload_progress {
                true => " +backload",
                false => "",
            },
            match solver_config.adversarial {
                true => " +adversarial",
                false => "",
            },
        );
        let initial_quality = crate::util::get_initial_quality(recipe_config, crafter_config);
        Self {
            unique_id: generate_unique_rotation_id(),
            name: name.into(),
            solver: solver_params,
            actions,
            recipe_info: Some(RecipeInfo::create_from(
                &recipe_config.recipe,
                custom_recipe_overrides_configuration,
            )),
            solve_info: Some(SolveInfo::new(
                game_settings,
                initial_quality,
                solver_config,
            )),
            food: food.map(|consumable| (consumable.item_id, consumable.hq)),
            potion: potion.map(|consumable| (consumable.item_id, consumable.hq)),
            crafter_stats: *crafter_config.active_stats(),
        }
    }
}

impl Clone for Rotation {
    fn clone(&self) -> Self {
        Self {
            unique_id: generate_unique_rotation_id(),
            name: self.name.clone(),
            solver: self.solver.clone(),
            actions: self.actions.clone(),
            recipe_info: self.recipe_info.clone(),
            solve_info: self.solve_info.clone(),
            food: self.food,
            potion: self.potion,
            crafter_stats: self.crafter_stats,
        }
    }
}

impl PartialEq for Rotation {
    fn eq(&self, other: &Self) -> bool {
        // unique_id & name are skipped
        self.solver == other.solver
            && self.actions == other.actions
            && self.recipe_info == other.recipe_info
            && self.solve_info == other.solve_info
            && self.food == other.food
            && self.potion == other.potion
            && self.crafter_stats == other.crafter_stats
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoadOperation {
    LoadRotation,
    LoadRotationRecipe,
    LoadRotationRecipeConsumables,
}

impl std::fmt::Display for LoadOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output_str = match self {
            Self::LoadRotation => "Load rotation",
            Self::LoadRotationRecipe => "Load rotation & recipe",
            Self::LoadRotationRecipeConsumables => "Load rotation, recipe & consumables",
        };
        write!(f, "{}", output_str)
    }
}

impl Default for LoadOperation {
    fn default() -> Self {
        Self::LoadRotation
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedRotationsConfig {
    pub load_from_saved_rotations: bool,
    pub default_load_operation: LoadOperation,
    pub max_history_size: usize,
}

impl Default for SavedRotationsConfig {
    fn default() -> Self {
        Self {
            load_from_saved_rotations: false,
            default_load_operation: LoadOperation::LoadRotation,
            max_history_size: 50,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavedRotationsData {
    pinned: Vec<Rotation>,
    solve_history: VecDeque<Rotation>,
}

impl SavedRotationsData {
    pub fn add_solved_rotation(&mut self, rotation: Rotation, config: &SavedRotationsConfig) {
        if let Some(index) = self
            .solve_history
            .iter()
            .position(|saved_rotation| *saved_rotation == rotation)
        {
            self.solve_history.remove(index);
        }

        self.solve_history.push_front(rotation);
        while self.solve_history.len() > config.max_history_size {
            self.solve_history.pop_back();
        }
    }

    pub fn find_solved_rotation(
        &self,
        game_settings: &Settings,
        initial_quality: u16,
        solver_config: &SolverConfig,
    ) -> Option<Vec<Action>> {
        let solve_info = SolveInfo::new(game_settings, initial_quality, solver_config);
        let find_and_map_rotation = |rotation: &Rotation| {
            if let (Some(saved_solver_version), Some(saved_solve_info)) =
                (rotation.solver.split(' ').nth(1), &rotation.solve_info)
                && saved_solver_version == format!("v{}", env!("CARGO_PKG_VERSION"))
                && *saved_solve_info == solve_info
            {
                Some(rotation.actions.clone())
            } else {
                None
            }
        };
        let history_search_result = self.solve_history.iter().find_map(find_and_map_rotation);
        if history_search_result.is_some() {
            history_search_result
        } else {
            self.pinned.iter().find_map(find_and_map_rotation)
        }
    }
}

struct RotationWidget<'a> {
    locale: Locale,
    config: &'a mut SavedRotationsConfig,
    pinned: &'a mut bool,
    deleted: &'a mut bool,
    rotation: &'a Rotation,
    actions: &'a mut Vec<Action>,
    crafter_config: &'a mut CrafterConfig,
    recipe_config: &'a mut RecipeConfiguration,
    custom_recipe_overrides_config: &'a mut CustomRecipeOverridesConfiguration,
    selected_food: &'a mut Option<Consumable>,
    selected_potion: &'a mut Option<Consumable>,
}

impl<'a> RotationWidget<'a> {
    pub fn new(
        locale: Locale,
        config: &'a mut SavedRotationsConfig,
        pinned: &'a mut bool,
        deleted: &'a mut bool,
        rotation: &'a Rotation,
        actions: &'a mut Vec<Action>,
        crafter_config: &'a mut CrafterConfig,
        recipe_config: &'a mut RecipeConfiguration,
        custom_recipe_overrides_config: &'a mut CustomRecipeOverridesConfiguration,
        selected_food: &'a mut Option<Consumable>,
        selected_potion: &'a mut Option<Consumable>,
    ) -> Self {
        Self {
            locale,
            config,
            pinned,
            deleted,
            rotation,
            actions,
            crafter_config,
            recipe_config,
            custom_recipe_overrides_config,
            selected_food,
            selected_potion,
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
                let load_button_response = ui.button("Load");
                let mut selected_load_operation = None;
                load_button_response.context_menu(|ui| {
                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        ui.close();
                    }
                    for saved_rotation_load_operation in [
                        LoadOperation::LoadRotation,
                        LoadOperation::LoadRotationRecipe,
                        LoadOperation::LoadRotationRecipeConsumables,
                    ] {
                        let text = format!("{}", saved_rotation_load_operation);
                        if ui.button(text).clicked() {
                            selected_load_operation = Some(saved_rotation_load_operation);
                        }
                    }
                    if self.rotation.recipe_info.is_none() {
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new("⚠ pre-v0.21.0 rotation. No recipe data.")
                                    .small()
                                    .color(ui.visuals().warn_fg_color),
                            )
                            .wrap(),
                        );
                    }
                });
                if load_button_response.clicked() {
                    selected_load_operation = Some(self.config.default_load_operation);
                }
                if let Some(load_operation) = selected_load_operation {
                    match load_operation {
                        LoadOperation::LoadRotation => {
                            self.actions.clone_from(&self.rotation.actions);
                        }
                        LoadOperation::LoadRotationRecipe => {
                            self.actions.clone_from(&self.rotation.actions);
                            self.load_saved_recipe();
                        }
                        LoadOperation::LoadRotationRecipeConsumables => {
                            self.actions.clone_from(&self.rotation.actions);
                            self.load_saved_recipe();
                            self.load_saved_consumables();
                        }
                    }
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

    fn load_saved_recipe(&mut self) {
        if let Some(recipe_configuration) = &self.rotation.recipe_info {
            match recipe_configuration {
                RecipeInfo::NormalRecipe(recipe_id) => {
                    if let Some(recipe) = raphael_data::RECIPES.get(recipe_id) {
                        *self.recipe_config = RecipeConfiguration {
                            recipe: *recipe,
                            quality_source: QualitySource::HqMaterialList([0; 6]),
                        };
                        self.crafter_config.selected_job = recipe.job_id;
                        self.custom_recipe_overrides_config.use_custom_recipe = false;
                    } else {
                        log::debug!("Unable to find recipe with recipe_id={:?}", recipe_id);
                    }
                }
                RecipeInfo::CustomRecipe(recipe, custom_recipe_overrides_config) => {
                    *self.recipe_config = RecipeConfiguration {
                        recipe: *recipe,
                        quality_source: QualitySource::Value(0),
                    };
                    *self.custom_recipe_overrides_config = *custom_recipe_overrides_config;
                    self.crafter_config.selected_job = recipe.job_id;
                }
            }
        }
    }

    fn load_saved_consumables(&mut self) {
        *self.selected_food = self.rotation.food.and_then(|(item_id, hq)| {
            raphael_data::MEALS
                .iter()
                .find(|food| food.item_id == item_id && food.hq == hq)
                .copied()
        });
        *self.selected_potion = self.rotation.potion.and_then(|(item_id, hq)| {
            raphael_data::POTIONS
                .iter()
                .find(|potion| potion.item_id == item_id && potion.hq == hq)
                .copied()
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
            Some((item_id, hq)) => raphael_data::get_item_name(item_id, hq, self.locale)
                .unwrap_or("Unknown item".to_owned()),
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
        let recipe = self.get_recipe();
        let job_id = recipe.map(|recipe| recipe.job_id);
        let job_string = format!(
            "Level {} {}",
            self.rotation.crafter_stats.level,
            job_id.map_or("", |job_id| raphael_data::get_job_name(job_id, self.locale))
        );
        if let Some(recipe) = recipe {
            self.show_info_row(
                ui,
                "Recipe",
                raphael_data::get_item_name(recipe.item_id, false, self.locale)
                    .unwrap_or("Unknown item".to_owned()),
            );
        }
        self.show_info_row(ui, "Crafter stats", stats_string);
        self.show_info_row(ui, "Job", job_string);
        self.show_info_row(ui, "Food", self.get_consumable_name(self.rotation.food));
        self.show_info_row(ui, "Potion", self.get_consumable_name(self.rotation.potion));
        self.show_info_row(ui, "Solver", &self.rotation.solver);
    }

    fn show_rotation_actions(&self, ui: &mut egui::Ui) {
        let job_id = self.get_recipe().map_or(0, |recipe| recipe.job_id);
        egui::ScrollArea::horizontal()
            .id_salt(self.id_salt("scroll_area"))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    for action in &self.rotation.actions {
                        let image = util::get_action_icon(*action, job_id)
                            .fit_to_exact_size(egui::Vec2::new(30.0, 30.0))
                            .corner_radius(4.0);
                        ui.add(image)
                            .on_hover_text(raphael_data::action_name(*action, self.locale));
                    }
                });
            });
    }

    fn get_recipe(&self) -> Option<&Recipe> {
        self.rotation
            .recipe_info
            .as_ref()
            .and_then(|recipe_config| match recipe_config {
                RecipeInfo::NormalRecipe(recipe_id) => raphael_data::RECIPES.get(recipe_id),
                RecipeInfo::CustomRecipe(recipe, _) => Some(recipe),
            })
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
    config: &'a mut SavedRotationsConfig,
    rotations: &'a mut SavedRotationsData,
    actions: &'a mut Vec<Action>,
    crafter_config: &'a mut CrafterConfig,
    recipe_config: &'a mut RecipeConfiguration,
    custom_recipe_overrides_config: &'a mut CustomRecipeOverridesConfiguration,
    selected_food: &'a mut Option<Consumable>,
    selected_potion: &'a mut Option<Consumable>,
}

impl<'a> SavedRotationsWidget<'a> {
    pub fn new(
        locale: Locale,
        config: &'a mut SavedRotationsConfig,
        rotations: &'a mut SavedRotationsData,
        actions: &'a mut Vec<Action>,
        crafter_config: &'a mut CrafterConfig,
        recipe_config: &'a mut RecipeConfiguration,
        custom_recipe_overrides_config: &'a mut CustomRecipeOverridesConfiguration,
        selected_food: &'a mut Option<Consumable>,
        selected_potion: &'a mut Option<Consumable>,
    ) -> Self {
        Self {
            locale,
            config,
            rotations,
            actions,
            crafter_config,
            recipe_config,
            custom_recipe_overrides_config,
            selected_food,
            selected_potion,
        }
    }
}

impl egui::Widget for SavedRotationsWidget<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.vertical(|ui| {
            ui.style_mut().visuals.collapsing_header_frame = true;
            ui.collapsing("Settings", |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
                ui.vertical(|ui| {
                    ui.checkbox(
                        &mut self.config.load_from_saved_rotations,
                        "Load saved rotations when initiating solve",
                    );
                    ui.separator();
                    ui.label("Default operation on clicking Load button:");
                    for saved_rotation_load_operation in [
                        LoadOperation::LoadRotation,
                        LoadOperation::LoadRotationRecipe,
                        LoadOperation::LoadRotationRecipeConsumables,
                    ] {
                        let text = format!("{}", saved_rotation_load_operation);
                        ui.selectable_value(
                            &mut self.config.default_load_operation,
                            saved_rotation_load_operation,
                            text,
                        );
                    }

                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(
                                "⚠ Rotations saved before v0.21.0 do not contain the necessary information to load recipe data.",
                            )
                            .small()
                            .color(ui.visuals().warn_fg_color),
                        )
                        .wrap(),
                    );
                });
            });
            ui.separator();
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
                            self.config,
                            &mut true,
                            &mut deleted,
                            rotation,
                            self.actions,
                            self.crafter_config,
                            self.recipe_config,
                            self.custom_recipe_overrides_config,
                            self.selected_food,
                            self.selected_potion,
                        ));
                        !deleted
                    });
                });

                ui.add_space(5.0);

                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Solve history").strong());
                        ui.horizontal(|ui| {
                            ui.style_mut().spacing.item_spacing.x = 3.0;
                            ui.label("(");
                            ui.add_enabled(false, egui::DragValue::new(&mut self.rotations.solve_history.len()));
                            ui.label("/");
                            ui.add(
                                egui::DragValue::new(&mut self.config.max_history_size)
                                    .range(20..=200),
                            );
                            ui.label(")");
                        });
                        if self.rotations.solve_history.len() > self.config.max_history_size {
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(
                                        "⚠ Oldest rotations will be lost on next solve",
                                    )
                                    .small()
                                    .color(ui.visuals().warn_fg_color),
                                )
                                .wrap(),
                            );
                        }
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
                            self.config,
                            &mut pinned,
                            &mut deleted,
                            rotation,
                            self.actions,
                            self.crafter_config,
                            self.recipe_config,
                            self.custom_recipe_overrides_config,
                            self.selected_food,
                            self.selected_potion,
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
