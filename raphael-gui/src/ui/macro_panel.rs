use dioxus::prelude::*;
use raphael_translations::{t, t_format};

use super::{
    icons::{Icon, IconKind},
    use_app_context, use_dialogs, use_solve_state,
    widgets::{Checkbox, CopyButton, HelpTip, Modal, NumberInput, Radio},
};
use crate::{
    macros::{MacroContextData, MacroViewConfig, create_macros, custom_format_help_text_string},
    platform,
};

#[component]
pub fn MacroPanel() -> Element {
    let app_context = use_app_context();
    let mut solve_state = use_solve_state();
    let mut dialogs = use_dialogs();
    let ctx = app_context.read();
    let locale = ctx.locale;
    let solve = solve_state.read();
    let actions = &solve.actions;
    let duration: u8 = actions.iter().map(|action| action.time_cost()).sum();
    let steps = actions.len();
    let macros = create_macros(
        &MacroContextData::from_app_context(&ctx),
        &ctx.macro_view_config,
        actions,
        platform::macro_newline(),
    );
    let macro_count = macros.len();
    let has_actions = !actions.is_empty();
    drop(solve);
    drop(ctx);

    rsx! {
        section { class: "card macro",
            div { class: "card-header",
                h2 { {t!(locale, "Macro")} }
                div { class: "spacer" }
                span { class: "muted", {t_format!(locale, "{steps} steps, {duration} seconds")} }
                button {
                    class: "btn icon-btn ghost",
                    disabled: !has_actions,
                    onclick: move |_| solve_state.write().actions.clear(),
                    Icon { kind: IconKind::Trash }
                }
                button {
                    class: "btn icon-btn",
                    title: t!(locale, "Macro"),
                    onclick: move |_| dialogs.write().macro_config = true,
                    Icon { kind: IconKind::Settings }
                }
            }
            div { class: "card-body macro-body",
                if macros.is_empty() {
                    div { class: "empty-state",
                        Icon { kind: IconKind::Clipboard }
                    }
                }
                for (index , macro_) in macros.into_iter().enumerate() {
                    div { key: "{index}", class: "macro-block",
                        div { class: "macro-block-header",
                            span { class: "pill", "{index + 1} / {macro_count}" }
                            span { class: "pill", "{macro_.duration}s" }
                            div { class: "spacer" }
                            CopyButton { text: macro_.text.clone() }
                        }
                        pre { tabindex: 0, "{macro_.text}" }
                    }
                }
            }
        }
        MacroConfigDialog {}
    }
}

