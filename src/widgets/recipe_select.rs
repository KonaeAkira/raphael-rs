use egui::{
    util::cache::{ComputerMut, FrameCache},
    Align, Id, Layout, Widget,
};
use egui_extras::Column;
use game_data::{
    get_game_settings, get_item_name, get_job_name, Consumable, Ingredient, Locale, RLVLS,
};

use crate::{
    config::{CrafterConfig, QualitySource, RecipeConfiguration},
    utils::contains_noncontiguous,
};

#[derive(Default)]
struct RecipeFinder {}

impl ComputerMut<(&str, Locale), Vec<usize>> for RecipeFinder {
    fn compute(&mut self, (text, locale): (&str, Locale)) -> Vec<usize> {
        game_data::RECIPES
            .iter()
            .enumerate()
            .filter_map(|(index, recipe)| {
                let item_name = get_item_name(recipe.item_id, false, locale);
                match contains_noncontiguous(&item_name.to_lowercase(), text) {
                    true => Some(index),
                    false => None,
                }
            })
            .collect()
    }
}

type SearchCache<'a> = FrameCache<Vec<usize>, RecipeFinder>;

pub struct RecipeSelect<'a> {
    crafter_config: &'a mut CrafterConfig,
    recipe_config: &'a mut RecipeConfiguration,
    selected_food: Option<Consumable>, // used for base prog/qual display
    selected_potion: Option<Consumable>, // used for base prog/qual display
    locale: Locale,
}

impl<'a> RecipeSelect<'a> {
    pub fn new(
        crafter_config: &'a mut CrafterConfig,
        recipe_config: &'a mut RecipeConfiguration,
        selected_food: Option<Consumable>,
        selected_potion: Option<Consumable>,
        locale: Locale,
    ) -> Self {
        Self {
            crafter_config,
            recipe_config,
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

        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut search_text);
        });
        ui.separator();

        let mut search_result = Vec::new();
        ui.ctx().memory_mut(|mem| {
            let search_cache = mem.caches.cache::<SearchCache<'_>>();
            search_result = search_cache.get((&search_text.to_lowercase(), self.locale));
        });

        ui.ctx().data_mut(|data| {
            data.insert_persisted(Id::new("RECIPE_SEARCH_TEXT"), search_text);
        });

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);
        let table = egui_extras::TableBuilder::new(ui)
            .auto_shrink(false)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::exact(28.0)) // Column::auto causes jittering when scrolling
            .column(Column::remainder())
            .min_scrolled_height(0.0);
        table.body(|body| {
            body.rows(text_height, search_result.len(), |mut row| {
                let recipe = game_data::RECIPES[search_result[row.index()]];
                row.col(|ui| {
                    if ui.button("Select").clicked() {
                        self.crafter_config.selected_job = recipe.job_id;
                        *self.recipe_config = RecipeConfiguration {
                            recipe,
                            quality_source: QualitySource::HqMaterialList([0; 6]),
                        }
                    };
                });
                row.col(|ui| {
                    ui.label(get_job_name(recipe.job_id, self.locale));
                });
                row.col(|ui| {
                    ui.label(get_item_name(recipe.item_id, false, self.locale));
                });
            });
        });
    }

    fn draw_custom_recipe_select(self, ui: &mut egui::Ui) {
        self.recipe_config.recipe.item_id = 0;
        self.recipe_config.recipe.material_quality_factor = 0;
        self.recipe_config.recipe.ingredients = [Ingredient {
            item_id: 0,
            amount: 0,
        }; 6];

        let game_settings = get_game_settings(
            self.recipe_config.recipe,
            *self.crafter_config.active_stats(),
            self.selected_food,
            self.selected_potion,
            false,
        );

        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Level:");
                    ui.add(
                        egui::DragValue::new(&mut self.recipe_config.recipe.level)
                            .clamp_range(1..=100),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Recipe Level:");
                    ui.add(
                        egui::DragValue::new(&mut self.recipe_config.recipe.recipe_level)
                            .clamp_range(1..=RLVLS.len() - 1),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Progress:");
                    ui.add(egui::DragValue::new(
                        &mut self.recipe_config.recipe.progress,
                    ));
                });
                ui.horizontal(|ui| {
                    ui.label("Quality:");
                    ui.add(egui::DragValue::new(&mut self.recipe_config.recipe.quality));
                });
                if let QualitySource::Value(initial_quality) =
                    &mut self.recipe_config.quality_source
                {
                    ui.horizontal(|ui| {
                        ui.label("Initial Quality:");
                        ui.add(
                            egui::DragValue::new(initial_quality)
                                .clamp_range(0..=self.recipe_config.recipe.quality),
                        );
                    });
                }
                ui.horizontal(|ui| {
                    ui.label("Durability:");
                    ui.add(
                        egui::DragValue::new(&mut self.recipe_config.recipe.durability)
                            .clamp_range(10..=100),
                    );
                });
                ui.checkbox(&mut self.recipe_config.recipe.is_expert, "Expert recipe");
            });
            ui.separator();
            ui.vertical(|ui| {
                let mut rlvl = RLVLS[self.recipe_config.recipe.recipe_level as usize];
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
                ui.horizontal(|ui| {
                    ui.label("Progress per 100% efficiency:");
                    ui.label(egui::RichText::new(game_settings.base_progress.to_string()).strong());
                });
                ui.horizontal(|ui| {
                    ui.label("Quality per 100% efficiency:");
                    ui.label(egui::RichText::new(game_settings.base_quality.to_string()).strong());
                });
            });
        });
    }
}

impl<'a> Widget for RecipeSelect<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                let mut custom_recipe = false;
                ui.ctx().data_mut(|data| {
                    if let Some(value) = data.get_persisted::<bool>(Id::new("CUSTOM_RECIPE")) {
                        custom_recipe = value;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Recipe").strong());
                    ui.label(egui::RichText::new(get_item_name(
                        self.recipe_config.recipe.item_id,
                        false,
                        self.locale,
                    )));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.checkbox(&mut custom_recipe, "Custom Recipe").changed() {
                            self.recipe_config.quality_source = match custom_recipe {
                                true => QualitySource::Value(0),
                                false => QualitySource::HqMaterialList([0; 6]),
                            }
                        };
                    });
                });
                ui.separator();
                if custom_recipe {
                    self.draw_custom_recipe_select(ui);
                } else {
                    self.draw_normal_recipe_select(ui);
                }

                ui.ctx().data_mut(|data| {
                    data.insert_persisted(Id::new("CUSTOM_RECIPE"), custom_recipe);
                });
            });
        })
        .response
    }
}
