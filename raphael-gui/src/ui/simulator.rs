use dioxus::prelude::*;
use raphael_sim::SimulationState;
use raphael_translations::{t, t_format};

use super::{
    use_app_context, use_solve_state,
    widgets::{ActionIcon, HelpTip, strip_icon},
};
use crate::config::QualityTarget;

#[component]
fn StatBar(label: &'static str, kind: &'static str, value: u32, maximum: u32) -> Element {
    let locale = use_app_context().read().locale;
    let percentage = match maximum {
        0 => 0.0,
        _ => (f64::from(value) / f64::from(maximum) * 100.0).min(100.0),
    };
    let text = if value > maximum {
        let overflow = value - maximum;
        t_format!(locale, "{value} / {maximum}  (+{overflow} overflow)")
    } else {
        format!("{value} / {maximum}")
    };
    rsx! {
        div { class: "stat-label", "{label}" }
        div {
            class: "bar bar-{kind}",
            role: "progressbar",
            "aria-valuenow": value,
            "aria-valuemax": maximum,
            div { class: "bar-fill", style: "width: {percentage}%" }
        }
        span { class: "bar-text", "{text}" }
    }
}

#[component]
pub fn SimulatorPanel() -> Element {
    let app_context = use_app_context();
    let solve_state = use_solve_state();
    let ctx = app_context.read();
    let solve = solve_state.read();
    let locale = ctx.locale;

    let settings = ctx.game_settings();
    let initial_quality = ctx.initial_quality();
    let job_id = ctx.crafter_config.selected_job;
    let item_always_collectable = raphael_data::ITEMS
        .get(ctx.recipe_config.recipe().item_id)
        .map(|item| item.always_collectable)
        .unwrap_or_default();
    let config_changed = solve.config_changed(&ctx);

    let (state, errors) = SimulationState::from_macro_continue_on_error(&settings, &solve.actions);
    let quality = initial_quality.saturating_add(state.quality);

    let outcome = if !state.is_final(&settings) {
        None
    } else if state.progress < settings.max_progress {
        Some(("outcome-fail", t!(locale, "Synthesis failed").to_string()))
    } else if item_always_collectable {
        let tier = match quality {
            q if q >= QualityTarget::CollectableT3.get_target(settings.max_quality) => 3,
            q if q >= QualityTarget::CollectableT2.get_target(settings.max_quality) => 2,
            q if q >= QualityTarget::CollectableT1.get_target(settings.max_quality) => 1,
            _ => 0,
        };
        Some(("outcome-ok", t_format!(locale, "Tier {tier} collectable")))
    } else {
        let hq = raphael_data::hq_percentage(quality, settings.max_quality).unwrap_or(0);
        Some(("outcome-ok", t_format!(locale, "{hq}% HQ")))
    };
    let assumption = if settings.adversarial {
        t!(
            locale,
            "Calculated assuming worst possible sequence of conditions"
        )
    } else {
        t!(locale, "Calculated assuming Normal conditon on every step")
    };

    rsx! {
        section { class: "card simulator",
            div { class: "card-header",
                h2 { {t!(locale, "Simulation")} }
                if config_changed {
                    span { class: "badge warn",
                        {strip_icon(t!(locale, "⚠ Some parameters have changed since last solve."))}
                    }
                }
                div { class: "spacer" }
                if let Some((class, text)) = outcome {
                    span { class: "badge {class}", "{text}" }
                }
                HelpTip { text: assumption }
            }
            div { class: "sim-body",
                div { class: "stat-bars",
                    StatBar {
                        label: t!(locale, "Progress"),
                        kind: "progress",
                        value: u32::from(state.progress),
                        maximum: u32::from(settings.max_progress),
                    }
                    StatBar {
                        label: t!(locale, "Quality"),
                        kind: "quality",
                        value: u32::from(quality),
                        maximum: u32::from(settings.max_quality),
                    }
                    StatBar {
                        label: t!(locale, "Durability"),
                        kind: "durability",
                        value: u32::from(state.durability),
                        maximum: u32::from(settings.max_durability),
                    }
                    StatBar {
                        label: t!(locale, "CP"),
                        kind: "cp",
                        value: u32::from(state.cp),
                        maximum: u32::from(settings.max_cp),
                    }
                }
                div { class: "action-strip",
                    for (index , (action , error)) in solve.actions.iter().zip(errors.iter()).enumerate() {
                        ActionIcon {
                            key: "{index}",
                            action: *action,
                            job_id,
                            locale,
                            failed: error.is_err(),
                            step: index + 1,
                        }
                    }
                }
            }
        }
    }
}
