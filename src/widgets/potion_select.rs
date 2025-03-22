use egui::{
    Align, Id, Layout, Widget,
    util::cache::{ComputerMut, FrameCache},
};
use egui_extras::Column;
use raphael_data::{Consumable, CrafterStats, Locale, find_potions};

use super::{ItemNameLabel, util};

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

impl Widget for PotionSelect<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.vertical(|ui| {
                let mut collapsed = false;
                ui.horizontal(|ui| {
                    util::collapse_persisted(
                        ui,
                        Id::new("POTION_SEARCH_COLLAPSED"),
                        &mut collapsed,
                    );
                    ui.label(egui::RichText::new("Potion").strong());
                    match self.selected_consumable {
                        None => ui.label("None"),
                        Some(item) => {
                            ui.add(ItemNameLabel::new(item.item_id, item.hq, self.locale))
                        }
                    };
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

                if collapsed {
                    return;
                }

                ui.separator();

                let id = Id::new("POTION_SEARCH_TEXT");

                let mut search_text = String::new();
                ui.ctx().data_mut(|data| {
                    if let Some(text) = data.get_persisted::<String>(id) {
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
                    let search_cache = mem.caches.cache::<PotionSearchCache<'_>>();
                    search_result = search_cache.get((&search_text, self.locale));
                });

                ui.ctx().data_mut(|data| {
                    data.insert_persisted(id, search_text);
                });

                let line_height = ui.spacing().interact_size.y;
                let line_spacing = ui.spacing().item_spacing.y;
                let table_height = 4.3 * line_height + 4.0 * line_spacing;

                let column_spacing = 2.0 * ui.spacing().item_spacing.x;
                let available_text_width = ui.available_width() - column_spacing - 42.0;
                let item_name_width = (0.7 * available_text_width).clamp(220.0, 320.0);
                let effect_width = (available_text_width - item_name_width).max(0.0);

                let table = egui_extras::TableBuilder::new(ui)
                    .id_salt("POTION_SELECT_TABLE")
                    .auto_shrink(false)
                    .striped(true)
                    .column(Column::exact(42.0))
                    .column(Column::exact(item_name_width))
                    .column(Column::exact(effect_width))
                    .min_scrolled_height(table_height)
                    .max_scroll_height(table_height);

                table.body(|body| {
                    body.rows(line_height, search_result.len(), |mut row| {
                        let item = raphael_data::POTIONS[search_result[row.index()]];
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
