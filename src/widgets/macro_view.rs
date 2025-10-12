use egui::{Align, Layout, Widget};
use raphael_data::{Locale, action_name, get_item_name};
use raphael_sim::Action;
use raphael_translations::{t, t_format};
use serde::{Deserialize, Serialize};

use crate::{context::AppContext, widgets::HelpText};

#[inline]
fn custom_format_help_text_string(locale: Locale) -> &'static str {
    t!(
        locale,
        "The format can be any arbitrary text or FFXIV command.

The following placeholders can be used to output their respective value:
  - {index} : the index or number of the macro block
  - {max_index} : the maximum index or number of macro blocks
  - {item_name} : the name of the selected recipe's item result
  - {food} : the name of the selected food
  - {potion} : the name of the selected potion
  - {craftsmanship} : the base craftsmanship stat
  - {control} : the base control stat
  - {cp} : the base CP stat"
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MacroViewConfig {
    #[serde(default)]
    split_macro: bool,
    #[serde(default)]
    include_delay: bool,
    #[serde(default)]
    extra_delay: u8,
    #[serde(default)]
    notification_enabled: bool,
    #[serde(default)]
    notification_config: MacroNotificationConfig,
    #[serde(default)]
    intro_enabled: bool,
    #[serde(default)]
    intro_config: MacroIntroConfig,
}

impl Default for MacroViewConfig {
    fn default() -> Self {
        Self {
            split_macro: true,
            include_delay: true,
            extra_delay: 0,
            notification_enabled: false,
            notification_config: MacroNotificationConfig::default(),
            intro_enabled: false,
            intro_config: MacroIntroConfig::default(),
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
    #[serde(default)]
    avoid_single_action_macro: bool,
}

impl Default for MacroNotificationConfig {
    fn default() -> Self {
        Self {
            default_notification: true,
            notification_sound: 1,
            custom_notification_format: String::new(),
            different_last_notification: false,
            custom_last_notification_format: String::new(),
            avoid_single_action_macro: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MacroIntroConfig {
    #[serde(default)]
    default_intro: bool,
    #[serde(default)]
    custom_intro_format: String,
}

impl Default for MacroIntroConfig {
    fn default() -> Self {
        Self {
            default_intro: true,
            custom_intro_format: String::new(),
        }
    }
}

struct MacroTextBox {
    text: String,
    id: egui::Id,
}

#[derive(Debug)]
struct FixedFormattingData {
    pub locale: Locale,
    pub notification_format: String,
    pub last_notification_format: String,
    pub intro_format: String,
}

fn preformat_fixed_data(format: &str, app_context: &AppContext) -> String {
    let locale = app_context.locale;
    let item_name = get_item_name(app_context.recipe_config.recipe.item_id, false, locale)
        .unwrap_or(t!(locale, "Unknown item").to_owned());
    let food_string = app_context
        .selected_food
        .map(|food| {
            get_item_name(food.item_id, food.hq, locale).unwrap_or("Unknown item".to_owned())
        })
        .unwrap_or(t!(locale, "None").to_owned());
    let potion_string = app_context
        .selected_potion
        .map(|potion| {
            get_item_name(potion.item_id, potion.hq, locale)
                .unwrap_or(t!(locale, "Unknown item").to_owned())
        })
        .unwrap_or(t!(locale, "None").to_owned());
    format
        .replace("{item_name}", &item_name)
        .replace("{food}", &food_string)
        .replace("{potion}", &potion_string)
        .replace(
            "{craftsmanship}",
            &app_context.active_stats().craftsmanship.to_string(),
        )
        .replace("{control}", &app_context.active_stats().control.to_string())
        .replace("{cp}", &app_context.active_stats().cp.to_string())
        .replace("{level}", &app_context.active_stats().level.to_string())
}

impl FixedFormattingData {
    pub fn new(app_context: &AppContext) -> Self {
        Self {
            locale: app_context.locale,
            notification_format: preformat_fixed_data(
                &app_context
                    .macro_view_config
                    .notification_config
                    .custom_notification_format,
                app_context,
            ),
            last_notification_format: preformat_fixed_data(
                &app_context
                    .macro_view_config
                    .notification_config
                    .custom_last_notification_format,
                app_context,
            ),
            intro_format: preformat_fixed_data(
                &app_context
                    .macro_view_config
                    .intro_config
                    .custom_intro_format,
                app_context,
            ),
        }
    }
}

fn format_custom_macro_command(command_format: &str, index: usize, max_index: usize) -> String {
    command_format
        .replace("{index}", &index.to_string())
        .replace("{max_index}", &max_index.to_string())
}

impl MacroTextBox {
    pub fn new(
        index: usize,
        max_index: usize,
        actions: &[Action],
        fixed_formatting_data: &FixedFormattingData,
        config: &MacroViewConfig,
        newline: &'static str,
    ) -> Self {
        let mut lines: Vec<String> = Vec::new();
        if config.intro_enabled {
            if config.intro_config.default_intro {
                lines.push("/macrolock".to_string());
            } else {
                lines.push(format_custom_macro_command(
                    &fixed_formatting_data.intro_format,
                    index,
                    max_index,
                ))
            }
        }
        lines.extend(actions.iter().map(|action| {
            if config.include_delay {
                format!(
                    "/ac \"{}\" <wait.{}>",
                    action_name(*action, fixed_formatting_data.locale),
                    action.time_cost() + config.extra_delay
                )
            } else {
                format!(
                    "/ac \"{}\"",
                    action_name(*action, fixed_formatting_data.locale)
                )
            }
        }));
        if config.notification_enabled && lines.len() < 15 {
            if config.notification_config.default_notification {
                lines.push(format!(
                    "/echo Macro finished ({}/{}) <se.{}>",
                    index, max_index, config.notification_config.notification_sound
                ));
            } else {
                let notification = if config.notification_config.different_last_notification
                    && index == max_index
                {
                    &fixed_formatting_data.last_notification_format
                } else {
                    &fixed_formatting_data.notification_format
                };

                lines.push(format_custom_macro_command(notification, index, max_index))
            }
        }
        Self {
            text: lines.join(newline),
            id: egui::Id::new(("MACRO_TEXT", index)),
        }
    }
}

impl Widget for MacroTextBox {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.add(
            super::MultilineMonospace::new(self.text)
                .scrollable([true, false])
                .id_salt(self.id),
        )
    }
}

pub struct MacroView<'a> {
    app_context: &'a mut AppContext,
    actions: &'a mut Vec<Action>,
}

impl<'a> MacroView<'a> {
    pub fn new(app_context: &'a mut AppContext, actions: &'a mut Vec<Action>) -> Self {
        Self {
            app_context,
            actions,
        }
    }
}

impl MacroView<'_> {
    fn macro_notification_menu(
        ui: &mut egui::Ui,
        notification_cfg: &mut MacroNotificationConfig,
        locale: Locale,
    ) {
        ui.style_mut().spacing.item_spacing.y = 3.0;
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut notification_cfg.default_notification,
                true,
                t!(locale, "Use default notification"),
            );
            ui.add_enabled(
                notification_cfg.default_notification,
                egui::DragValue::new(&mut notification_cfg.notification_sound)
                    .range(1..=16)
                    .prefix("<se.")
                    .suffix(">"),
            );
        });
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut notification_cfg.default_notification,
                false,
                t!(locale, "Use custom notification format"),
            );
            ui.add(HelpText::new(custom_format_help_text_string(locale)));
        });

        ui.horizontal(|ui| {
            ui.add_space(18.0);
            ui.vertical(|ui| {
                ui.add_enabled_ui(!notification_cfg.default_notification, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut notification_cfg.custom_notification_format,
                        )
                        .font(egui::TextStyle::Monospace)
                        .hint_text("/echo Done {index}/{max_index} <se.1>"),
                    );
                    ui.add_space(2.0);
                    ui.checkbox(
                        &mut notification_cfg.different_last_notification,
                        t!(locale, "Use different format for last notification"),
                    );
                    ui.add_enabled(
                        notification_cfg.different_last_notification,
                        egui::TextEdit::singleline(
                            &mut notification_cfg.custom_last_notification_format,
                        )
                        .font(egui::TextStyle::Monospace)
                        .hint_text("/echo All macros done <se.2>"),
                    );
                });
            });
        });
        ui.separator();
        ui.checkbox(
            &mut notification_cfg.avoid_single_action_macro,
            t!(locale, "Avoid single-action macros"),
        );
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            ui.vertical(|ui| {
                ui.label(
                    t!(locale, "Skip last notification if doing so avoids creating a macro with a single action."),
                )
            });
        });
    }

    fn macro_intro_menu(ui: &mut egui::Ui, intro_cfg: &mut MacroIntroConfig, locale: Locale) {
        ui.style_mut().spacing.item_spacing.y = 3.0;
        ui.radio_value(
            &mut intro_cfg.default_intro,
            true,
            t!(locale, "Use \"/macrolock\" as macro intro"),
        );
        ui.horizontal(|ui| {
            ui.radio_value(
                &mut intro_cfg.default_intro,
                false,
                t!(locale, "Use custom intro format"),
            );
            ui.add(HelpText::new(custom_format_help_text_string(locale)));
        });
        ui.horizontal(|ui| {
            ui.add_space(18.0);
            ui.vertical(|ui| {
                ui.add_enabled_ui(!intro_cfg.default_intro, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut intro_cfg.custom_intro_format)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("/macrolock <wait.5>"),
                    );
                });
            });
        });
    }
}

