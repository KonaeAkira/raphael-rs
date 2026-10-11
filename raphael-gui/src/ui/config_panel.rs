use dioxus::prelude::*;
use raphael_data::{Locale, action_name};
use raphael_sim::{Action, ActionImpl, HeartAndSoul, Manipulation, QuickInnovation};
use raphael_translations::{t, t_format};

use super::{
    icons::{Icon, IconKind},
    use_app_context, use_dialogs, use_solve_state,
    widgets::{
        Checkbox, GameDataName, HelpTip, NameSource, NumberInput, Select, Warning, job_names,
    },
};
use crate::{
    config::{QualitySource, QualityTarget},
    platform,
    solve::{self, LastSolveInfo},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stat {
    Craftsmanship,
    Control,
    Cp,
}

#[component]
fn StatRow(stat: Stat) -> Element {
    let mut app_context = use_app_context();
    let ctx = app_context.read();
    let locale = ctx.locale;
    let consumables = [ctx.selected_food, ctx.selected_potion];
    let stats = ctx.active_stats();
    let (label, base, bonus) = match stat {
        Stat::Craftsmanship => (
            t!(locale, "Craftsmanship"),
            stats.craftsmanship,
            raphael_data::craftsmanship_bonus(stats.craftsmanship, &consumables),
        ),
        Stat::Control => (
            t!(locale, "Control"),
            stats.control,
            raphael_data::control_bonus(stats.control, &consumables),
        ),
        Stat::Cp => (
            t!(locale, "CP"),
            stats.cp,
            raphael_data::cp_bonus(stats.cp, &consumables),
        ),
    };
    drop(ctx);

    let mut set_base = move |value: u16| {
        let mut ctx = app_context.write();
        let stats = ctx.active_stats_mut();
        match stat {
            Stat::Craftsmanship => stats.craftsmanship = value,
            Stat::Control => stats.control = value,
            Stat::Cp => stats.cp = value,
        }
    };

    rsx! {
        div { class: "stat-row",
            label { "{label}" }
            NumberInput {
                value: i64::from(base),
                max: 9000,
                onchange: move |value: i64| set_base(value as u16),
            }
            span { class: "arrow", "→" }
            NumberInput {
                value: i64::from(base + bonus),
                max: 9999,
                lazy: true,
                highlighted: bonus != 0,
                onchange: move |value: i64| {
                    if value == i64::from(base + bonus) {
                        return;
                    }
                    let unbuffed = {
                        let ctx = app_context.read();
                        let consumables = [ctx.selected_food, ctx.selected_potion];
                        let value = value as u16;
                        match stat {
                            Stat::Craftsmanship => raphael_data::craftsmanship_unbuffed(value, &consumables),
                            Stat::Control => raphael_data::control_unbuffed(value, &consumables),
                            Stat::Cp => raphael_data::cp_unbuffed(value, &consumables),
                        }
                    };
                    if let Some(unbuffed) = unbuffed {
                        set_base(unbuffed);
                    }
                },
            }
        }
    }
}

fn experimental_warning_text(locale: Locale) -> &'static str {
    if platform::IS_WEB {
        t!(
            locale,
            "⚠ EXPERIMENTAL FEATURE\nMay crash the solver due to reaching the 4GB memory limit of 32-bit web assembly, causing the UI to get stuck in the \"solving\" state indefinitely."
        )
    } else {
        t!(
            locale,
            "⚠ EXPERIMENTAL FEATURE\nThis option may use a lot of memory (sometimes well above 4GB) which may cause your system to run out of memory."
        )
    }
}

const RELEASES_URL: &str = "https://github.com/KonaeAkira/raphael-rs/releases/latest";

