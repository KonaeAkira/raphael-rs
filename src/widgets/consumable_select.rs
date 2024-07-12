use egui::{Align, Layout, Widget};
use egui_extras::Column;
use game_data::{get_item_name, Consumable, CrafterStats, Locale};

use crate::utils::contains_noncontiguous;

pub struct ConsumableSelect<'a> {
    title: &'static str,
    crafter_stats: CrafterStats,
    consumables: &'a [Consumable],
    search_text: &'a mut String,
    selected_consumable: &'a mut Option<Consumable>,
    locale: Locale,
}

impl<'a> ConsumableSelect<'a> {
    pub fn new(
        title: &'static str,
        crafter_stats: CrafterStats,
        consumables: &'a [Consumable],
        search_text: &'a mut String,
        selected_consumable: &'a mut Option<Consumable>,
        locale: Locale,
    ) -> Self {
        Self {
            title,
            crafter_stats,
            consumables,
            search_text,
            selected_consumable,
            locale,
        }
    }
}

impl<'a> Widget for ConsumableSelect<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(self.title).strong());
                    ui.label(match self.selected_consumable {
                        Some(item) => get_item_name(item.item_id, item.hq, self.locale),
                        None => "None".to_string(),
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                self.selected_consumable.is_some(),
                                egui::Button::new("Clear"),
                            )
                            .clicked()
                        {
                            *self.selected_consumable = None;
                        }
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(self.search_text);
                });
                ui.separator();

                let search_pattern = self.search_text.to_lowercase();
                let search_result: Vec<&Consumable> = self
                    .consumables
                    .iter()
                    .filter(|item| {
                        let item_name = get_item_name(item.item_id, item.hq, self.locale);
                        contains_noncontiguous(&item_name.to_lowercase(), &search_pattern)
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
                    .column(Column::exact(240.0))
                    .column(Column::remainder())
                    .min_scrolled_height(0.0);
                table.body(|body| {
                    body.rows(text_height, search_result.len(), |mut row| {
                        let item = search_result[row.index()];
                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                *self.selected_consumable = Some(*item);
                            }
                        });
                        row.col(|ui| {
                            ui.label(get_item_name(item.item_id, item.hq, self.locale));
                        });
                        row.col(|ui| {
                            ui.label(item.effect_string(
                                self.crafter_stats.craftsmanship,
                                self.crafter_stats.control,
                                self.crafter_stats.cp,
                            ));
                        });
                    });
                });
            });
        })
        .response
    }
}
