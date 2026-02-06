use std::collections::VecDeque;

use egui::{Align, Layout, Widget};
use raphael_data::{Locale, get_item_name, macro_name};
use raphael_sim::Action;
use raphael_translations::{t, t_format};
use serde::{Deserialize, Serialize};

use crate::{
    context::AppContext,
    widgets::{HelpText, MultilineMonospace},
};

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

impl Widget for MacroView<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let locale = self.app_context.locale;
        if config_menu_is_visible(ui.ctx()) {
            draw_config_menu(ui.ctx(), &mut self.app_context.macro_view_config, locale);
        }
        ui.group(|ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 3.0);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(t!(locale, "Macro")).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("⚙").clicked() {
                            set_config_menu_visibility(ui.ctx(), true);
                        }
                        if ui
                            .add_enabled(!self.actions.is_empty(), egui::Button::new("🗑"))
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
                if self.actions.is_empty() {
                    ui.label(t!(locale, "None"));
                } else {
                    let newline = match ui.ctx().os() {
                        egui::os::OperatingSystem::Mac => "\n",
                        _ => "\r\n",
                    };
                    let context = MacroContextData::from_app_context(self.app_context);
                    let config = &self.app_context.macro_view_config;
                    let macros = create_macros(&context, config, self.actions, newline);
                    for (macro_idx, macro_text) in macros.into_iter().enumerate() {
                        let id = egui::Id::new("MACRO_TEXT_BOX").with(macro_idx);
                        ui.add(MultilineMonospace::new(id, macro_text).scrollable([true, false]));
                    }
                }
                // Fill the remaining space.
                ui.with_layout(Layout::bottom_up(Align::LEFT), |_| {});
            });
        })
        .response
    }
}

fn config_menu_is_visible(ctx: &egui::Context) -> bool {
    let id = egui::Id::new("MACRO_CONFIG_MODAL_VISIBLE");
    ctx.data(|data| data.get_temp(id) == Some(true))
}

fn set_config_menu_visibility(ctx: &egui::Context, visible: bool) {
    let id = egui::Id::new("MACRO_CONFIG_MODAL_VISIBLE");
    ctx.data_mut(|data| data.insert_temp(id, visible));
}

fn draw_config_menu(ctx: &egui::Context, config: &mut MacroViewConfig, locale: Locale) {
    egui::containers::Modal::new(egui::Id::new("MACRO_CONFIG_MODAL")).show(ctx, |ui| {
        ui.set_width(
            (ctx.content_rect().width() - ui.style().spacing.item_spacing.x * 4.0)
                .clamp(0.0, 360.0),
        );
        ui.style_mut().spacing.item_spacing.y = 3.0;
        ui.label(egui::RichText::new(t!(locale, "Macro")).strong());
        ui.separator();
        ui.horizontal(|ui| {
            ui.checkbox(&mut config.include_delay, t!(locale, "Include delay"));
            ui.add_enabled_ui(config.include_delay, |ui| {
                ui.label(t!(locale, "Extra delay"));
                ui.add(egui::DragValue::new(&mut config.extra_delay).range(0..=9));
            });
        });
        ui.checkbox(&mut config.split_macro, t!(locale, "Split macro"));
        ui.checkbox(&mut config.intro_enabled, t!(locale, "Macro lock / intro"));
        ui.indent("macro_lock_indent", |ui| {
            if !config.intro_enabled {
                ui.disable();
            }
            draw_macro_intro_subconfig(ui, &mut config.intro_config, locale);
        });
        ui.checkbox(
            &mut config.notification_enabled,
            t!(locale, "End-of-macro notification"),
        );
        ui.indent("notification_indent", |ui| {
            if !config.notification_enabled {
                ui.disable();
            }
            draw_macro_notification_subconfig(ui, &mut config.notification_config, locale);
        });
        ui.separator();
        ui.vertical_centered_justified(|ui| {
            if ui.button(t!(locale, "Close")).clicked() {
                set_config_menu_visibility(ctx, false);
            }
        });
    });
}

