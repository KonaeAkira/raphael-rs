use std::rc::Rc;

use dioxus::prelude::*;
use raphael_data::{
    Consumable, CrafterStats, Locale, control_bonus, cp_bonus, craftsmanship_bonus, find_meals,
    find_potions,
};
use raphael_translations::{t, t_format};

use super::{
    icons::{Icon, IconKind},
    use_app_context, use_search_text,
    virtual_list::VirtualList,
    widgets::{CardHeader, GameDataName, NameSource, strip_icon},
};
use crate::context::{AppContext, SearchText};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumableKind {
    Food,
    Potion,
}

impl ConsumableKind {
    fn title(self, locale: Locale) -> &'static str {
        match self {
            Self::Food => t!(locale, "Food"),
            Self::Potion => t!(locale, "Potion"),
        }
    }

    fn search_text(self, search_text: &SearchText) -> &String {
        match self {
            Self::Food => &search_text.food,
            Self::Potion => &search_text.potion,
        }
    }

    fn search_text_mut(self, search_text: &mut SearchText) -> &mut String {
        match self {
            Self::Food => &mut search_text.food,
            Self::Potion => &mut search_text.potion,
        }
    }

    fn selected(self, app_context: &AppContext) -> Option<Consumable> {
        match self {
            Self::Food => app_context.selected_food,
            Self::Potion => app_context.selected_potion,
        }
    }

    fn selected_mut(self, app_context: &mut AppContext) -> &mut Option<Consumable> {
        match self {
            Self::Food => &mut app_context.selected_food,
            Self::Potion => &mut app_context.selected_potion,
        }
    }
}

pub fn effect_string(
    consumable: Consumable,
    crafter_stats: &CrafterStats,
    locale: Locale,
) -> String {
    let consumables = &[Some(consumable)];
    let mut effects = Vec::new();
    if consumable.craft_rel != 0 {
        effects.push(t_format!(
            locale,
            "Crafts. +{rel}% ({val}), ",
            rel = consumable.craft_rel,
            val = craftsmanship_bonus(crafter_stats.craftsmanship, consumables)
        ));
    }
    if consumable.control_rel != 0 {
        effects.push(t_format!(
            locale,
            "Control +{rel}% ({val}), ",
            rel = consumable.control_rel,
            val = control_bonus(crafter_stats.control, consumables)
        ));
    }
    if consumable.cp_rel != 0 {
        effects.push(t_format!(
            locale,
            "CP +{rel}% ({val}), ",
            rel = consumable.cp_rel,
            val = cp_bonus(crafter_stats.cp, consumables)
        ));
    }
    let mut effect = effects.concat();
    effect.pop(); // This will potentially not work for JP & KR
    effect.pop();
    effect
}

#[derive(Clone)]
struct Results(Rc<Vec<&'static Consumable>>);

impl PartialEq for Results {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[component]
pub fn ConsumablePanel(kind: ConsumableKind) -> Element {
    let mut app_context = use_app_context();
    let mut search = use_search_text();
    let collapsed = use_signal(|| false);
    let query = use_memo(move || {
        let ctx = app_context.read();
        (kind.search_text(&search.read()).clone(), ctx.locale)
    });
    let results = use_memo(move || {
        let (text, locale) = query();
        let results = match kind {
            ConsumableKind::Food => find_meals(&text, locale).collect(),
            ConsumableKind::Potion => find_potions(&text, locale).collect(),
        };
        Results(Rc::new(results))
    });

    let (locale, search_text, selected, stats) = {
        let ctx = app_context.read();
        (
            ctx.locale,
            kind.search_text(&search.read()).clone(),
            kind.selected(&ctx),
            *ctx.active_stats(),
        )
    };
    let results = results();
    let count = results.0.len();
    let render_row = use_callback(move |index: usize| {
        let Some(&&item) = results.0.get(index) else {
            return rsx! {};
        };
        let is_selected = selected
            .is_some_and(|selected| (selected.item_id, selected.hq) == (item.item_id, item.hq));
        rsx! {
            div {
                key: "{item.item_id}-{item.hq}",
                class: if is_selected { "vrow row selectable selected" } else { "vrow row selectable" },
                onclick: move |_| *kind.selected_mut(&mut app_context.write()) = Some(item),
                GameDataName { source: NameSource::from(&item), locale }
                span { class: "row-effect", {effect_string(item, &stats, locale)} }
            }
        }
    });

    rsx! {
        section { class: "card",
            CardHeader { title: kind.title(locale), collapsed,
                match selected {
                    Some(item) => rsx! {
                        GameDataName { source: NameSource::from(&item), locale }
                    },
                    None => rsx! {
                        span { class: "muted", {t!(locale, "None")} }
                    },
                }
                div { class: "spacer" }
                button {
                    class: "btn icon-btn ghost",
                    disabled: selected.is_none(),
                    onclick: move |_| *kind.selected_mut(&mut app_context.write()) = None,
                    Icon { kind: IconKind::Trash }
                }
            }
            if !collapsed() {
                div { class: "card-body",
                    div { class: "search",
                        Icon { kind: IconKind::Search }
                        input {
                            r#type: "search",
                            placeholder: strip_icon(t!(locale, "🔍 Search")),
                            value: "{search_text}",
                            oninput: move |event| {
                                *kind.search_text_mut(&mut search.write()) = event.value().replace('\0', "");
                            },
                        }
                    }
                    for key in std::iter::once(format!("{:?}", query())) {
                        VirtualList {
                            key: "{key}",
                            class: "consumable-list",
                            count,
                            deps: (locale, selected.map(|item| (item.item_id, item.hq)), stats),
                            render_row,
                        }
                    }
                }
            }
        }
    }
}