impl Widget for MacroView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let fixed_fromatting_data = FixedFormattingData::new(self.app_context);
        let AppContext {
            macro_view_config: config,
            locale,
            ..
        } = self.app_context;

        ui.ctx().data_mut(|d| {
            let actions_hash_id = egui::Id::new("ACTIONS_HASH");
            let current_actions_hash =
                egui::ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one(&self.actions);
            if let Some(stored_hash) = d.get_temp::<u64>(actions_hash_id)
                && stored_hash != current_actions_hash
            {
                for index in 0..=4 {
                    let macro_text_id = egui::Id::new(("MACRO_TEXT", index));
                    if let Some(ui_id) = d.get_temp::<egui::Id>(macro_text_id) {
                        // This has to match the exact sequence of hashes `egui::ScrollArea` does to work
                        let id_salt = egui::Id::new(macro_text_id);
                        let id = ui_id.with(id_salt);
                        d.remove::<egui::scroll_area::State>(id);
                    }
                }
            }
            d.insert_temp(actions_hash_id, current_actions_hash);
        });

        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t!(locale, "Macro")).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                !self.actions.is_empty(),
                                egui::Button::new(t!(locale, "Clear")),
                            )
                            .clicked()
                        {
                            self.actions.clear();
                        }
                        let duration = self
                            .actions
                            .iter()
                            .map(|action| action.time_cost())
                            .sum::<u8>();
                        ui.label(t_format!(
                            locale,
                            "{steps} steps, {duration} seconds",
                            steps = self.actions.len(),
                        ));
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.checkbox(&mut config.include_delay, t!(locale, "Include delay"));
                    ui.add_enabled_ui(config.include_delay, |ui| {
                        ui.label(t!(locale, "Extra delay"));
                        ui.add(egui::DragValue::new(&mut config.extra_delay).range(0..=9));
                    });
                });
                ui.horizontal(|ui| {
                    ui.checkbox(&mut config.split_macro, t!(locale, "Split macro"));
                    ui.checkbox(&mut config.intro_enabled, t!(locale, "Macro lock / intro"));
                    ui.add_enabled_ui(config.intro_enabled, |ui| {
                        egui::containers::menu::MenuButton::new(t!(locale, "✏ Edit"))
                            .config(
                                egui::containers::menu::MenuConfig::default()
                                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                            )
                            .ui(ui, |ui| {
                                ui.reset_style(); // prevent egui::DragValue from looking weird
                                ui.set_max_width(305.0);
                                Self::macro_intro_menu(ui, &mut config.intro_config, *locale)
                            });
                    });
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Checkbox::new(
                        &mut config.notification_enabled,
                        t!(locale, "End-of-macro notification"),
                    ));
                    ui.add_enabled_ui(config.notification_enabled, |ui| {
                        egui::containers::menu::MenuButton::new(t!(locale, "✏ Edit"))
                            .config(
                                egui::containers::menu::MenuConfig::default()
                                    .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside),
                            )
                            .ui(ui, |ui| {
                                ui.reset_style(); // prevent egui::DragValue from looking weird
                                ui.set_max_width(305.0);
                                Self::macro_notification_menu(
                                    ui,
                                    &mut config.notification_config,
                                    *locale,
                                )
                            });
                    });
                });
                ui.separator();

                let mut chunks = Vec::new();
                let mut remaining_actions = self.actions.as_slice();
                while !remaining_actions.is_empty() {
                    let max_chunk_size = if config.split_macro {
                        let chunk_size = 15 - usize::from(config.intro_enabled);
                        let avoid_notif = config.notification_config.avoid_single_action_macro
                            && remaining_actions.len() == chunk_size;
                        let has_notif = config.notification_enabled && !avoid_notif;
                        chunk_size - usize::from(has_notif)
                    } else {
                        usize::MAX
                    };
                    let (this_chunk, remaining) = remaining_actions
                        .split_at(std::cmp::min(max_chunk_size, remaining_actions.len()));
                    chunks.push(this_chunk);
                    remaining_actions = remaining;
                }

                let newline = match ui.ctx().os() {
                    egui::os::OperatingSystem::Mac => "\n",
                    _ => "\r\n",
                };
                let num_chunks = chunks.len();
                for (index, actions) in chunks.into_iter().enumerate() {
                    ui.add(MacroTextBox::new(
                        index + 1,
                        num_chunks,
                        actions,
                        &fixed_fromatting_data,
                        config,
                        newline,
                    ));
                }

                if self.actions.is_empty() {
                    ui.label(t!(locale, "None"));
                }

                // fill the remaining space
                ui.with_layout(Layout::bottom_up(Align::LEFT), |_| {});
            });
        })
        .response
    }
}
