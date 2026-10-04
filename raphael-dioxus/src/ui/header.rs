use dioxus::prelude::*;
use raphael_data::Locale;
use raphael_translations::t;

use super::{
    icons::{Icon, IconKind},
    use_app_context, use_dialogs,
    widgets::Select,
};

pub const SELECTABLE_LOCALES: [Locale; 7] = [
    Locale::EN,
    Locale::DE,
    Locale::FR,
    Locale::JP,
    Locale::CN,
    Locale::KR,
    Locale::TW,
];

#[component]
pub fn Header() -> Element {
    let mut app_context = use_app_context();
    let mut dialogs = use_dialogs();
    let locale = app_context.read().locale;
    let locale_index = SELECTABLE_LOCALES
        .iter()
        .position(|l| *l == locale)
        .unwrap_or_default();

    rsx! {
        header { class: "topbar",
            div { class: "brand",
                span { class: "brand-mark", "R" }
                div { class: "brand-text",
                    span { class: "brand-name", "Raphael" }
                    span { class: "brand-sub", "FFXIV Crafting Solver" }
                }
                span { class: "pill", "v{env!(\"CARGO_PKG_VERSION\")}" }
            }
            nav { class: "links",
                a {
                    href: "https://github.com/KonaeAkira/raphael-rs",
                    target: "_blank",
                    rel: "noopener",
                    {t!(locale, "View source on GitHub")}
                }
                a {
                    href: "https://discord.com/invite/m2aCy3y8he",
                    target: "_blank",
                    rel: "noopener",
                    {t!(locale, "Join Discord")}
                }
                a {
                    href: "https://ko-fi.com/konaeakira",
                    target: "_blank",
                    rel: "noopener",
                    {t!(locale, "Support me on Ko-fi")}
                }
            }
            div { class: "topbar-actions",
                Select {
                    class: "locale-select",
                    selected: locale_index,
                    options: SELECTABLE_LOCALES.iter().map(|l| l.short_code().to_string()).collect::<Vec<_>>(),
                    onchange: move |index: usize| app_context.write().locale = SELECTABLE_LOCALES[index],
                }
                button {
                    class: "btn icon-btn",
                    title: t!(locale, "Saved macros & solve history"),
                    onclick: move |_| dialogs.write().saved_rotations = true,
                    Icon { kind: IconKind::Bookmark }
                }
                button {
                    class: "btn icon-btn",
                    title: super::widgets::strip_icon(t!(locale, "⚙ Settings")),
                    onclick: move |_| dialogs.write().settings = true,
                    Icon { kind: IconKind::Settings }
                }
            }
        }
    }
}
