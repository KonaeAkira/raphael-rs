use egui::{
    Align, Id, Layout, Widget,
    util::cache::{ComputerMut, FrameCache},
};
use egui_extras::Column;
use raphael_data::{
    Consumable, CustomRecipeOverrides, Ingredient, Locale, RLVLS, find_recipes, get_game_settings,
    get_job_name,
};

use crate::config::{
    CrafterConfig, CustomRecipeOverridesConfiguration, QualitySource, RecipeConfiguration,
};

use super::{ItemNameLabel, util};

#[derive(Default)]
struct RecipeFinder {}

impl ComputerMut<(&str, Locale), Vec<u32>> for RecipeFinder {
    fn compute(&mut self, (text, locale): (&str, Locale)) -> Vec<u32> {
        find_recipes(text, locale)
    }
}

type SearchCache<'a> = FrameCache<Vec<u32>, RecipeFinder>;

pub struct RecipeSelect<'a> {
    crafter_config: &'a mut CrafterConfig,
    recipe_config: &'a mut RecipeConfiguration,
    custom_recipe_overrides_config: &'a mut CustomRecipeOverridesConfiguration,
    selected_food: Option<Consumable>, // used for base prog/qual display
    selected_potion: Option<Consumable>, // used for base prog/qual display
    locale: Locale,
}

impl<'a> RecipeSelect<'a> {
    pub fn new(
        crafter_config: &'a mut CrafterConfig,
        recipe_config: &'a mut RecipeConfiguration,
        custom_recipe_overrides_config: &'a mut CustomRecipeOverridesConfiguration,
        selected_food: Option<Consumable>,
        selected_potion: Option<Consumable>,
        locale: Locale,
    ) -> Self {
        Self {
            crafter_config,
            recipe_config,
            custom_recipe_overrides_config,
            selected_food,
            selected_potion,
            locale,
        }
    }

    fn draw_normal_recipe_select(self, ui: &mut egui::Ui) {
        let mut search_text = String::new();
        ui.ctx().data_mut(|data| {
            if let Some(text) = data.get_persisted::<String>(Id::new("RECIPE_SEARCH_TEXT")) {
                search_text = text;
            }
        });

        if egui::TextEdit::singleline(&mut search_text)
            .desired_width(f32::INFINITY)
            .hint_text("🔍 Search")
            .ui(ui)
            .changed()
        {
            search_text = search_text.replace('\0', "");
        }
        ui.separator();

        let mut search_result = Vec::new();
        ui.ctx().memory_mut(|mem| {
            let search_cache = mem.caches.cache::<SearchCache<'_>>();
            search_result = search_cache.get((&search_text, self.locale));
        });

        ui.ctx().data_mut(|data| {
            data.insert_persisted(Id::new("RECIPE_SEARCH_TEXT"), search_text);
        });

        let line_height = ui.spacing().interact_size.y;
        let line_spacing = ui.spacing().item_spacing.y;
        let table_height = 6.3 * line_height + 6.0 * line_spacing;

        // Column::remainder().clip(true) is buggy when resizing the table
        // manually calculate the width of the last col to avoid janky behavior when resizing tables
        // this is a workaround until this bug is fixed in egui_extras
        let spacing = 2.0 * ui.spacing().item_spacing.x;
        let item_name_width = (ui.available_width() - 42.0 - 28.0 - spacing).max(0.0);

        let table = egui_extras::TableBuilder::new(ui)
            .id_salt("RECIPE_SELECT_TABLE")
            .auto_shrink(false)
            .striped(true)
            .column(Column::exact(42.0))
            .column(Column::exact(28.0))
            .column(Column::exact(item_name_width))
            .min_scrolled_height(table_height)
            .max_scroll_height(table_height);
        table.body(|body| {
            body.rows(line_height, search_result.len(), |mut row| {
                let recipe_id = search_result[row.index()];
                let recipe = raphael_data::RECIPES[&recipe_id];
                row.col(|ui| {
                    if ui.button("Select").clicked() {
                        self.crafter_config.selected_job = recipe.job_id;
                        *self.recipe_config = RecipeConfiguration {
                            recipe,
                            quality_source: QualitySource::HqMaterialList([0; 6]),
                        }
                    }
                });
                row.col(|ui| {
                    ui.label(get_job_name(recipe.job_id, self.locale));
                });
                row.col(|ui| {
                    ui.add(ItemNameLabel::new(recipe.item_id, false, self.locale));
                });
            });
        });
    }

