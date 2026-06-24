use std::marker::PhantomData;

use egui::{
    Align, Id, Layout, Widget,
    util::cache::{ComputerMut, FrameCache},
};
use egui_extras::Column;
use raphael_data::{Consumable, CrafterStats, Locale, find_meals, find_potions};
use raphael_translations::t;

use crate::{
    context::AppContext,
    elements::{
        util::{self, TableColumnWidth},
        widgets::{GameDataNameLabel, collapse_persisted},
    },
};

trait ConsumableSelectType {
    const COLLAPSE_ID: &str;
    const TABLE_ID: &str;

    const CONSUMABLE_TYPE: ConsumableType;

    fn panel_name(locale: Locale) -> &'static str;
}

struct ConsumableSelect<'a, Type> {
    search_text: &'a mut String,
    crafter_stats: &'a CrafterStats,
    selected_consumable: &'a mut Option<Consumable>,
    locale: Locale,
    // Height of the search result table. `None` uses the default minimum height; `Some` lets
    // the caller stretch the table to fill available vertical space.
    table_height: Option<f32>,
    r#type: PhantomData<Type>,
}

/// Default and minimum height of a consumable (food/potion) search result table, in points
/// (~4 rows). Used as the floor when stretching the table to fill vertical space.
///
/// Like [`recipe_table_min_height`](super::recipe_table_min_height), this uses the panel's own
/// [`PANEL_ITEM_SPACING`](super::PANEL_ITEM_SPACING) rather than the caller's ambient spacing.
pub fn consumable_table_min_height(ui: &egui::Ui) -> f32 {
    let line_height = ui.spacing().interact_size.y;
    4.3 * line_height + 4.0 * super::PANEL_ITEM_SPACING.y
}

#[derive(Debug, Clone, Copy)]
enum ConsumableType {
    Food,
    Potion,
}

impl<'a, Type> ConsumableSelect<'a, Type>
where
    Type: ConsumableSelectType,
{
    pub fn new(app_context: &'a mut AppContext) -> Self {
        let AppContext {
            locale,
            search_state,
            selected_food,
            selected_potion,
            crafter_config,
            ..
        } = app_context;
        let (search_text, selected_consumable) = match Type::CONSUMABLE_TYPE {
            ConsumableType::Food => (&mut search_state.food.search_text, selected_food),
            ConsumableType::Potion => (&mut search_state.potion.search_text, selected_potion),
        };

        Self {
            search_text,
            crafter_stats: crafter_config.active_stats(),
            selected_consumable,
            locale: *locale,
            table_height: None,
            r#type: PhantomData,
        }
    }

    /// Sets an explicit height for the search result table. The caller is responsible for
    /// flooring this at [`consumable_table_min_height`] when a minimum is desired.
    pub fn table_height(mut self, table_height: Option<f32>) -> Self {
        self.table_height = table_height;
        self
    }
}

