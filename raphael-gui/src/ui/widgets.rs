use dioxus::prelude::*;
use raphael_data::{
    Consumable, Ingredient, Locale, Recipe, action_name, get_item_name, get_job_name,
    get_stellar_mission_name,
};
use raphael_sim::Action;
use raphael_translations::t;

use super::icons::{Icon, IconKind};
use crate::platform;

/// The game does not have a text glyph for the expert recipe icon. The bundled icon font places it here.
const EXPERT_RECIPE_ICON_CHAR: char = '\u{e100}';

/// Removes decorative symbols that some translated strings start with (e.g. "⚠ ...", "🔍 Search"),
/// since the UI renders proper icons instead.
pub fn strip_icon(text: &str) -> &str {
    text.trim_start_matches(['⚠', '🔍', '⚙', '🗐', '📋', '✏', '📌', '🗑', '\u{fe0f}', ' '])
}

/// Formats a number with thousands separators.
pub fn format_thousands(value: usize) -> String {
    value
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

pub fn job_names(locale: Locale) -> Vec<String> {
    (0..8)
        .map(|job_id| get_job_name(job_id, locale).to_string())
        .collect()
}

#[component]
pub fn NumberInput(
    value: i64,
    #[props(default = 0)] min: i64,
    #[props(default = i64::from(u16::MAX))] max: i64,
    #[props(default)] disabled: bool,
    #[props(default)] highlighted: bool,
    /// Only commit the value when the input loses focus or Enter is pressed.
    #[props(default)]
    lazy: bool,
    #[props(into, default)] prefix: String,
    #[props(into, default)] suffix: String,
    #[props(into, default)] title: String,
    onchange: Option<EventHandler<i64>>,
) -> Element {
    // Text being edited by the user. `None` when the input shows `value`.
    let mut draft = use_signal(|| None::<String>);
    let shown = draft().unwrap_or_else(|| value.to_string());
    let mut class = String::from("num");
    if highlighted {
        class.push_str(" highlighted");
    }
    if disabled {
        class.push_str(" disabled");
    }
    rsx! {
        div { class, title,
            if !prefix.is_empty() {
                span { class: "num-affix", "{prefix}" }
            }
            input {
                r#type: "number",
                inputmode: "numeric",
                style: "width: calc(5ch + 20px)",
                min,
                max,
                disabled,
                value: "{shown}",
                oninput: move |event| {
                    let text = event.value();
                    if !lazy
                        && let Ok(parsed) = text.trim().parse::<i64>()
                        && (min..=max).contains(&parsed)
                        && let Some(handler) = onchange
                    {
                        handler.call(parsed);
                    }
                    draft.set(Some(text));
                },
                onchange: move |event| {
                    if let Ok(parsed) = event.value().trim().parse::<i64>()
                        && let Some(handler) = onchange
                    {
                        handler.call(parsed.clamp(min, max));
                    }
                    draft.set(None);
                },
                onblur: move |_| draft.set(None),
            }
            if !suffix.is_empty() {
                span { class: "num-affix", "{suffix}" }
            }
        }
    }
}

#[component]
pub fn Select(
    selected: usize,
    options: Vec<String>,
    onchange: EventHandler<usize>,
    #[props(default)] disabled: bool,
    #[props(into, default)] class: String,
) -> Element {
    rsx! {
        div { class: "select {class}",
            select {
                disabled,
                value: "{selected}",
                onchange: move |event| {
                    if let Ok(index) = event.value().parse::<usize>() {
                        onchange.call(index);
                    }
                },
                for (index , label) in options.iter().enumerate() {
                    option { key: "{index}", value: "{index}", selected: index == selected, "{label}" }
                }
            }
            Icon { kind: IconKind::ChevronDown }
        }
    }
}

#[component]
pub fn Checkbox(
    checked: bool,
    #[props(into)] label: String,
    #[props(default)] disabled: bool,
    onchange: EventHandler<bool>,
) -> Element {
    rsx! {
        label { class: if disabled { "check disabled" } else { "check" },
            input {
                r#type: "checkbox",
                checked,
                disabled,
                onchange: move |event| onchange.call(event.checked()),
            }
            span { class: "check-box",
                Icon { kind: IconKind::Check }
            }
            span { class: "check-label", "{label}" }
        }
    }
}

#[component]
pub fn Radio(
    checked: bool,
    #[props(into)] label: String,
    #[props(default)] disabled: bool,
    onselect: EventHandler<()>,
) -> Element {
    rsx! {
        label { class: if disabled { "check radio disabled" } else { "check radio" },
            input {
                r#type: "radio",
                checked,
                disabled,
                onchange: move |_| onselect.call(()),
            }
            span { class: "check-box" }
            span { class: "check-label", "{label}" }
        }
    }
}

/// Segmented control for picking one of a few options.
#[component]
pub fn Segmented(selected: usize, options: Vec<String>, onchange: EventHandler<usize>) -> Element {
    rsx! {
        div { class: "segmented", role: "radiogroup",
            for (index , label) in options.iter().enumerate() {
                button {
                    key: "{index}",
                    class: if index == selected { "active" },
                    role: "radio",
                    "aria-checked": index == selected,
                    onclick: move |_| onchange.call(index),
                    "{label}"
                }
            }
        }
    }
}

#[component]
pub fn HelpTip(#[props(into)] text: String, #[props(default)] align_left: bool) -> Element {
    rsx! {
        span {
            class: if align_left { "help align-left" } else { "help" },
            tabindex: 0,
            "aria-label": "{text}",
            Icon { kind: IconKind::Help }
            span { class: "tip", "{text}" }
        }
    }
}

#[component]
pub fn Warning(#[props(into)] text: String, children: Element) -> Element {
    rsx! {
        div { class: "warning",
            Icon { kind: IconKind::Alert }
            div {
                p { "{strip_icon(&text)}" }
                {children}
            }
        }
    }
}

/// Icon button that shows a check mark for a short while after being clicked.
#[component]
pub fn CopyButton(
    text: String,
    #[props(into, default)] label: String,
    #[props(into, default)] title: String,
    #[props(default)] disabled: bool,
) -> Element {
    let mut copied = use_signal(|| false);
    rsx! {
        button {
            class: if label.is_empty() { "btn icon-btn ghost" } else { "btn" },
            class: if copied() { "copied" },
            title,
            disabled,
            onclick: move |_| {
                platform::copy_to_clipboard(text.clone());
                copied.set(true);
                spawn(async move {
                    platform::sleep(web_time::Duration::from_millis(1200)).await;
                    copied.set(false);
                });
            },
            Icon { kind: if copied() { IconKind::Check } else { IconKind::Copy } }
            if !label.is_empty() {
                span { "{label}" }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameSource {
    Item { item_id: u32, hq: bool },
    Recipe { item_id: u32, expert: bool },
    Mission { mission_id: u32 },
}

impl From<&Consumable> for NameSource {
    fn from(consumable: &Consumable) -> Self {
        Self::Item {
            item_id: consumable.item_id,
            hq: consumable.hq,
        }
    }
}

impl From<&Ingredient> for NameSource {
    fn from(ingredient: &Ingredient) -> Self {
        Self::Item {
            item_id: ingredient.item_id,
            hq: false,
        }
    }
}

impl From<&Recipe> for NameSource {
    fn from(recipe: &Recipe) -> Self {
        Self::Recipe {
            item_id: recipe.item_id,
            expert: recipe.is_expert,
        }
    }
}

/// Name of an item, recipe or mission, with a button to copy the name.
#[component]
pub fn GameDataName(source: NameSource, locale: Locale) -> Element {
    let name = match source {
        NameSource::Item { item_id, hq } => {
            get_item_name(item_id, hq, locale).unwrap_or(t!(locale, "Unknown item").to_owned())
        }
        NameSource::Recipe { item_id, .. } => {
            get_item_name(item_id, false, locale).unwrap_or(t!(locale, "Unknown item").to_owned())
        }
        NameSource::Mission { mission_id } => get_stellar_mission_name(mission_id, locale)
            .unwrap_or(t!(locale, "Unknown mission"))
            .to_owned(),
    };
    let copy_text = name
        .trim_end_matches([' ', raphael_data::HQ_ICON_CHAR, raphael_data::CL_ICON_CHAR])
        .to_string();
    let copy_title = match source {
        NameSource::Item { .. } | NameSource::Recipe { .. } => t!(locale, "Copy item name"),
        NameSource::Mission { .. } => t!(locale, "Copy mission name"),
    };
    let expert = matches!(source, NameSource::Recipe { expert: true, .. });
    rsx! {
        span { class: "game-name",
            span { class: "game-name-text", title: "{name}",
                "{name}"
                if expert {
                    span { class: "expert-icon", " {EXPERT_RECIPE_ICON_CHAR}" }
                }
            }
            CopyButton { text: copy_text, title: copy_title }
        }
    }
}

pub fn action_icon_url(action: Action, job_id: u8) -> String {
    format!(
        "action-icons/{}/{}.webp",
        get_job_name(job_id, Locale::EN),
        action_name(action, Locale::EN)
    )
}

#[component]
pub fn ActionIcon(
    action: Action,
    job_id: u8,
    locale: Locale,
    #[props(default)] failed: bool,
    step: Option<usize>,
) -> Element {
    let name = action_name(action, locale);
    rsx! {
        div { class: if failed { "action failed" } else { "action" }, title: "{name}",
            img {
                src: action_icon_url(action, job_id),
                alt: "{name}",
                draggable: false,
            }
            if failed {
                img {
                    class: "action-disabled",
                    src: "action-icons/disabled.webp",
                    alt: "",
                    draggable: false,
                }
            }
            if let Some(step) = step {
                span { class: "action-step", "{step}" }
            }
        }
    }
}

#[component]
pub fn Modal(
    #[props(into)] title: String,
    onclose: EventHandler<()>,
    #[props(into, default = "md".to_string())] size: String,
    #[props(default = true)] closable: bool,
    footer: Option<Element>,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "modal-backdrop",
            onclick: move |_| {
                if closable {
                    onclose.call(());
                }
            },
            div {
                class: "modal modal-{size}",
                role: "dialog",
                "aria-modal": "true",
                tabindex: -1,
                onclick: move |event| event.stop_propagation(),
                onkeydown: move |event| {
                    if closable && event.key() == Key::Escape {
                        onclose.call(());
                    }
                },
                onmounted: move |event| async move {
                    let _ = event.set_focus(true).await;
                },
                div { class: "modal-header",
                    h2 { "{title}" }
                    if closable {
                        button {
                            class: "btn icon-btn ghost",
                            onclick: move |_| onclose.call(()),
                            Icon { kind: IconKind::Close }
                        }
                    }
                }
                div { class: "modal-body", {children} }
                if let Some(footer) = footer {
                    div { class: "modal-footer", {footer} }
                }
            }
        }
    }
}

/// Header of a card with a collapse toggle.
#[component]
pub fn CardHeader(
    #[props(into)] title: String,
    collapsed: Option<Signal<bool>>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "card-header",
            if let Some(mut collapsed) = collapsed {
                button {
                    class: "btn icon-btn ghost collapse-btn",
                    "aria-expanded": !collapsed(),
                    onclick: move |_| collapsed.toggle(),
                    Icon {
                        kind: if collapsed() { IconKind::ChevronRight } else { IconKind::ChevronDown },
                    }
                }
            }
            h2 { "{title}" }
            {children}
        }
    }
}
