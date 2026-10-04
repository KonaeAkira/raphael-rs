use dioxus::prelude::*;
use raphael_data::{action_name, get_job_name};
use raphael_sim::Action;
use raphael_translations::t;

use super::{
    icons::{Icon, IconKind},
    use_app_context, use_dialogs,
    widgets::{Checkbox, CopyButton, Modal, NumberInput, strip_icon},
};
use crate::config::CrafterConfig;

#[component]
pub fn StatsEditDialog() -> Element {
    let mut app_context = use_app_context();
    let mut dialogs = use_dialogs();
    let mut paste_text = use_signal(String::new);
    let mut pasted = use_signal(|| false);
    let (locale, crafter_config) = {
        let ctx = app_context.read();
        (ctx.locale, ctx.crafter_config)
    };
    let serialized_config = ron::to_string(&crafter_config).unwrap_or_default();

    rsx! {
        Modal {
            title: t!(locale, "Crafter stats"),
            onclose: move |_| dialogs.write().stats_edit = false,
            footer: rsx! {
                CopyButton {
                    text: serialized_config,
                    label: strip_icon(t!(locale, "🗐 Copy crafter config")),
                }
                div { class: if pasted() { "paste-input pasted" } else { "paste-input" },
                    Icon { kind: if pasted() { IconKind::Check } else { IconKind::Clipboard } }
                    input {
                        r#type: "text",
                        placeholder: strip_icon(t!(locale, "📋 Paste config here to load")),
                        value: "{paste_text}",
                        oninput: move |event| {
                            let text = event.value();
                            if let Ok(config) = ron::from_str::<CrafterConfig>(&text) {
                                let mut ctx = app_context.write();
                                let selected_job = ctx.crafter_config.selected_job;
                                ctx.crafter_config = config;
                                ctx.crafter_config.selected_job = selected_job;
                                paste_text.set(String::new());
                                pasted.set(true);
                                spawn(async move {
                                    crate::platform::sleep(web_time::Duration::from_millis(1200)).await;
                                    pasted.set(false);
                                });
                            } else {
                                paste_text.set(text);
                            }
                        },
                    }
                }
            },
            div { class: "job-stats-list",
                for job_id in 0..8u8 {
                    {
                        let stats = crafter_config.crafter_stats[usize::from(job_id)];
                        let mut update = move |f: &dyn Fn(&mut raphael_data::CrafterStats)| {
                            f(&mut app_context.write().crafter_config.crafter_stats[usize::from(job_id)]);
                        };
                        rsx! {
                            div { key: "{job_id}", class: "job-stats",
                                div { class: "job-stats-header",
                                    strong { {get_job_name(job_id, locale)} }
                                    button {
                                        class: "btn small",
                                        onclick: move |_| {
                                            let mut ctx = app_context.write();
                                            ctx.crafter_config.crafter_stats = [stats; 8];
                                        },
                                        {t!(locale, "Copy to all jobs")}
                                    }
                                }
                                div { class: "job-stats-grid",
                                    label {
                                        span { {t!(locale, "Craftsmanship")} }
                                        NumberInput {
                                            value: i64::from(stats.craftsmanship),
                                            min: 1,
                                            max: 9999,
                                            onchange: move |value: i64| update(&|stats| stats.craftsmanship = value as u16),
                                        }
                                    }
                                    label {
                                        span { {t!(locale, "Control")} }
                                        NumberInput {
                                            value: i64::from(stats.control),
                                            min: 1,
                                            max: 9999,
                                            onchange: move |value: i64| update(&|stats| stats.control = value as u16),
                                        }
                                    }
                                    label {
                                        span { {t!(locale, "CP")} }
                                        NumberInput {
                                            value: i64::from(stats.cp),
                                            min: 1,
                                            max: 999,
                                            onchange: move |value: i64| update(&|stats| stats.cp = value as u16),
                                        }
                                    }
                                    label {
                                        span { {t!(locale, "Job level")} }
                                        NumberInput {
                                            value: i64::from(stats.level),
                                            min: 1,
                                            max: 100,
                                            onchange: move |value: i64| update(&|stats| stats.level = value as u8),
                                        }
                                    }
                                }
                                div { class: "check-list horizontal",
                                    Checkbox {
                                        checked: stats.manipulation,
                                        label: action_name(Action::Manipulation, locale),
                                        onchange: move |checked: bool| update(&|stats| stats.manipulation = checked),
                                    }
                                    Checkbox {
                                        checked: stats.heart_and_soul,
                                        label: action_name(Action::HeartAndSoul, locale),
                                        onchange: move |checked: bool| update(&|stats| stats.heart_and_soul = checked),
                                    }
                                    Checkbox {
                                        checked: stats.quick_innovation,
                                        label: action_name(Action::QuickInnovation, locale),
                                        onchange: move |checked: bool| update(&|stats| stats.quick_innovation = checked),
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