#[component]
pub fn ConfigPanel() -> Element {
    let mut app_context = use_app_context();
    let mut dialogs = use_dialogs();
    let solve_state = use_solve_state();
    let ctx = app_context.read();
    let locale = ctx.locale;
    let selected_job = ctx.crafter_config.selected_job;
    let stats = *ctx.active_stats();
    let recipe = *ctx.recipe_config.recipe();
    let hq_materials = match ctx.recipe_config.quality_source {
        QualitySource::HqMaterialList(materials) => Some(materials),
        QualitySource::Value(_) => None,
    };
    let solver_config = ctx.solver_config;
    let max_quality = ctx.game_settings().max_quality;
    let max_stellar_steady_hand_charges = ctx
        .recipe_config
        .recipe_source
        .max_stellar_steady_hand_charges();
    drop(ctx);

    let solve = solve_state.read();
    let busy = solve.is_busy();
    let last_solve_info = solve.last_solve_info.clone();
    drop(solve);

    let manipulation_available = stats.level >= Manipulation::LEVEL_REQUIREMENT;
    let heart_and_soul_available = stats.level >= HeartAndSoul::LEVEL_REQUIREMENT;
    let quick_innovation_available = stats.level >= QuickInnovation::LEVEL_REQUIREMENT;
    let specialist_enabled = (heart_and_soul_available && stats.heart_and_soul)
        || (quick_innovation_available && stats.quick_innovation);

    let quality_targets = [
        QualityTarget::Zero,
        QualityTarget::Half,
        QualityTarget::CollectableT1,
        QualityTarget::CollectableT2,
        QualityTarget::CollectableT3,
        QualityTarget::Full,
        QualityTarget::Custom(solver_config.quality_target.get_target(max_quality)),
    ];
    let selected_target = quality_targets
        .iter()
        .position(|target| {
            std::mem::discriminant(target) == std::mem::discriminant(&solver_config.quality_target)
        })
        .unwrap_or_default();

    let ingredients: Vec<_> = recipe
        .ingredients
        .into_iter()
        .enumerate()
        .filter(|(_, ingredient)| ingredient.item_id != 0)
        .collect();

    let on_solve = move |_| {
        if solve::meets_stat_requirements(&app_context.read()) {
            spawn(solve::solve(app_context, solve_state));
        } else {
            dialogs.write().missing_stats = true;
        }
    };

    rsx! {
        section { class: "card config",
            div { class: "card-header",
                h2 { {t!(locale, "Configuration")} }
                div { class: "spacer" }
                Select {
                    selected: usize::from(selected_job),
                    options: job_names(locale),
                    onchange: move |job: usize| app_context.write().crafter_config.selected_job = job as u8,
                }
                button {
                    class: "btn icon-btn",
                    title: t!(locale, "Crafter stats"),
                    onclick: move |_| dialogs.write().stats_edit = true,
                    Icon { kind: IconKind::Pencil }
                }
            }
            div { class: "card-body config-body",
                h3 { class: "section-title", {t!(locale, "Crafter stats")} }
                StatRow { stat: Stat::Craftsmanship }
                StatRow { stat: Stat::Control }
                StatRow { stat: Stat::Cp }
                div { class: "stat-row",
                    label { {t!(locale, "Job level")} }
                    NumberInput {
                        value: i64::from(stats.level),
                        min: 1,
                        max: 100,
                        onchange: move |level: i64| app_context.write().active_stats_mut().level = level as u8,
                    }
                }

                h3 { class: "section-title", {t!(locale, "HQ materials")} }
                if let Some(hq_materials) = hq_materials.filter(|_| !ingredients.is_empty()) {
                    for (index , ingredient) in ingredients {
                        div { key: "{index}", class: "stat-row ingredient-row",
                            GameDataName { source: NameSource::from(&ingredient), locale }
                            NumberInput {
                                value: i64::from(hq_materials[index]),
                                max: i64::from(ingredient.amount),
                                onchange: move |amount: i64| {
                                    if let QualitySource::HqMaterialList(materials) = &mut app_context
                                        .write()
                                        .recipe_config
                                        .quality_source
                                    {
                                        materials[index] = amount as u8;
                                    }
                                },
                            }
                            span { class: "muted", "/ {ingredient.amount}" }
                        }
                    }
                } else {
                    p { class: "muted", {t!(locale, "None")} }
                }

                h3 { class: "section-title", {t!(locale, "Actions")} }
                div { class: "check-list",
                    Checkbox {
                        checked: manipulation_available && stats.manipulation,
                        disabled: !manipulation_available,
                        label: action_name(Action::Manipulation, locale),
                        onchange: move |checked: bool| app_context.write().active_stats_mut().manipulation = checked,
                    }
                    Checkbox {
                        checked: heart_and_soul_available && stats.heart_and_soul,
                        disabled: !heart_and_soul_available,
                        label: action_name(Action::HeartAndSoul, locale),
                        onchange: move |checked: bool| app_context.write().active_stats_mut().heart_and_soul = checked,
                    }
                    Checkbox {
                        checked: quick_innovation_available && stats.quick_innovation,
                        disabled: !quick_innovation_available,
                        label: action_name(Action::QuickInnovation, locale),
                        onchange: move |checked: bool| app_context.write().active_stats_mut().quick_innovation = checked,
                    }
                }
                if max_stellar_steady_hand_charges > 0 {
                    div { class: "stat-row",
                        label { {action_name(Action::StellarSteadyHand, locale)} }
                        NumberInput {
                            value: i64::from(solver_config.stellar_steady_hand_charges.min(max_stellar_steady_hand_charges)),
                            max: i64::from(max_stellar_steady_hand_charges),
                            onchange: move |charges: i64| app_context.write().solver_config.stellar_steady_hand_charges = charges as u8,
                        }
                        span { class: "muted", "/ {max_stellar_steady_hand_charges}" }
                    }
                }
                if specialist_enabled {
                    if platform::IS_WEB {
                        Warning {
                            text: t!(
                                locale,
                                "⚠ Specialist actions substantially increase solve time and memory usage. It is recommended that you download and use the native version if you want to enable specialist actions."
                            ),
                            a { href: RELEASES_URL, target: "_blank", rel: "noopener",
                                {t!(locale, "Download latest release from GitHub")}
                            }
                        }
                    } else {
                        Warning {
                            text: t!(
                                locale,
                                "⚠ Specialist actions substantially increase solve time and memory usage."
                            ),
                        }
                    }
                }

                h3 { class: "section-title", {t!(locale, "Solver settings")} }
                div { class: "stat-row",
                    label { {t!(locale, "Target quality")} }
                    Select {
                        selected: selected_target,
                        options: quality_targets.iter().map(|target| target.display(locale).to_string()).collect::<Vec<_>>(),
                        onchange: move |index: usize| app_context.write().solver_config.quality_target = quality_targets[index],
                    }
                    NumberInput {
                        value: i64::from(solver_config.quality_target.get_target(max_quality)),
                        disabled: !matches!(solver_config.quality_target, QualityTarget::Custom(_)),
                        onchange: move |value: i64| app_context.write().solver_config.quality_target = QualityTarget::Custom(value as u16),
                    }
                }
                div { class: "check-with-help",
                    Checkbox {
                        checked: solver_config.must_reach_target_quality,
                        label: t!(locale, "Solution must reach target quality"),
                        onchange: move |checked: bool| app_context.write().solver_config.must_reach_target_quality = checked,
                    }
                    HelpTip {
                        text: t!(
                            locale,
                            "Reduce memory usage by skipping candidate solutions that cannot reach the target quality. Basically, you either get a solution that reaches the target quality or you get no solution at all. If you want to know how close you are to reaching the target quality, keep this option turned off."
                        ),
                    }
                }
                div { class: "check-with-help",
                    Checkbox {
                        checked: solver_config.backload_progress,
                        label: t!(locale, "Backload progress"),
                        onchange: move |checked: bool| app_context.write().solver_config.backload_progress = checked,
                    }
                    HelpTip {
                        text: t!(
                            locale,
                            "Find a rotation that only uses Progress-increasing actions at the end of the rotation.\n  - May decrease achievable Quality.\n  - May increase macro duration."
                        ),
                    }
                }
                div { class: "check-with-help",
                    Checkbox {
                        checked: solver_config.adversarial && !recipe.is_expert,
                        disabled: recipe.is_expert,
                        label: t!(locale, "Ensure 100% reliability"),
                        onchange: move |checked: bool| app_context.write().solver_config.adversarial = checked,
                    }
                    HelpTip {
                        text: t!(
                            locale,
                            "Find a rotation that can reach the target quality no matter how unlucky the random conditions are.\n  - May decrease achievable Quality.\n  - May increase macro duration.\n  - Much longer solve time.\nThe solver never tries to use Tricks of the Trade to \"eat\" Excellent quality procs, so in some cases this option does not produce the optimal macro."
                        ),
                    }
                }
                if solver_config.adversarial && !recipe.is_expert {
                    Warning { text: experimental_warning_text(locale) }
                }
            }
            div { class: "card-footer solve-bar",
                button {
                    class: "btn primary solve-btn",
                    disabled: busy,
                    onclick: on_solve,
                    Icon { kind: IconKind::Sparkles }
                    span { {t!(locale, "Solve")} }
                }
                if let Some(LastSolveInfo { duration, loaded_from_history, .. }) = last_solve_info {
                    span { class: "muted solve-info",
                        if loaded_from_history {
                            {t!(locale, "Loaded from saved rotations")}
                        } else {
                            {t_format!(locale, "Elapsed time: {dur:.2}s", dur = duration.as_secs_f32())}
                        }
                    }
                }
            }
        }
    }
}
