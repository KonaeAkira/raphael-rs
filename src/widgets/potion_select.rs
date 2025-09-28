use egui::{
    Align, Id, Layout, Widget,
    util::cache::{ComputerMut, FrameCache},
};
use egui_extras::Column;
use raphael_data::{Consumable, CrafterStats, Locale, find_potions};
use raphael_translations::t;

use crate::{
    context::AppContext,
    widgets::GameDataNameLabel,
    widgets::util::{TableColumnWidth, calculate_column_widths},
};

use super::util;

#[derive(Default)]
struct PotionFinder {}

impl ComputerMut<(&str, Locale), Vec<&'static Consumable>> for PotionFinder {
    fn compute(&mut self, (text, locale): (&str, Locale)) -> Vec<&'static Consumable> {
        find_potions(text, locale).collect::<Vec<_>>()
    }
}

type PotionSearchCache<'a> = FrameCache<Vec<&'static Consumable>, PotionFinder>;

pub struct PotionSelect<'a> {
    crafter_stats: &'a CrafterStats,
    selected_consumable: &'a mut Option<Consumable>,
    locale: Locale,
}

impl<'a> PotionSelect<'a> {
    pub fn new(app_context: &'a mut AppContext) -> Self {
        let AppContext {
            locale,
            selected_potion: selected_consumable,
            crafter_config,
            ..
        } = app_context;
        Self {
            crafter_stats: crafter_config.active_stats(),
            selected_consumable,
            locale: *locale,
        }
    }
}

impl Widget for PotionSelect<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let locale = self.locale;
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
                    ui.label(egui::RichText::new(t!(locale, "Potion")).strong());
                    match self.selected_consumable {
                        None => ui.label(t!(locale, "None")),
                        Some(item) => ui.add(GameDataNameLabel::new(&*item, locale)),
                    };
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(self.selected_consumable.is_some(), egui::Button::new("🗑"))
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
                    .hint_text(t!(locale, "🔍 Search"))
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

                // Column::remainder().clip(true) is buggy when resizing the table
                let column_widths = calculate_column_widths(
                    ui,
                    [
                        TableColumnWidth::SelectButton,
                        TableColumnWidth::RelativeToRemainingClamped {
                            scale: 0.7,
                            min: 220.0,
                            max: 320.0,
                        },
                        TableColumnWidth::Remaining,
                    ],
                    locale,
                );

                let table = egui_extras::TableBuilder::new(ui)
                    .id_salt("POTION_SELECT_TABLE")
                    .auto_shrink(false)
                    .striped(true)
                    .column(Column::exact(column_widths[0]))
                    .column(Column::exact(column_widths[1]))
                    .column(Column::exact(column_widths[2]))
                    .min_scrolled_height(table_height)
                    .max_scroll_height(table_height);

                table.body(|body| {
                    body.rows(line_height, search_result.len(), |mut row| {
                        let item = search_result[row.index()];
                        row.col(|ui| {
                            if ui.button(t!(locale, "Select")).clicked() {
                                *self.selected_consumable = Some(*item);
                            }
                        });
                        row.col(|ui| {
                            ui.add(GameDataNameLabel::new(item, locale));
                        });
                        row.col(|ui| {
                            ui.label(util::effect_string(*item, self.crafter_stats, locale));
                        });
                    });
                });
            });
        })
        .response
    }
}