fn draw_macro_notification_subconfig(
    ui: &mut egui::Ui,
    notification_cfg: &mut MacroNotificationConfig,
    locale: Locale,
) {
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
    ui.indent("custom_notification_indent", |ui| {
        if notification_cfg.default_notification {
            ui.disable();
        }
        ui.add(
            egui::TextEdit::singleline(&mut notification_cfg.custom_notification_format)
                .font(egui::TextStyle::Monospace)
                .hint_text("/echo Done {index}/{max_index} <se.1>"),
        );
        ui.checkbox(
            &mut notification_cfg.different_last_notification,
            t!(locale, "Use different format for last notification"),
        );
        ui.add_enabled(
            notification_cfg.different_last_notification,
            egui::TextEdit::singleline(&mut notification_cfg.custom_last_notification_format)
                .font(egui::TextStyle::Monospace)
                .hint_text("/echo All macros done <se.2>"),
        );
    });
    ui.checkbox(
        &mut notification_cfg.avoid_single_action_macro,
        t!(locale, "Avoid single-action macros"),
    );
    ui.indent("avoid_single_action_macro_indent", |ui| {
        if !notification_cfg.avoid_single_action_macro {
            ui.disable();
        }
        ui.label(t!(
            locale,
            "Skip last notification if doing so avoids creating a macro with a single action."
        ))
    });
}

fn draw_macro_intro_subconfig(ui: &mut egui::Ui, intro_cfg: &mut MacroIntroConfig, locale: Locale) {
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
    ui.indent("custom_intro_indent", |ui| {
        if intro_cfg.default_intro {
            ui.disable();
        }
        ui.add(
            egui::TextEdit::singleline(&mut intro_cfg.custom_intro_format)
                .font(egui::TextStyle::Monospace)
                .hint_text("/macrolock <wait.5>"),
        );
    });
}

/// Extra data from the app context that is needed to create macros.
struct MacroContextData {
    locale: Locale,
    item_name: String,
    food_name: String,
    potion_name: String,
    crafter_stats: raphael_data::CrafterStats,
}

impl MacroContextData {
    fn from_app_context(app_context: &AppContext) -> Self {
        let locale = app_context.locale;
        let item_name = get_item_name(app_context.recipe_config.recipe.item_id, false, locale)
            .unwrap_or(t!(locale, "Unknown item").to_owned());
        let food_name = match app_context.selected_food {
            Some(item) => {
                get_item_name(item.item_id, item.hq, locale).unwrap_or("Unknown item".to_owned())
            }
            None => t!(locale, "None").to_owned(),
        };
        let potion_name = match app_context.selected_potion {
            Some(item) => {
                get_item_name(item.item_id, item.hq, locale).unwrap_or("Unknown item".to_owned())
            }
            None => t!(locale, "None").to_owned(),
        };
        Self {
            locale,
            item_name,
            food_name,
            potion_name,
            crafter_stats: *app_context.active_stats(),
        }
    }
}

