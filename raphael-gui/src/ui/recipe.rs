use std::rc::Rc;

use dioxus::prelude::*;
use raphael_data::{
    CosmicExplorationZone, RLVLS, Recipe, RecipeSearchEntry, RecipeSearchQuery,
    StellarMissionSearchEntry, StellarSearchQuery, find_recipes, find_stellar_missions,
    get_cosmic_exploration_zone_name, get_game_settings, get_job_name,
};
use raphael_translations::{t, t_format};

use super::{
    icons::{Icon, IconKind},
    use_app_context, use_dialogs,
    virtual_list::VirtualList,
    widgets::{
        CardHeader, Checkbox, GameDataName, Modal, NameSource, NumberInput, Select, Warning,
        strip_icon,
    },
};
use crate::{
    config::{QualitySource, RecipeConfiguration, RecipeSource},
    context::{AppContext, RecipeSearchDomain},
};

/// Search results shared between renders. Compares by identity so that memos only notify
/// subscribers when the results were actually recomputed.
#[derive(Clone)]
struct Results<T>(Rc<Vec<T>>);

impl<T> PartialEq for Results<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn select_normal_recipe(app_context: &mut AppContext, recipe_id: u32, recipe: Recipe) {
    app_context.crafter_config.selected_job = recipe.job_id;
    app_context.recipe_config = RecipeConfiguration {
        recipe_source: RecipeSource::Normal {
            id: recipe_id,
            data: recipe,
        },
        quality_source: QualitySource::HqMaterialList([0; 6]),
    };
    app_context.solver_config.stellar_steady_hand_charges = 0;
}

fn selected_recipe_id(app_context: &AppContext) -> Option<u32> {
    match app_context.recipe_config.recipe_source {
        RecipeSource::Normal { id, .. } => Some(id),
        RecipeSource::Custom { .. } => None,
    }
}

#[component]
pub fn RecipePanel() -> Element {
    let mut app_context = use_app_context();
    let collapsed = use_signal(|| false);
    let (locale, recipe, show_custom) = {
        let ctx = app_context.read();
        (
            ctx.locale,
            *ctx.recipe_config.recipe(),
            ctx.search_state.recipe.show_custom_recipe_select,
        )
    };

    rsx! {
        section { class: "card",
            CardHeader { title: t!(locale, "Recipe"), collapsed,
                GameDataName { source: NameSource::from(&recipe), locale }
                div { class: "spacer" }
                Checkbox {
                    checked: show_custom,
                    label: t!(locale, "Custom"),
                    onchange: move |checked: bool| {
                        let mut ctx = app_context.write();
                        ctx.search_state.recipe.show_custom_recipe_select = checked;
                        if checked && let RecipeSource::Normal { .. } = ctx.recipe_config.recipe_source {
                            let default_game_settings = get_game_settings(
                                *ctx.recipe_config.recipe(),
                                None,
                                *ctx.active_stats(),
                                ctx.selected_food,
                                ctx.selected_potion,
                            );
                            let level = ctx.active_stats().level;
                            ctx.recipe_config.recipe_source = ctx
                                .recipe_config
                                .recipe_source
                                .into_custom(level, default_game_settings);
                            ctx.recipe_config.quality_source = QualitySource::Value(0);
                        }
                    },
                }
            }
            if !collapsed() {
                div { class: "card-body",
                    if show_custom {
                        CustomRecipeEditor {}
                    } else {
                        RecipeSearch {}
                    }
                }
            }
        }
        RecipeFilterDialog {}
    }
}

