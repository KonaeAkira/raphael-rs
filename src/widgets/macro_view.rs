use egui::{Align, Id, Layout, Widget};
use raphael_data::{Locale, action_name};
use raphael_sim::Action;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MacroViewConfig {
    #[serde(default)]
    split_macro: bool,
    #[serde(default)]
    include_delay: bool,
    #[serde(default)]
    notification_enabled: bool,
    #[serde(default)]
    notification_config: MacroNotificationConfig,
    #[serde(default)]
    macro_lock: bool,
}

impl Default for MacroViewConfig {
    fn default() -> Self {
        Self {
            split_macro: true,
            include_delay: true,
            notification_enabled: false,
            notification_config: MacroNotificationConfig::default(),
            macro_lock: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MacroNotificationConfig {
    #[serde(default)]
    default_notification: bool,
    #[serde(default)]
    notification_sound: u8,
    #[serde(default)]
    custom_notification_format: String,
    #[serde(default)]
    different_last_notification: bool,
    #[serde(default)]
    custom_last_notification_format: String,
}

impl Default for MacroNotificationConfig {
    fn default() -> Self {
        Self {
            default_notification: true,
            notification_sound: 1,
            custom_notification_format: "/echo Example ({index}/{max_index}) <se.1>".to_owned(),
            different_last_notification: false,
            custom_last_notification_format: "/echo Example End".to_owned(),
        }
    }
}

struct MacroTextBox {
    text: String,
}

fn format_custom_notification(notification_format: &str, index: usize, max_index: usize) -> String {
    notification_format
        .replace("{index}", &index.to_string())
        .replace("{max_index}", &max_index.to_string())
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
            if config.notification_config.default_notification {
                lines.push(format!(
                    "/echo Macro finished ({}/{}) <se.{}>",
                    index, max_index, config.notification_config.notification_sound
                ));
            } else {
                let notification = if config.notification_config.different_last_notification
                    && index == max_index
                {
                    &config.notification_config.custom_last_notification_format
                } else {
                    &config.notification_config.custom_notification_format
                };

                lines.push(format_custom_notification(notification, index, max_index))
            }
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
                            ui.ctx().copy_text(self.text);
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

impl MacroView<'_> {
    fn macro_notification_menu(ui: &mut egui::Ui, notification_cfg: &mut MacroNotificationConfig) {
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut notification_cfg.default_notification,
                true,
                "Use default notification",
            );
            ui.add_enabled_ui(notification_cfg.default_notification, |ui| {
                ui.reset_style();
                ui.add_sized(
                    [50.0, ui.available_height()],
                    egui::DragValue::new(&mut notification_cfg.notification_sound)
                        .range(1..=16)
                        .prefix("<se.")
                        .suffix(">"),
                );
            });
        });

        ui.separator();
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut notification_cfg.default_notification,
                false,
                "Use custom notification format",
            );
            ui.add(super::HelpText::new("Specify the exact format of the command that is executed at the end of each macro.\n\nUse the special format strings \"{index}\" and \"{max_index}\" to add the respective value to the notification."));
        });

        ui.add_space(5.0);
        ui.add_enabled_ui(!notification_cfg.default_notification, |ui| {
            ui.vertical(|ui| {
                ui.text_edit_singleline(&mut notification_cfg.custom_notification_format);
                ui.add_space(5.0);
                ui.checkbox(
                    &mut notification_cfg.different_last_notification,
                    "Use different format for last notification",
                );
                ui.add_enabled_ui(notification_cfg.different_last_notification, |ui| {
                    egui::TextEdit::singleline(
                        &mut notification_cfg.custom_last_notification_format,
                    )
                    .ui(ui);
                });
            });
        });
    }
}

impl Widget for MacroView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
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
                        let duration = self
                            .actions
                            .iter()
                            .map(|action| action.time_cost())
                            .sum::<u8>();
                        ui.label(format!(
                            "{} steps, {} seconds",
                            self.actions.len(),
                            duration
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
                        egui::containers::menu::MenuButton::new("✏ Edit contents")
                            .config(
                                egui::containers::menu::MenuConfig::default()
                                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                            )
                            .ui(ui, |ui| {
                                Self::macro_notification_menu(
                                    ui,
                                    &mut self.config.notification_config,
                                )
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

                if self.actions.is_empty() {
                    ui.label("None");
                }

                // fill the remaining space
                ui.with_layout(Layout::bottom_up(Align::LEFT), |_| {});
            });
        })
        .response
    }
}
