use egui::{
    util::cache::{ComputerMut, FrameCache},
    Align, Id, Layout, Widget,
};
use egui_extras::Column;
use game_data::{find_potions, Consumable, CrafterStats, Locale};

use super::ItemNameLabel;

#[derive(Default)]
struct PotionFinder {}

impl ComputerMut<(&str, Locale), Vec<usize>> for PotionFinder {
    fn compute(&mut self, (text, locale): (&str, Locale)) -> Vec<usize> {
        find_potions(text, locale)
    }
}

type PotionSearchCache<'a> = FrameCache<Vec<usize>, PotionFinder>;

pub struct PotionSelect<'a> {
    crafter_stats: CrafterStats,
    selected_consumable: &'a mut Option<Consumable>,
    locale: Locale,
}

impl<'a> PotionSelect<'a> {
    pub fn new(
        crafter_stats: CrafterStats,
        selected_consumable: &'a mut Option<Consumable>,
        locale: Locale,
    ) -> Self {
        Self {
            crafter_stats,
            selected_consumable,
            locale,
        }
    }
}

impl<'a> Widget for PotionSelect<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Potion").strong());
                    match self.selected_consumable {
                        None => {
                            ui.label("None".to_string());
                        }
                        Some(item) => {
                            ui.add(ItemNameLabel::new(item.item_id, item.hq, self.locale));
                        }
                    }
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

                let id = Id::new("POTION_SEARCH_TEXT");

                let mut search_text = String::new();
                ui.ctx().data_mut(|data| {
                    if let Some(text) = data.get_persisted::<String>(id) {
                        search_text = text;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Search:");
                    if ui.text_edit_singleline(&mut search_text).changed() {
                        search_text = search_text.replace("\0", "");
                    }
                });
                ui.separator();

                let mut search_result = Vec::new();
                ui.ctx().memory_mut(|mem| {
                    let search_cache = mem.caches.cache::<PotionSearchCache<'_>>();
                    search_result = search_cache.get((&search_text, self.locale));
                });

                ui.ctx().data_mut(|data| {
                    data.insert_persisted(id, search_text);
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
                    .column(Column::exact(240.0))
                    .column(Column::remainder())
                    .min_scrolled_height(0.0);
                table.body(|body| {
                    body.rows(text_height, search_result.len(), |mut row| {
                        let item = game_data::POTIONS[search_result[row.index()]];
                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                *self.selected_consumable = Some(item);
                            }
                        });
                        row.col(|ui| {
                            ui.add(ItemNameLabel::new(item.item_id, item.hq, self.locale));
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