#[component]
fn RecipeSearch() -> Element {
    let mut app_context = use_app_context();
    let mut dialogs = use_dialogs();
    let (locale, domain, search_text, filters_active) = {
        let ctx = app_context.read();
        let search = &ctx.search_state.recipe;
        (
            ctx.locale,
            search.search_domain,
            search.search_text.clone(),
            search.filters_active(),
        )
    };
    let domains = [
        RecipeSearchDomain::Recipes,
        RecipeSearchDomain::StellarMissions,
    ];

    rsx! {
        div { class: "search-row",
            Select {
                selected: domains.iter().position(|d| *d == domain).unwrap_or_default(),
                options: domains.iter().map(|d| d.display(locale).to_string()).collect::<Vec<_>>(),
                onchange: move |index: usize| app_context.write().search_state.recipe.search_domain = domains[index],
            }
            div { class: "search",
                Icon { kind: IconKind::Search }
                input {
                    r#type: "search",
                    placeholder: strip_icon(t!(locale, "🔍 Search")),
                    value: "{search_text}",
                    oninput: move |event| {
                        app_context.write().search_state.recipe.search_text = event
                            .value()
                            .replace('\0', "");
                    },
                }
            }
            button {
                class: if filters_active { "btn icon-btn active" } else { "btn icon-btn" },
                title: t!(locale, "Recipe filters"),
                onclick: move |_| dialogs.write().recipe_filters = true,
                Icon { kind: IconKind::Filter }
            }
        }
        match domain {
            RecipeSearchDomain::Recipes => rsx! {
                RecipeList {}
            },
            RecipeSearchDomain::StellarMissions => rsx! {
                MissionList {}
            },
        }
    }
}

#[component]
fn RecipeList() -> Element {
    let mut app_context = use_app_context();
    let query = use_memo(move || {
        let ctx = app_context.read();
        let search = &ctx.search_state.recipe;
        (
            search.search_text.clone(),
            ctx.locale,
            search.filters.construct_recipe_filter(&ctx.crafter_config),
        )
    });
    let results = use_memo(move || {
        let (text, locale, filters) = query();
        Results(Rc::new(
            find_recipes(RecipeSearchQuery {
                text: &text,
                locale,
                filters,
            })
            .collect::<Vec<RecipeSearchEntry>>(),
        ))
    });
    // Subscribe only to the values used for rendering, not to the whole app context.
    let (locale, selected_id) = use_memo(move || {
        let ctx = app_context.read();
        (ctx.locale, selected_recipe_id(&ctx))
    })();
    let results = results();
    let count = results.0.len();
    let render_row = use_callback(move |index: usize| {
        let Some(&(recipe_id, recipe)) = results.0.get(index) else {
            return rsx! {};
        };
        let selected = selected_id == Some(recipe_id);
        rsx! {
            div {
                key: "{recipe_id}",
                class: if selected { "vrow row selectable selected" } else { "vrow row selectable" },
                onclick: move |_| select_normal_recipe(&mut app_context.write(), recipe_id, *recipe),
                span { class: "row-job", {get_job_name(recipe.job_id, locale)} }
                GameDataName { source: NameSource::from(recipe), locale }
            }
        }
    });

    let list_key = format!("{:?}", query());
    rsx! {
        // Re-create the list when the query changes so that it scrolls back to the top.
        for key in std::iter::once(list_key) {
            VirtualList { key: "{key}", class: "recipe-list", count, render_row }
        }
    }
}