#[component]
fn MacroConfigDialog() -> Element {
    let mut app_context = use_app_context();
    let mut dialogs = use_dialogs();
    if !dialogs.read().macro_config {
        return rsx! {};
    }
    let (locale, config) = {
        let ctx = app_context.read();
        (ctx.locale, ctx.macro_view_config.clone())
    };
    let mut update =
        move |f: &dyn Fn(&mut MacroViewConfig)| f(&mut app_context.write().macro_view_config);
    let help_text = custom_format_help_text_string(locale);

    rsx! {
        Modal {
            title: t!(locale, "Macro"),
            onclose: move |_| dialogs.write().macro_config = false,
            footer: rsx! {
                button { class: "btn primary block", onclick: move |_| dialogs.write().macro_config = false,
                    {t!(locale, "Close")}
                }
            },
            div { class: "inline-row",
                Checkbox {
                    checked: config.include_delay,
                    label: t!(locale, "Include delay"),
                    onchange: move |checked: bool| update(&|config| config.include_delay = checked),
                }
                div { class: if config.include_delay { "inline-row" } else { "inline-row disabled" },
                    span { {t!(locale, "Extra delay")} }
                    NumberInput {
                        value: i64::from(config.extra_delay),
                        max: 9,
                        disabled: !config.include_delay,
                        onchange: move |delay: i64| update(&|config| config.extra_delay = delay as u8),
                    }
                }
            }
            Checkbox {
                checked: config.split_macro,
                label: t!(locale, "Split macro"),
                onchange: move |checked: bool| update(&|config| config.split_macro = checked),
            }

            Checkbox {
                checked: config.intro_enabled,
                label: t!(locale, "Macro lock / intro"),
                onchange: move |checked: bool| update(&|config| config.intro_enabled = checked),
            }
            fieldset { class: "indent", disabled: !config.intro_enabled,
                Radio {
                    checked: config.intro_config.default_intro,
                    label: t!(locale, "Use \"/macrolock\" as macro intro"),
                    disabled: !config.intro_enabled,
                    onselect: move |_| update(&|config| config.intro_config.default_intro = true),
                }
                div { class: "inline-row",
                    Radio {
                        checked: !config.intro_config.default_intro,
                        label: t!(locale, "Use custom intro format"),
                        disabled: !config.intro_enabled,
                        onselect: move |_| update(&|config| config.intro_config.default_intro = false),
                    }
                    HelpTip { text: help_text }
                }
                input {
                    class: "text-input mono",
                    r#type: "text",
                    placeholder: "/macrolock <wait.5>",
                    disabled: !config.intro_enabled || config.intro_config.default_intro,
                    value: "{config.intro_config.custom_intro_format}",
                    oninput: move |event| {
                        app_context.write().macro_view_config.intro_config.custom_intro_format = event.value();
                    },
                }
            }

            Checkbox {
                checked: config.notification_enabled,
                label: t!(locale, "End-of-macro notification"),
                onchange: move |checked: bool| update(&|config| config.notification_enabled = checked),
            }
            fieldset { class: "indent", disabled: !config.notification_enabled,
                div { class: "inline-row",
                    Radio {
                        checked: config.notification_config.default_notification,
                        label: t!(locale, "Use default notification"),
                        disabled: !config.notification_enabled,
                        onselect: move |_| update(&|config| config.notification_config.default_notification = true),
                    }
                    NumberInput {
                        value: i64::from(config.notification_config.notification_sound),
                        min: 1,
                        max: 16,
                        prefix: "<se.",
                        suffix: ">",
                        disabled: !config.notification_enabled
                            || !config.notification_config.default_notification,
                        onchange: move |sound: i64| update(&|config| config.notification_config.notification_sound = sound as u8),
                    }
                }
                div { class: "inline-row",
                    Radio {
                        checked: !config.notification_config.default_notification,
                        label: t!(locale, "Use custom notification format"),
                        disabled: !config.notification_enabled,
                        onselect: move |_| update(&|config| config.notification_config.default_notification = false),
                    }
                    HelpTip { text: help_text }
                }
                fieldset {
                    class: "indent",
                    disabled: !config.notification_enabled
                        || config.notification_config.default_notification,
                    input {
                        class: "text-input mono",
                        r#type: "text",
                        placeholder: "/echo Done {{index}}/{{max_index}} <se.1>",
                        value: "{config.notification_config.custom_notification_format}",
                        oninput: move |event| {
                            app_context.write().macro_view_config.notification_config.custom_notification_format = event
                                .value();
                        },
                    }
                    Checkbox {
                        checked: config.notification_config.different_last_notification,
                        label: t!(locale, "Use different format for last notification"),
                        disabled: !config.notification_enabled
                            || config.notification_config.default_notification,
                        onchange: move |checked: bool| update(&|config| config.notification_config.different_last_notification = checked),
                    }
                    input {
                        class: "text-input mono",
                        r#type: "text",
                        placeholder: "/echo All macros done <se.2>",
                        disabled: !config.notification_config.different_last_notification,
                        value: "{config.notification_config.custom_last_notification_format}",
                        oninput: move |event| {
                            app_context
                                .write()
                                .macro_view_config
                                .notification_config
                                .custom_last_notification_format = event.value();
                        },
                    }
                }
                Checkbox {
                    checked: config.notification_config.avoid_single_action_macro,
                    label: t!(locale, "Avoid single-action macros"),
                    disabled: !config.notification_enabled,
                    onchange: move |checked: bool| update(&|config| config.notification_config.avoid_single_action_macro = checked),
                }
                p {
                    class: if config.notification_config.avoid_single_action_macro { "hint indent" } else { "hint indent disabled" },
                    {t!(locale, "Skip last notification if doing so avoids creating a macro with a single action.")}
                }
            }
        }
    }
}