    fn draw_custom_recipe_select(self, ui: &mut egui::Ui) {
        let custom_recipe_overrides =
            &mut self.custom_recipe_overrides_config.custom_recipe_overrides;
        let default_game_settings = get_game_settings(
            self.recipe_config.recipe,
            None,
            *self.crafter_config.active_stats(),
            self.selected_food,
            self.selected_potion,
        );
        let use_base_increase_overrides = self
            .custom_recipe_overrides_config
            .use_base_increase_overrides;
        ui.label(egui::RichText::new("⚠ Only use custom recipes if you are an advanced user or if new recipes haven't been added yet. Patch 7.25 recipes are now fully supported.").small().color(ui.visuals().warn_fg_color));
        ui.separator();
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                let mut recipe_job_level = RLVLS
                    .get(self.recipe_config.recipe.recipe_level as usize)
                    .unwrap()
                    .job_level;
                ui.horizontal(|ui| {
                    ui.label("Level:");
                    ui.add_enabled_ui(use_base_increase_overrides, |ui| {
                        ui.add(egui::DragValue::new(&mut recipe_job_level).range(1..=100));
                        if use_base_increase_overrides {
                            self.recipe_config.recipe.recipe_level =
                                raphael_data::LEVEL_ADJUST_TABLE[recipe_job_level as usize];
                        }
                    });
                });
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!use_base_increase_overrides, |ui| {
                        ui.label("Recipe Level:");
                        let mut rlvl_drag_value_widget =
                            egui::DragValue::new(&mut self.recipe_config.recipe.recipe_level)
                                .range(1..=RLVLS.len() - 1);
                        if use_base_increase_overrides && recipe_job_level >= 50 {
                            rlvl_drag_value_widget = rlvl_drag_value_widget.suffix("+");
                        }
                        ui.add(rlvl_drag_value_widget);
                    });
                });
                ui.horizontal(|ui| {
                    ui.label("Progress:");
                    ui.add(egui::DragValue::new(
                        &mut custom_recipe_overrides.max_progress_override,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Quality:");
                    ui.add(egui::DragValue::new(
                        &mut custom_recipe_overrides.max_quality_override,
                    ));
                });
                if let QualitySource::Value(initial_quality) =
                    &mut self.recipe_config.quality_source
                {
                    ui.horizontal(|ui| {
                        ui.label("Initial Quality:");
                        ui.add(egui::DragValue::new(initial_quality));
                    });
                }
                ui.horizontal(|ui| {
                    ui.label("Durability:");
                    ui.add(
                        egui::DragValue::new(&mut custom_recipe_overrides.max_durability_override)
                            .range(10..=100),
                    );
                });
                ui.checkbox(&mut self.recipe_config.recipe.is_expert, "Expert recipe");
            });
            ui.separator();
            ui.vertical(|ui| {
                let mut rlvl = RLVLS[self.recipe_config.recipe.recipe_level as usize];
                ui.add_enabled_ui(!use_base_increase_overrides, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Progress divider");
                        ui.add_enabled(false, egui::DragValue::new(&mut rlvl.progress_div));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Quality divider");
                        ui.add_enabled(false, egui::DragValue::new(&mut rlvl.quality_div));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Progress modifier");
                        ui.add_enabled(false, egui::DragValue::new(&mut rlvl.progress_mod));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Quality modifier");
                        ui.add_enabled(false, egui::DragValue::new(&mut rlvl.quality_mod));
                    });
                });

                ui.horizontal(|ui| {
                    ui.label("Progress per 100% efficiency:");
                    if !use_base_increase_overrides {
                        ui.label(
                            egui::RichText::new(default_game_settings.base_progress.to_string())
                                .strong(),
                        );
                    } else {
                        let mut base_progress_override_value =
                            custom_recipe_overrides.base_progress_override.unwrap();
                        ui.add(
                            egui::DragValue::new(&mut base_progress_override_value).range(0..=999),
                        );
                        custom_recipe_overrides.base_progress_override =
                            Some(base_progress_override_value);
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Quality per 100% efficiency:");
                    if !use_base_increase_overrides {
                        ui.label(
                            egui::RichText::new(default_game_settings.base_quality.to_string())
                                .strong(),
                        );
                    } else {
                        let mut base_quality_override_value =
                            custom_recipe_overrides.base_quality_override.unwrap();
                        ui.add(
                            egui::DragValue::new(&mut base_quality_override_value).range(0..=999),
                        );
                        custom_recipe_overrides.base_quality_override =
                            Some(base_quality_override_value);
                    }
                });
                if ui
                    .checkbox(
                        &mut self
                            .custom_recipe_overrides_config
                            .use_base_increase_overrides,
                        "Override per 100% efficiency values",
                    )
                    .changed()
                {
                    if self
                        .custom_recipe_overrides_config
                        .use_base_increase_overrides
                    {
                        custom_recipe_overrides.base_progress_override =
                            Some(default_game_settings.base_progress);
                        custom_recipe_overrides.base_quality_override =
                            Some(default_game_settings.base_quality);
                    } else {
                        custom_recipe_overrides.base_progress_override = None;
                        custom_recipe_overrides.base_quality_override = None;
                    }
                }
            });
        });
    }
}

