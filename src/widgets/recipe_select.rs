use egui::{Align, Layout, Widget};
use egui_extras::Column;
use game_data::{get_item_name, get_job_name, Ingredient, Locale, RecipeConfiguration, RLVLS};

use crate::utils::contains_noncontiguous;

pub struct RecipeSelect<'a> {
    selected_job: &'a mut u8,
    recipe_config: &'a mut RecipeConfiguration,
    custom_recipe: &'a mut bool,
    search_text: &'a mut String,
    locale: Locale,
}

impl<'a> RecipeSelect<'a> {
    pub fn new(
        selected_job: &'a mut u8,
        recipe_config: &'a mut RecipeConfiguration,
        custom_recipe: &'a mut bool,
        search_text: &'a mut String,
        locale: Locale,
    ) -> Self {
        Self {
            selected_job,
            recipe_config,
            custom_recipe,
            search_text,
            locale,
        }
    }

    fn draw_normal_recipe_select(self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(self.search_text);
        });
        ui.separator();

        let search_pattern = self.search_text.to_lowercase();
        let search_result: Vec<usize> = game_data::RECIPES
            .iter()
            .enumerate()
            .filter_map(|(index, recipe)| {
                let item_name = get_item_name(recipe.item_id, false, self.locale);
                match contains_noncontiguous(&item_name.to_lowercase(), &search_pattern) {
                    true => Some(index),
                    false => None,
                }
            })
            .collect();

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
            .column(Column::remainder())
            .min_scrolled_height(0.0);
        table.body(|body| {
            body.rows(text_height, search_result.len(), |mut row| {
                let recipe = game_data::RECIPES[search_result[row.index()]];
                row.col(|ui| {
                    let job_label =
                        egui::RichText::new(get_job_name(recipe.job_id, self.locale)).monospace();
                    if ui.button(job_label).clicked() {
                        *self.selected_job = recipe.job_id;
                        *self.recipe_config = RecipeConfiguration {
                            recipe,
                            hq_ingredients: [0; 6],
                        }
                    };
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
        self.recipe_config.hq_ingredients = [0; 6];
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Level:");
                    ui.add(
                        egui::DragValue::new(&mut self.recipe_config.recipe.level).range(1..=100),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Recipe Level:");
                    ui.add(
                        egui::DragValue::new(&mut self.recipe_config.recipe.recipe_level)
                            .range(1..=RLVLS.len() - 1),
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
                ui.horizontal(|ui| {
                    ui.label("Durability:");
                    ui.add(
                        egui::DragValue::new(&mut self.recipe_config.recipe.durability)
                            .range(10..=100),
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
            });
        });
    }
}

impl<'a> Widget for RecipeSelect<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Recipe").strong());
                    ui.label(egui::RichText::new(get_item_name(
                        self.recipe_config.recipe.item_id,
                        false,
                        self.locale,
                    )));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.checkbox(self.custom_recipe, "Custom Recipe");
                    });
                });
                ui.separator();
                if *self.custom_recipe {
                    self.draw_custom_recipe_select(ui);
                } else {
                    self.draw_normal_recipe_select(ui);
                }
            });
        })
        .response
    }
}
