use egui::{Align, Id, Layout, Widget};
use serde::{Deserialize, Serialize};
use simulator::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct MacroViewConfig {
    split_macro: bool,
    include_delay: bool,
    // end-of-macro notification
    notification_enabled: bool,
    notification_sound: u8,
}

impl Default for MacroViewConfig {
    fn default() -> Self {
        Self {
            split_macro: false,
            include_delay: true,
            notification_enabled: false,
            notification_sound: 1,
        }
    }
}

struct MacroTextBox {
    text: String,
}

impl MacroTextBox {
    pub fn new(
        index: usize,
        max_index: usize,
        actions: &[Action],
        config: &MacroViewConfig,
    ) -> Self {
        let mut lines: Vec<_> = actions
            .into_iter()
            .map(|action| {
                if config.include_delay {
                    format!(
                        "/ac \"{}\" <wait.{}>",
                        action.display_name(),
                        action.time_cost()
                    )
                } else {
                    format!("/ac \"{}\"", action.display_name())
                }
            })
            .collect();
        if config.notification_enabled {
            lines.push(format!(
                "/echo Macro finished ({}/{}) <se.{}>",
                index, max_index, config.notification_sound
            ));
        }
        Self {
            text: lines.join("\r\n"),
        }
    }
}

impl Widget for MacroTextBox {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let id = Id::new(&self.text);
        ui.group(|ui| {
            ui.horizontal_top(|ui| {
                ui.monospace(&self.text);
                ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if ui.ctx().animate_bool_with_time(id, false, 2.0) == 0.0 {
                        if ui.button("Copy").clicked() {
                            ui.output_mut(|output| output.copied_text = self.text);
                            ui.ctx().animate_bool_with_time(id, true, 0.0);
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Copied"));
                    }
                });
            });
        })
        .response
    }
}

pub struct MacroView<'a> {
    actions: &'a mut Vec<Action>,
    config: &'a mut MacroViewConfig,
}

impl<'a> MacroView<'a> {
    pub fn new(actions: &'a mut Vec<Action>, config: &'a mut MacroViewConfig) -> Self {
        Self { actions, config }
    }
}

impl<'a> Widget for MacroView<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Macro").strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(!self.actions.is_empty(), egui::Button::new("Clear"))
                            .clicked()
                        {
                            self.actions.clear();
                        }
                        ui.label(format!(
                            "{} steps | {} seconds",
                            self.actions.len(),
                            self.actions
                                .iter()
                                .map(|action| action.time_cost())
                                .sum::<i16>()
                        ));
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.config.include_delay, "Include delay");
                    ui.checkbox(&mut self.config.split_macro, "Split macro");
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Checkbox::new(
                        &mut self.config.notification_enabled,
                        "End-of-macro notification",
                    ));
                    ui.add_enabled_ui(self.config.notification_enabled, |ui| {
                        egui::ComboBox::from_id_source("SOUND_EFFECT")
                            .selected_text(format!("<se.{}>", self.config.notification_sound))
                            .show_ui(ui, |ui| {
                                for i in 1..=16 {
                                    ui.selectable_value(
                                        &mut self.config.notification_sound,
                                        i,
                                        format!("<se.{}>", i),
                                    );
                                }
                            });
                    });
                });
                ui.separator();
                let chunk_size = match self.config.split_macro {
                    true if self.config.notification_enabled => 14,
                    true => 15,
                    false => usize::MAX,
                };
                let count = self.actions.chunks(chunk_size).count();
                for (index, actions) in self.actions.chunks(chunk_size).enumerate() {
                    ui.add(MacroTextBox::new(index + 1, count, actions, &self.config));
                }
                // fill the remaining space
                ui.with_layout(Layout::bottom_up(Align::LEFT), |_| {});
            });
        })
        .response
    }
}
