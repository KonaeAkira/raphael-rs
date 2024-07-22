use egui::{Align, Id, Layout, Widget};
use game_data::{action_name, Locale};
use serde::{Deserialize, Serialize};
use simulator::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct MacroViewConfig {
    #[serde(default)]
    split_macro: bool,
    #[serde(default)]
    include_delay: bool,
    #[serde(default)]
    notification_enabled: bool,
    #[serde(default)]
    notification_sound: u8,
    #[serde(default)]
    macro_lock: bool,
}

impl Default for MacroViewConfig {
    fn default() -> Self {
        Self {
            split_macro: true,
            include_delay: true,
            notification_enabled: false,
            notification_sound: 1,
            macro_lock: false,
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
        newline: &'static str,
        locale: Locale,
    ) -> Self {
        let mut lines: Vec<String> = Vec::new();
        if config.macro_lock {
            lines.push("/macrolock ".to_string());
        }
        lines.extend(actions.iter().map(|action| {
            if config.include_delay {
                format!(
                    "/ac \"{}\" <wait.{}>",
                    action_name(*action, locale),
                    action.time_cost()
                )
            } else {
                format!("/ac \"{}\"", action_name(*action, locale))
            }
        }));
        if config.notification_enabled {
            lines.push(format!(
                "/echo Macro finished ({}/{}) <se.{}>",
                index, max_index, config.notification_sound
            ));
        }
        Self {
            text: lines.join(newline),
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
    locale: Locale,
}

impl<'a> MacroView<'a> {
    pub fn new(
        actions: &'a mut Vec<Action>,
        config: &'a mut MacroViewConfig,
        locale: Locale,
    ) -> Self {
        Self {
            actions,
            config,
            locale,
        }
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
                    ui.checkbox(&mut self.config.macro_lock, "Macro lock");
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
                    true => {
                        let mut chunk_size = 15;
                        if self.config.notification_enabled {
                            chunk_size -= 1;
                        }
                        if self.config.macro_lock {
                            chunk_size -= 1;
                        }
                        chunk_size
                    }
                    false => usize::MAX,
                };
                let count = self.actions.chunks(chunk_size).count();
                let newline = match ui.ctx().os() {
                    egui::os::OperatingSystem::Mac => "\n",
                    _ => "\r\n",
                };
                for (index, actions) in self.actions.chunks(chunk_size).enumerate() {
                    ui.add(MacroTextBox::new(
                        index + 1,
                        count,
                        actions,
                        self.config,
                        newline,
                        self.locale,
                    ));
                }
                // fill the remaining space
                ui.with_layout(Layout::bottom_up(Align::LEFT), |_| {});
            });
        })
        .response
    }
}