#[component]
fn MissionList() -> Element {
    let mut app_context = use_app_context();
    let query = use_memo(move || {
        let ctx = app_context.read();
        let search = &ctx.search_state.recipe;
        (
            search.search_text.clone(),
            ctx.locale,
            search
                .filters
                .construct_stellar_mission_filter(&ctx.crafter_config),
        )
    });
    let results = use_memo(move || {
        let (text, locale, filters) = query();
        Results(Rc::new(
            find_stellar_missions(StellarSearchQuery {
                text: &text,
                locale,
                filters,
            })
            .collect::<Vec<StellarMissionSearchEntry>>(),
        ))
    });
    // Subscribe only to the values used for rendering, not to the whole app context.
    let (locale, selected_id) = use_memo(move || {
        let ctx = app_context.read();
        (ctx.locale, selected_recipe_id(&ctx))
    })();
    let results = results();

    rsx! {
        div { class: "list mission-list",
            for &(mission_id , mission) in results.0.iter() {
                div { key: "{mission_id}", class: "mission",
                    div { class: "row mission-header",
                        span { class: "row-job", {get_job_name(mission.job_id, locale)} }
                        GameDataName { source: NameSource::Mission { mission_id }, locale }
                    }
                    for &recipe_id in mission.recipe_ids {
                        {
                            let recipe = raphael_data::RECIPES[recipe_id];
                            let selected = selected_id == Some(recipe_id);
                            rsx! {
                                div {
                                    key: "{recipe_id}",
                                    class: if selected { "row selectable selected sub" } else { "row selectable sub" },
                                    onclick: move |_| select_normal_recipe(&mut app_context.write(), recipe_id, recipe),
                                    GameDataName { source: NameSource::from(&recipe), locale }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CustomRecipeEditor() -> Element {
    let mut app_context = use_app_context();
    let ctx = app_context.read();
    let locale = ctx.locale;
    let default_game_settings = get_game_settings(
        *ctx.recipe_config.recipe(),
        None,
        *ctx.active_stats(),
        ctx.selected_food,
        ctx.selected_potion,
    );
    let RecipeSource::Custom {
        data: recipe,
        overrides,
    } = ctx.recipe_config.recipe_source
    else {
        return rsx! {};
    };
    let initial_quality = match ctx.recipe_config.quality_source {
        QualitySource::Value(value) => Some(value),
        QualitySource::HqMaterialList(_) => None,
    };
    drop(ctx);

    let use_base_overrides = overrides.base_progress_override.is_some();
    let rlvl = RLVLS[recipe.recipe_level as usize];
    let recipe_job_level = rlvl.job_level;

    // Applies a modification to the custom recipe data and overrides.
    let mut modify = move |f: &dyn Fn(&mut Recipe, &mut raphael_data::CustomRecipeOverrides)| {
        if let RecipeSource::Custom { data, overrides } =
            &mut app_context.write().recipe_config.recipe_source
        {
            f(data, overrides);
        }
    };

    rsx! {
        Warning {
            text: t_format!(
                locale,
                "⚠ Patch {ffxiv_patch} recipes and items are already included. Only use custom recipes if you are an advanced user or if new recipes haven't been added yet.",
                ffxiv_patch = "7.55"
            ),
        }
        div { class: "custom-recipe",
            div { class: "form-grid",
                label { {t!(locale, "Level:")} }
                NumberInput {
                    value: i64::from(recipe_job_level),
                    min: 1,
                    max: 100,
                    disabled: !use_base_overrides,
                    onchange: move |level: i64| modify(&|recipe, _| {
                        recipe.recipe_level = raphael_data::LEVEL_ADJUST_TABLE[level as usize];
                    }),
                }
                label { {t!(locale, "Recipe Level:")} }
                NumberInput {
                    value: i64::from(recipe.recipe_level),
                    min: 1,
                    max: RLVLS.len() as i64 - 1,
                    disabled: use_base_overrides,
                    suffix: if use_base_overrides && recipe_job_level >= 50 { "+" } else { "" },
                    onchange: move |value: i64| modify(&|recipe, _| recipe.recipe_level = value as u16),
                }
                label { {t!(locale, "Progress:")} }
                NumberInput {
                    value: i64::from(overrides.max_progress_override),
                    onchange: move |value: i64| modify(&|_, overrides| overrides.max_progress_override = value as u16),
                }
                label { {t!(locale, "Quality:")} }
                NumberInput {
                    value: i64::from(overrides.max_quality_override),
                    onchange: move |value: i64| modify(&|_, overrides| overrides.max_quality_override = value as u16),
                }
                if let Some(initial_quality) = initial_quality {
                    label { {t!(locale, "Initial Quality:")} }
                    NumberInput {
                        value: i64::from(initial_quality),
                        onchange: move |value: i64| {
                            app_context.write().recipe_config.quality_source = QualitySource::Value(
                                value as u16,
                            );
                        },
                    }
                }
                label { {t!(locale, "Durability:")} }
                NumberInput {
                    value: i64::from(overrides.max_durability_override),
                    min: 10,
                    max: 100,
                    onchange: move |value: i64| modify(&|_, overrides| overrides.max_durability_override = value as u16),
                }
                div { class: "span-2",
                    Checkbox {
                        checked: recipe.is_expert,
                        label: t!(locale, "Expert recipe"),
                        onchange: move |checked: bool| modify(&|recipe, _| recipe.is_expert = checked),
                    }
                }
            }
            div { class: "form-grid",
                label { {t!(locale, "Progress divider")} }
                NumberInput { value: i64::from(rlvl.progress_div), disabled: true }
                label { {t!(locale, "Quality divider")} }
                NumberInput { value: i64::from(rlvl.quality_div), disabled: true }
                label { {t!(locale, "Progress modifier")} }
                NumberInput { value: i64::from(rlvl.progress_mod), disabled: true }
                label { {t!(locale, "Quality modifier")} }
                NumberInput { value: i64::from(rlvl.quality_mod), disabled: true }
                label { {t!(locale, "Progress per 100% efficiency:")} }
                NumberInput {
                    value: i64::from(overrides.base_progress_override.unwrap_or(default_game_settings.base_progress)),
                    max: 9999,
                    disabled: !use_base_overrides,
                    onchange: move |value: i64| modify(&|_, overrides| overrides.base_progress_override = Some(value as u16)),
                }
                label { {t!(locale, "Quality per 100% efficiency:")} }
                NumberInput {
                    value: i64::from(overrides.base_quality_override.unwrap_or(default_game_settings.base_quality)),
                    max: 9999,
                    disabled: !use_base_overrides,
                    onchange: move |value: i64| modify(&|_, overrides| overrides.base_quality_override = Some(value as u16)),
                }
                div { class: "span-2",
                    Checkbox {
                        checked: use_base_overrides,
                        label: t!(locale, "Override per 100% efficiency values"),
                        onchange: move |checked: bool| modify(&|_, overrides| {
                            if checked {
                                overrides.base_progress_override = Some(default_game_settings.base_progress);
                                overrides.base_quality_override = Some(default_game_settings.base_quality);
                            } else {
                                overrides.base_progress_override = None;
                                overrides.base_quality_override = None;
                            }
                        }),
                    }
                }
            }
        }
    }
}

#[component]
fn RecipeFilterDialog() -> Element {
    let mut app_context = use_app_context();
    let mut dialogs = use_dialogs();
    if !dialogs.read().recipe_filters {
        return rsx! {};
    }
    let (locale, filters) = {
        let ctx = app_context.read();
        (ctx.locale, ctx.search_state.recipe.filters.clone())
    };
    let zones = [
        CosmicExplorationZone::SinusArdorum,
        CosmicExplorationZone::Phaenna,
        CosmicExplorationZone::Oizys,
        CosmicExplorationZone::Auxesia,
    ];
    let close = move |_| dialogs.write().recipe_filters = false;

    rsx! {
        Modal {
            title: t!(locale, "Recipe filters"),
            size: "sm",
            onclose: close,
            footer: rsx! {
                button { class: "btn primary block", onclick: move |_| dialogs.write().recipe_filters = false,
                    {t!(locale, "Close")}
                }
            },
            Checkbox {
                checked: filters.active_job_only,
                label: t!(locale, "Active job only"),
                onchange: move |checked: bool| app_context.write().search_state.recipe.filters.active_job_only = checked,
            }
            h3 { class: "section-title", {t!(locale, "Stellar missions")} }
            div { class: "chips",
                for zone in zones {
                    button {
                        key: "{zone:?}",
                        class: if filters.cosmic_exploration_zone.is_none_or(|selected| selected == zone) { "chip active" } else { "chip" },
                        onclick: move |_| {
                            let mut ctx = app_context.write();
                            let selected = &mut ctx.search_state.recipe.filters.cosmic_exploration_zone;
                            *selected = if *selected == Some(zone) { None } else { Some(zone) };
                        },
                        {get_cosmic_exploration_zone_name(zone, locale)}
                    }
                }
            }
        }
    }
}
