use egui::{Align, Id, Layout, Widget};
use egui_extras::Column;
use game_data::{get_item_name, Consumable, CrafterStats, Locale};

use crate::utils::contains_noncontiguous;

pub struct ConsumableSelect<'a> {
    title: &'static str,
    crafter_stats: CrafterStats,
    consumables: &'a [Consumable],
    selected_consumable: &'a mut Option<Consumable>,
    locale: Locale,
    allow_noncontiguous: &'a bool,
}

impl<'a> ConsumableSelect<'a> {
    pub fn new(
        title: &'static str,
        crafter_stats: CrafterStats,
        consumables: &'a [Consumable],
        selected_consumable: &'a mut Option<Consumable>,
        locale: Locale,
        allow_noncontiguous: &'a bool
    ) -> Self {
        Self {
            title,
            crafter_stats,
            consumables,
            selected_consumable,
            locale,
            allow_noncontiguous,
        }
    }
}

impl<'a> Widget for ConsumableSelect<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = Id::new(self.title);
        let mut search_text: String = ui.ctx().data(|data| data.get_temp(id).unwrap_or_default());
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
                    if ui.text_edit_singleline(&mut search_text).changed() {
                        ui.ctx()
                            .data_mut(|data| data.insert_temp(id, search_text.clone()));
                    }
                });
                ui.separator();

                let search_pattern = search_text.to_lowercase();
                let search_result: Vec<&Consumable> = self
                    .consumables
                    .iter()
                    .filter(|item| {
                        let item_name = get_item_name(item.item_id, item.hq, self.locale);
                        match self.allow_noncontiguous {
                            false => {item_name.to_lowercase().contains(&search_pattern)}
                            true => {contains_noncontiguous(&item_name.to_lowercase(), &search_pattern)}
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