fn create_macros(
    context: &MacroContextData,
    config: &MacroViewConfig,
    actions: &[Action],
    newline: &str,
) -> Vec<String> {
    let mut lines = VecDeque::new();

    if config.intro_enabled {
        if config.intro_config.default_intro {
            lines.push_back("/macrolock".to_string());
        } else {
            lines.push_back(config.intro_config.custom_intro_format.clone());
        }
    }

    for action in actions {
        if config.include_delay {
            lines.push_back(format!(
                "/ac \"{}\" <wait.{}>",
                macro_name(*action, context.locale),
                action.time_cost() + config.extra_delay
            ));
        } else {
            lines.push_back(format!("/ac \"{}\"", macro_name(*action, context.locale)));
        }
    }

    let max_macro_len = if config.split_macro { 15 } else { usize::MAX };

    let mut macros = Vec::new();
    let mut current_macro = Vec::new();
    while let Some(line) = lines.pop_front() {
        current_macro.push(line);
        if config.notification_enabled
            && current_macro.len() < max_macro_len // Has place for notification.
            && (current_macro.len() + 1 == max_macro_len || lines.is_empty()) // Is end of macro.
            && !(config.notification_config.avoid_single_action_macro && lines.len() == 1)
        {
            let notification_command = if config.notification_config.default_notification {
                format!(
                    "/echo Macro finished ({{index}}/{{max_index}}) <se.{}>",
                    config.notification_config.notification_sound
                )
            } else if config.notification_config.different_last_notification && lines.is_empty() {
                config
                    .notification_config
                    .custom_last_notification_format
                    .clone()
            } else {
                config
                    .notification_config
                    .custom_notification_format
                    .clone()
            };
            current_macro.push(notification_command);
        }
        if current_macro.len() >= max_macro_len || lines.is_empty() {
            macros.push(current_macro.join(newline));
            current_macro.clear();
        }
    }

    let max_index = macros.len().to_string();
    for (macro_idx, macro_text) in macros.iter_mut().enumerate() {
        *macro_text = macro_text
            .replace("{index}", &(macro_idx + 1).to_string())
            .replace("{max_index}", &max_index)
            .replace("{item_name}", &context.item_name)
            .replace("{food}", &context.food_name)
            .replace("{potion}", &context.potion_name)
            .replace(
                "{craftsmanship}",
                &context.crafter_stats.craftsmanship.to_string(),
            )
            .replace("{control}", &context.crafter_stats.control.to_string())
            .replace("{cp}", &context.crafter_stats.cp.to_string())
            .replace("{level}", &context.crafter_stats.level.to_string());
    }

    macros
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    fn default_context() -> MacroContextData {
        MacroContextData {
            locale: Locale::EN,
            item_name: "Test Item".to_string(),
            food_name: "Test Food".to_string(),
            potion_name: "Test Potion".to_string(),
            crafter_stats: raphael_data::CrafterStats::default(),
        }
    }

    #[test]
    fn empty_macro() {
        let context = default_context();
        let config = MacroViewConfig::default();
        let macros = create_macros(&context, &config, &[], "\n");
        assert!(macros.is_empty());
    }

    #[test]
    /// This case actually doesn't happen in practice because `create_macros` is not called
    /// when there are no actions. This test only exists to track current behavior.
    fn empty_macro_with_lock_and_notification() {
        let context = default_context();
        let mut config = MacroViewConfig::default();
        config.intro_enabled = true;
        config.notification_enabled = true;
        let macros = create_macros(&context, &config, &[], "\n");
        assert_eq!(macros.len(), 1);
        expect_test::expect![[r#"
            /macrolock
            /echo Macro finished (1/1) <se.1>"#]]
        .assert_eq(&macros[0]);
    }

    #[test]
    /// This test serves two purposes:
    /// - Test the behavior of the default macro config.
    /// - Test the stringification for all actions.
    fn default_settings_all_actions() {
        let context = default_context();
        let config = MacroViewConfig::default();
        let actions = Action::iter().collect::<Vec<_>>();
        let macros = create_macros(&context, &config, &actions, "\n");
        assert_eq!(macros.len(), 3);
        expect_test::expect![[r#"
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Touch" <wait.3>
            /ac "Master's Mend" <wait.3>
            /ac "Observe" <wait.3>
            /ac "Tricks of the Trade" <wait.3>
            /ac "Waste Not" <wait.2>
            /ac "Veneration" <wait.2>
            /ac "Standard Touch" <wait.3>
            /ac "Great Strides" <wait.2>
            /ac "Innovation" <wait.2>
            /ac "Waste Not II" <wait.2>
            /ac "Byregot's Blessing" <wait.3>
            /ac "Precise Touch" <wait.3>
            /ac "Muscle Memory" <wait.3>
            /ac "Careful Synthesis" <wait.3>"#]]
        .assert_eq(&macros[0]);
        expect_test::expect![[r#"
            /ac "Manipulation" <wait.2>
            /ac "Prudent Touch" <wait.3>
            /ac "Advanced Touch" <wait.3>
            /ac "Reflect" <wait.3>
            /ac "Preparatory Touch" <wait.3>
            /ac "Groundwork" <wait.3>
            /ac "Delicate Synthesis" <wait.3>
            /ac "Intensive Synthesis" <wait.3>
            /ac "Trained Eye" <wait.3>
            /ac "Heart and Soul" <wait.3>
            /ac "Prudent Synthesis" <wait.3>
            /ac "Trained Finesse" <wait.3>
            /ac "Refined Touch" <wait.3>
            /ac "Quick Innovation" <wait.3>
            /ac "Immaculate Mend" <wait.3>"#]]
        .assert_eq(&macros[1]);
        expect_test::expect![[r#"
            /ac "Trained Perfection" <wait.3>
            /ac "Duty Action II" <wait.2>
            /ac "Rapid Synthesis" <wait.3>
            /ac "Hasty Touch" <wait.3>
            /ac "Hasty Touch" <wait.3>"#]]
        .assert_eq(&macros[2]);
    }

    #[test]
    /// Test that the last notification is skipped when it would create a single-action macro.
    fn skip_last_notification() {
        let context = default_context();
        let mut config = MacroViewConfig::default();
        config.split_macro = true;
        config.notification_enabled = true;
        config.notification_config.avoid_single_action_macro = true;
        let actions = [
            Action::MuscleMemory,
            Action::WasteNot,
            Action::Veneration,
            Action::Groundwork,
            Action::Groundwork,
            Action::Groundwork,
            Action::PrudentSynthesis,
            Action::MasterMend,
            Action::PrudentTouch,
            Action::Innovation,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::MasterMend,
            Action::Innovation,
            Action::PrudentTouch,
            Action::BasicTouch,
            Action::StandardTouch,
            Action::AdvancedTouch,
            Action::GreatStrides,
            Action::Innovation,
            Action::Observe,
            Action::AdvancedTouch,
            Action::GreatStrides,
            Action::ByregotsBlessing,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
        ];
        let macros = create_macros(&context, &config, &actions, "\n");
        assert_eq!(macros.len(), 2);
        expect_test::expect![[r#"
            /ac "Muscle Memory" <wait.3>
            /ac "Waste Not" <wait.2>
            /ac "Veneration" <wait.2>
            /ac "Groundwork" <wait.3>
            /ac "Groundwork" <wait.3>
            /ac "Groundwork" <wait.3>
            /ac "Prudent Synthesis" <wait.3>
            /ac "Master's Mend" <wait.3>
            /ac "Prudent Touch" <wait.3>
            /ac "Innovation" <wait.2>
            /ac "Prudent Touch" <wait.3>
            /ac "Prudent Touch" <wait.3>
            /ac "Prudent Touch" <wait.3>
            /ac "Prudent Touch" <wait.3>
            /echo Macro finished (1/2) <se.1>"#]]
        .assert_eq(&macros[0]);
        expect_test::expect![[r#"
            /ac "Master's Mend" <wait.3>
            /ac "Innovation" <wait.2>
            /ac "Prudent Touch" <wait.3>
            /ac "Basic Touch" <wait.3>
            /ac "Standard Touch" <wait.3>
            /ac "Advanced Touch" <wait.3>
            /ac "Great Strides" <wait.2>
            /ac "Innovation" <wait.2>
            /ac "Observe" <wait.3>
            /ac "Advanced Touch" <wait.3>
            /ac "Great Strides" <wait.2>
            /ac "Byregot's Blessing" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>"#]]
        .assert_eq(&macros[1]);
    }

    #[test]
    /// Test that the last notification skip behavior still works when macro lock is enabled.
    fn skip_last_notification_with_macro_lock() {
        let context = default_context();
        let mut config = MacroViewConfig::default();
        config.split_macro = true;
        config.intro_enabled = true;
        config.notification_enabled = true;
        config.notification_config.avoid_single_action_macro = true;
        let actions = [
            Action::MuscleMemory,
            Action::WasteNot,
            Action::Veneration,
            Action::Groundwork,
            Action::Groundwork,
            Action::Groundwork,
            Action::PrudentSynthesis,
            Action::MasterMend,
            Action::PrudentTouch,
            Action::Innovation,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::PrudentTouch,
            Action::MasterMend,
            Action::Innovation,
            Action::PrudentTouch,
            Action::BasicTouch,
            Action::StandardTouch,
            Action::AdvancedTouch,
            Action::GreatStrides,
            Action::Innovation,
            Action::Observe,
            Action::AdvancedTouch,
            Action::GreatStrides,
            Action::ByregotsBlessing,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
        ];
        let macros = create_macros(&context, &config, &actions, "\n");
        assert_eq!(macros.len(), 2);
        expect_test::expect![[r#"
            /macrolock
            /ac "Muscle Memory" <wait.3>
            /ac "Waste Not" <wait.2>
            /ac "Veneration" <wait.2>
            /ac "Groundwork" <wait.3>
            /ac "Groundwork" <wait.3>
            /ac "Groundwork" <wait.3>
            /ac "Prudent Synthesis" <wait.3>
            /ac "Master's Mend" <wait.3>
            /ac "Prudent Touch" <wait.3>
            /ac "Innovation" <wait.2>
            /ac "Prudent Touch" <wait.3>
            /ac "Prudent Touch" <wait.3>
            /ac "Prudent Touch" <wait.3>
            /echo Macro finished (1/2) <se.1>"#]]
        .assert_eq(&macros[0]);
        expect_test::expect![[r#"
            /ac "Prudent Touch" <wait.3>
            /ac "Master's Mend" <wait.3>
            /ac "Innovation" <wait.2>
            /ac "Prudent Touch" <wait.3>
            /ac "Basic Touch" <wait.3>
            /ac "Standard Touch" <wait.3>
            /ac "Advanced Touch" <wait.3>
            /ac "Great Strides" <wait.2>
            /ac "Innovation" <wait.2>
            /ac "Observe" <wait.3>
            /ac "Advanced Touch" <wait.3>
            /ac "Great Strides" <wait.2>
            /ac "Byregot's Blessing" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>"#]]
        .assert_eq(&macros[1]);
    }

    #[test]
    fn custom_formatters() {
        let context = default_context();
        let mut config = MacroViewConfig::default();
        config.split_macro = true;
        config.intro_enabled = true;
        config.intro_config.default_intro = false;
        config.intro_config.custom_intro_format =
            "/echo food={food}, potion={potion}, stats={craftsmanship} {control} {cp} {level} <wait.5>".to_string();
        config.notification_enabled = true;
        config.notification_config.default_notification = false;
        config.notification_config.custom_notification_format =
            "/echo Crafting {item_name} in progress: {index}/{max_index} <se.1>".to_string();
        config.notification_config.different_last_notification = true;
        config.notification_config.custom_last_notification_format =
            "/echo Crafting {item_name} done: {index}/{max_index} <se.1>".to_string();
        let actions = [
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
            Action::BasicSynthesis,
        ];
        let macros = create_macros(&context, &config, &actions, "\n");
        assert_eq!(macros.len(), 2);
        expect_test::expect![[r#"
            /echo food=Test Food, potion=Test Potion, stats=4900 4800 620 100 <wait.5>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /echo Crafting Test Item in progress: 1/2 <se.1>"#]]
        .assert_eq(&macros[0]);
        expect_test::expect![[r#"
            /ac "Basic Synthesis" <wait.3>
            /ac "Basic Synthesis" <wait.3>
            /echo Crafting Test Item done: 2/2 <se.1>"#]]
        .assert_eq(&macros[1]);
    }
}
