use egui::{Align, Layout, Widget};
use egui_extras::Column;
use game_data::{Consumable, CrafterStats};

pub struct ConsumableSelect<'a> {
    title: &'static str,
    crafter_stats: CrafterStats,
    consumables: &'a [Consumable],
    selected_consumable: &'a mut Option<Consumable>,
}

impl<'a> ConsumableSelect<'a> {
    pub fn new(
        title: &'static str,
        crafter_stats: CrafterStats,
        consumables: &'a [Consumable],
        selected_consumable: &'a mut Option<Consumable>,
    ) -> Self {
        Self {
            title,
            crafter_stats,
            consumables,
            selected_consumable,
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
                        Some(item) => item.name,
                        None => "None",
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
                table
                    .header(text_height, |mut header| {
                        header.col(|_| {});
                        header.col(|ui| {
                            ui.label("Item Name");
                        });
                        header.col(|ui| {
                            ui.label("Effect");
                        });
                    })
                    .body(|body| {
                        body.rows(text_height, self.consumables.len(), |mut row| {
                            let item = self.consumables[row.index()];
                            row.col(|ui| {
                                if ui.button("Select").clicked() {
                                    *self.selected_consumable = Some(item);
                                }
                            });
                            row.col(|ui| {
                                ui.label(item.name);
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