impl<Type> Widget for ConsumableSelect<'_, Type>
where
    Type: Default
        + ConsumableSelectType
        + for<'a> ComputerMut<(&'a str, raphael_data::Locale), std::vec::Vec<&'static Consumable>>,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let Self {
            search_text,
            crafter_stats,
            selected_consumable,
            locale,
            table_height,
            ..
        } = self;
        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = super::PANEL_ITEM_SPACING;
            ui.vertical(|ui| {
                let mut collapsed = false;

                ui.horizontal(|ui| {
                    collapse_persisted(ui, Id::new(Type::COLLAPSE_ID), &mut collapsed);
                    ui.label(egui::RichText::new(Type::panel_name(locale)).strong());
                    match selected_consumable {
                        None => ui.label(t!(locale, "None")),
                        Some(item) => ui.add(GameDataNameLabel::new(&*item, locale)),
                    };
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(selected_consumable.is_some(), egui::Button::new("🗑"))
                            .clicked()
                        {
                            *selected_consumable = None;
                        }
                    });
                });

                if collapsed {
                    return;
                }

                ui.separator();

                if egui::TextEdit::singleline(search_text)
                    .desired_width(f32::INFINITY)
                    .hint_text(t!(locale, "🔍 Search"))
                    .ui(ui)
                    .changed()
                {
                    *search_text = search_text.replace('\0', "");
                }
                ui.separator();

                let search_result = ui.ctx().memory_mut(|mem| {
                    let search_cache = mem
                        .caches
                        .cache::<FrameCache<Vec<&'static Consumable>, Type>>();
                    search_cache.get((search_text, locale)).clone()
                });

                let line_height = ui.spacing().interact_size.y;
                let table_height = table_height.unwrap_or_else(|| consumable_table_min_height(ui));

                // Column::remainder().clip(true) is buggy when resizing the table
                let column_widths = util::calculate_column_widths(
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
                    .id_salt(Type::TABLE_ID)
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
                                *selected_consumable = Some(*item);
                            }
                        });
                        row.col(|ui| {
                            ui.add(GameDataNameLabel::new(item, locale));
                        });
                        row.col(|ui| {
                            ui.label(util::effect_string(*item, crafter_stats, locale));
                        });
                    });
                });
            });
        })
        .response
    }
}

#[derive(Default)]
struct FoodType {}

impl ConsumableSelectType for FoodType {
    const COLLAPSE_ID: &str = "FOOD_SEARCH_COLLAPSED";
    const TABLE_ID: &str = "FOOD_SELECT_TABLE";

    const CONSUMABLE_TYPE: ConsumableType = ConsumableType::Food;

    #[inline]
    fn panel_name(locale: Locale) -> &'static str {
        t!(locale, "Food")
    }
}

impl ComputerMut<(&str, Locale), Vec<&'static Consumable>> for FoodType {
    fn compute(&mut self, (text, locale): (&str, Locale)) -> Vec<&'static Consumable> {
        find_meals(text, locale).collect::<Vec<_>>()
    }
}

pub struct FoodSelect<'a> {
    inner: ConsumableSelect<'a, FoodType>,
}

impl<'a> FoodSelect<'a> {
    #[inline]
    pub fn new(app_context: &'a mut AppContext) -> Self {
        Self {
            inner: ConsumableSelect::new(app_context),
        }
    }

    #[inline]
    pub fn table_height(mut self, table_height: Option<f32>) -> Self {
        self.inner = self.inner.table_height(table_height);
        self
    }
}

impl Widget for FoodSelect<'_> {
    #[inline]
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.inner.ui(ui)
    }
}

#[derive(Default)]
struct PotionType {}

impl ConsumableSelectType for PotionType {
    const COLLAPSE_ID: &str = "POTION_SEARCH_COLLAPSED";
    const TABLE_ID: &str = "POTION_SELECT_TABLE";

    const CONSUMABLE_TYPE: ConsumableType = ConsumableType::Potion;

    #[inline]
    fn panel_name(locale: Locale) -> &'static str {
        t!(locale, "Potion")
    }
}

impl ComputerMut<(&str, Locale), Vec<&'static Consumable>> for PotionType {
    fn compute(&mut self, (text, locale): (&str, Locale)) -> Vec<&'static Consumable> {
        find_potions(text, locale).collect::<Vec<_>>()
    }
}

pub struct PotionSelect<'a> {
    inner: ConsumableSelect<'a, PotionType>,
}

impl<'a> PotionSelect<'a> {
    #[inline]
    pub fn new(app_context: &'a mut AppContext) -> Self {
        Self {
            inner: ConsumableSelect::new(app_context),
        }
    }

    #[inline]
    pub fn table_height(mut self, table_height: Option<f32>) -> Self {
        self.inner = self.inner.table_height(table_height);
        self
    }
}

impl Widget for PotionSelect<'_> {
    #[inline]
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        self.inner.ui(ui)
    }
}