impl Widget for RecipeSelect<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.vertical(|ui| {
                let mut collapsed = false;

                ui.horizontal(|ui| {
                    util::collapse_persisted(
                        ui,
                        Id::new("RECIPE_SEARCH_COLLAPSED"),
                        &mut collapsed,
                    );
                    ui.label(egui::RichText::new("Recipe").strong());
                    ui.add(ItemNameLabel::new(
                        self.recipe_config.recipe.item_id,
                        false,
                        self.locale,
                    ));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let use_custom_recipe =
                            &mut self.custom_recipe_overrides_config.use_custom_recipe;
                        if ui.checkbox(use_custom_recipe, "Custom").changed() {
                            if *use_custom_recipe {
                                let default_game_settings = get_game_settings(
                                    self.recipe_config.recipe,
                                    None,
                                    *self.crafter_config.active_stats(),
                                    self.selected_food,
                                    self.selected_potion,
                                );

                                self.recipe_config.recipe.req_craftsmanship = 0;
                                self.recipe_config.recipe.req_control = 0;
                                self.recipe_config.recipe.max_level_scaling = 0;
                                self.recipe_config.recipe.material_factor = 0;
                                self.recipe_config.recipe.ingredients = [Ingredient::default(); 6];

                                // Only set appropriate overrides when switching from normal to custom recipe
                                // Switching back does not currently restore the other parameters, e.g. rlvl back to default
                                if self.recipe_config.recipe.item_id != 0 {
                                    self.custom_recipe_overrides_config.custom_recipe_overrides =
                                        CustomRecipeOverrides {
                                            max_progress_override: default_game_settings
                                                .max_progress,
                                            max_quality_override: default_game_settings.max_quality,
                                            max_durability_override: default_game_settings
                                                .max_durability,
                                            ..Default::default()
                                        };
                                    if self
                                        .custom_recipe_overrides_config
                                        .use_base_increase_overrides
                                    {
                                        self.custom_recipe_overrides_config
                                            .custom_recipe_overrides
                                            .base_progress_override =
                                            Some(default_game_settings.base_progress);
                                        self.custom_recipe_overrides_config
                                            .custom_recipe_overrides
                                            .base_quality_override =
                                            Some(default_game_settings.base_quality);
                                    }
                                }

                                self.recipe_config.quality_source = QualitySource::Value(0);

                                self.recipe_config.recipe.item_id = 0;
                            } else {
                                self.recipe_config.quality_source =
                                    QualitySource::HqMaterialList([0; 6]);
                            }
                        }
                    });
                });

                if collapsed {
                    return;
                }

                ui.separator();

                if self.custom_recipe_overrides_config.use_custom_recipe {
                    self.draw_custom_recipe_select(ui);
                } else {
                    self.draw_normal_recipe_select(ui);
                }
            });
        })
        .response
    }
}
