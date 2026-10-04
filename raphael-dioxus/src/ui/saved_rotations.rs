use dioxus::prelude::*;
use raphael_data::Locale;
use raphael_translations::{t, t_format};

use super::{
    icons::{Icon, IconKind},
    use_app_context, use_dialogs, use_solve_state,
    widgets::{ActionIcon, Checkbox, Modal, NumberInput, Warning},
};
use crate::saved_rotations::LoadOperation;

const LOAD_OPERATIONS: [LoadOperation; 3] = [
    LoadOperation::Rotation,
    LoadOperation::RotationRecipe,
    LoadOperation::RotationRecipeConsumables,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RotationList {
    Pinned,
    History,
}

#[component]
pub fn SavedRotationsDialog() -> Element {
    let mut app_context = use_app_context();
    let mut dialogs = use_dialogs();
    let mut settings_open = use_signal(|| false);
    let ctx = app_context.read();
    let locale = ctx.locale;
    let config = ctx.saved_rotations_config.clone();
    let pinned_ids: Vec<u64> = ctx
        .saved_rotations_data
        .pinned
        .iter()
        .map(|rotation| rotation.unique_id)
        .collect();
    let history_ids: Vec<u64> = ctx
        .saved_rotations_data
        .solve_history
        .iter()
        .map(|rotation| rotation.unique_id)
        .collect();
    drop(ctx);

    rsx! {
        Modal {
            title: t!(locale, "Saved macros & solve history"),
            size: "lg",
            onclose: move |_| dialogs.write().saved_rotations = false,
            div { class: "subcard",
                button {
                    class: "subcard-toggle",
                    onclick: move |_| settings_open.toggle(),
                    Icon {
                        kind: if settings_open() { IconKind::ChevronDown } else { IconKind::ChevronRight },
                    }
                    span { {t!(locale, "Settings")} }
                }
                if settings_open() {
                    div { class: "subcard-body",
                        Checkbox {
                            checked: config.load_from_saved_rotations,
                            label: t!(locale, "Load saved rotations when initiating solve"),
                            onchange: move |checked: bool| app_context.write().saved_rotations_config.load_from_saved_rotations = checked,
                        }
                        p { {t!(locale, "Default operation on clicking Load button:")} }
                        div { class: "chips",
                            for operation in LOAD_OPERATIONS {
                                button {
                                    key: "{operation:?}",
                                    class: if config.default_load_operation == operation { "chip active" } else { "chip" },
                                    onclick: move |_| app_context.write().saved_rotations_config.default_load_operation = operation,
                                    "{operation.display(locale)}"
                                }
                            }
                        }
                        Warning {
                            text: t!(
                                locale,
                                "⚠ Rotations saved before v0.21.0 do not contain the necessary information to load recipe data."
                            ),
                        }
                    }
                }
            }

            h3 { class: "section-title", {t!(locale, "Saved macros")} }
            if pinned_ids.is_empty() {
                p { class: "muted", {t!(locale, "No saved macros")} }
            }
            for unique_id in pinned_ids {
                RotationCard { key: "{unique_id}", list: RotationList::Pinned, unique_id }
            }

            div { class: "section-title history-title",
                span { {t!(locale, "Solve history")} }
                span { class: "muted", "{history_ids.len()} /" }
                NumberInput {
                    value: config.max_history_size as i64,
                    min: 20,
                    max: 200,
                    onchange: move |size: i64| app_context.write().saved_rotations_config.max_history_size = size as usize,
                }
            }
            if history_ids.len() > config.max_history_size {
                Warning { text: t!(locale, "⚠ Oldest rotations will be lost on next solve") }
            }
            if history_ids.is_empty() {
                p { class: "muted", {t!(locale, "No solve history")} }
            }
            for unique_id in history_ids {
                RotationCard { key: "{unique_id}", list: RotationList::History, unique_id }
            }
        }
    }
}

fn consumable_name(consumable: Option<(u32, bool)>, locale: Locale) -> String {
    match consumable {
        Some((item_id, hq)) => raphael_data::get_item_name(item_id, hq, locale)
            .unwrap_or(t!(locale, "Unknown item").to_owned()),
        None => t!(locale, "None").to_string(),
    }
}

#[component]
fn RotationCard(list: RotationList, unique_id: u64) -> Element {
    let mut app_context = use_app_context();
    let mut solve_state = use_solve_state();
    let mut expanded = use_signal(|| false);
    let mut menu_open = use_signal(|| false);

    let ctx = app_context.read();
    let locale = ctx.locale;
    let default_load_operation = ctx.saved_rotations_config.default_load_operation;
    let data = &ctx.saved_rotations_data;
    let rotation = match list {
        RotationList::Pinned => data.pinned.iter().find(|r| r.unique_id == unique_id),
        RotationList::History => data.solve_history.iter().find(|r| r.unique_id == unique_id),
    };
    let Some(rotation) = rotation.cloned() else {
        return rsx! {};
    };
    drop(ctx);

    let recipe = rotation.recipe().copied();
    let job_id = recipe.map_or(0, |recipe| recipe.job_id);
    let steps = rotation.actions.len();
    let duration = rotation.duration();
    let has_recipe_info = rotation.recipe_info.is_some();

    let load = use_callback({
        let rotation = rotation.clone();
        move |operation: LoadOperation| {
            menu_open.set(false);
            solve_state.write().actions.clone_from(&rotation.actions);
            let mut ctx = app_context.write();
            match operation {
                LoadOperation::Rotation => (),
                LoadOperation::RotationRecipe => ctx.load_rotation_recipe(&rotation),
                LoadOperation::RotationRecipeConsumables => {
                    ctx.load_rotation_recipe(&rotation);
                    ctx.load_rotation_consumables(&rotation);
                }
            }
        }
    });

    let info_rows = {
        let stats = rotation.crafter_stats;
        let mut rows = Vec::new();
        if let Some(recipe) = recipe {
            rows.push((
                t!(locale, "Recipe"),
                raphael_data::get_item_name(recipe.item_id, false, locale)
                    .unwrap_or(t!(locale, "Unknown item").to_owned()),
            ));
        }
        rows.push((
            t!(locale, "Crafter stats"),
            t_format!(
                locale,
                "{craftsmanship} CMS, {control} Control, {cp} CP",
                craftsmanship = stats.craftsmanship,
                control = stats.control,
                cp = stats.cp,
            ),
        ));
        rows.push((
            t!(locale, "Job"),
            t_format!(
                locale,
                "Level {level} {job}",
                level = stats.level,
                job = recipe.map_or("", |recipe| raphael_data::get_job_name(
                    recipe.job_id,
                    locale
                ))
            ),
        ));
        rows.push((t!(locale, "Food"), consumable_name(rotation.food, locale)));
        rows.push((
            t!(locale, "Potion"),
            consumable_name(rotation.potion, locale),
        ));
        rows.push((t!(locale, "Solver"), rotation.solver.clone()));
        rows
    };

    rsx! {
        div { class: "subcard rotation",
            div { class: "rotation-header",
                button {
                    class: "btn icon-btn ghost",
                    "aria-expanded": expanded(),
                    onclick: move |_| expanded.toggle(),
                    Icon {
                        kind: if expanded() { IconKind::ChevronDown } else { IconKind::ChevronRight },
                    }
                }
                strong { class: "rotation-name", "{rotation.name}" }
                span { class: "muted", {t_format!(locale, "{steps} steps, {duration} seconds")} }
                div { class: "spacer" }
                div { class: "split-btn",
                    button {
                        class: "btn primary",
                        onclick: move |_| load(default_load_operation),
                        {t!(locale, "Load")}
                    }
                    button {
                        class: "btn primary icon-btn",
                        "aria-haspopup": "menu",
                        "aria-expanded": menu_open(),
                        onclick: move |_| menu_open.toggle(),
                        Icon { kind: IconKind::ChevronDown }
                    }
                    if menu_open() {
                        div {
                            class: "menu-backdrop",
                            onclick: move |_| menu_open.set(false),
                        }
                        div { class: "menu", role: "menu",
                            for operation in LOAD_OPERATIONS {
                                button {
                                    key: "{operation:?}",
                                    role: "menuitem",
                                    onclick: move |_| load(operation),
                                    "{operation.display(locale)}"
                                }
                            }
                            if !has_recipe_info {
                                Warning { text: t!(locale, "⚠ pre-v0.21.0 rotation. No recipe data.") }
                            }
                        }
                    }
                }
                if list == RotationList::History {
                    button {
                        class: "btn icon-btn",
                        title: "Pin",
                        onclick: move |_| {
                            let mut ctx = app_context.write();
                            let data = &mut ctx.saved_rotations_data;
                            if let Some(index) = data.solve_history.iter().position(|r| r.unique_id == unique_id) {
                                let rotation = data.solve_history.remove(index).unwrap();
                                data.pinned.push(rotation);
                            }
                        },
                        Icon { kind: IconKind::Pin }
                    }
                }
                button {
                    class: "btn icon-btn danger",
                    onclick: move |_| {
                        let mut ctx = app_context.write();
                        let data = &mut ctx.saved_rotations_data;
                        match list {
                            RotationList::Pinned => data.pinned.retain(|r| r.unique_id != unique_id),
                            RotationList::History => data.solve_history.retain(|r| r.unique_id != unique_id),
                        }
                    },
                    Icon { kind: IconKind::Trash }
                }
            }
            if expanded() {
                dl { class: "info-table",
                    for (key , value) in info_rows {
                        dt { "{key}" }
                        dd { "{value}" }
                    }
                }
            }
            div { class: "action-strip compact",
                for (index , action) in rotation.actions.iter().enumerate() {
                    ActionIcon { key: "{index}", action: *action, job_id, locale }
                }
            }
        }
    }
}
