use egui::Widget;
use egui_extras::Column;
use game_data::{get_item_name, Locale, RecipeConfiguration};

use crate::utils::contains_noncontiguous;

pub struct RecipeSelect<'a> {
    recipe_config: &'a mut RecipeConfiguration,
    search_text: &'a mut String,
    locale: Locale,
}

impl<'a> RecipeSelect<'a> {
    pub fn new(
        recipe_config: &'a mut RecipeConfiguration,
        search_text: &'a mut String,
        locale: Locale,
    ) -> Self {
        Self {
            recipe_config,
            search_text,
            locale,
        }
    }
}

impl<'a> Widget for RecipeSelect<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Recipe").strong());
                    ui.label(egui::RichText::new(get_item_name(
                        self.recipe_config.item_id,
                        false,
                        self.locale,
                    )));
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(self.search_text);
                });
                ui.separator();

                let search_pattern = self.search_text.to_lowercase();
                let mut search_result: Vec<u32> = game_data::RECIPES
                    .keys()
                    .copied()
                    .filter(|item_id| {
                        let item_name = get_item_name(*item_id, false, self.locale);
                        contains_noncontiguous(&item_name.to_lowercase(), &search_pattern)
                    })
                    .collect();
                search_result.sort();

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
                        let item_id = search_result[row.index()];
                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                log::debug!("{}", get_item_name(item_id, false, self.locale));
                                *self.recipe_config = RecipeConfiguration {
                                    item_id,
                                    recipe: *game_data::RECIPES.get(&item_id).unwrap(),
                                    hq_ingredients: [0; 6],
                                }
                            };
                        });
                        row.col(|ui| {
                            ui.label(get_item_name(item_id, false, self.locale));
                        });
                    });
                });
            });
        })
        .response
    }
}
