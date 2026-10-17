use std::num::NonZeroUsize;

use dioxus::prelude::*;
use raphael_solver::SolverException;
use raphael_translations::{t, t_format};

use super::{
    saved_rotations::SavedRotationsDialog,
    stats_edit::StatsEditDialog,
    use_app_context, use_dialogs, use_solve_state,
    widgets::{Checkbox, Modal, NumberInput, Segmented, Warning, format_thousands, strip_icon},
};
use crate::{
    config::{LayoutPreference, ThemePreference},
    context::AppContext,
    platform,
    solve::SolveStatus,
    thread_pool,
};

#[component]
pub fn Dialogs() -> Element {
    let dialogs = use_dialogs()();
    rsx! {
        if dialogs.settings {
            SettingsDialog {}
        }
        if dialogs.stats_edit {
            StatsEditDialog {}
        }
        if dialogs.saved_rotations {
            SavedRotationsDialog {}
        }
        if dialogs.missing_stats {
            MissingStatsDialog {}
        }
        SolverDialogs {}
        if !platform::IS_WEB {
            {native_dialogs()}
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn native_dialogs() -> Element {
    rsx! {
        crate::native::UpdateDialogs {}
    }
}

#[cfg(target_arch = "wasm32")]
fn native_dialogs() -> Element {
    rsx! {}
}

#[component]
fn SettingsDialog() -> Element {
    let mut app_context = use_app_context();
    let mut dialogs = use_dialogs();
    let (locale, app_config) = {
        let ctx = app_context.read();
        (ctx.locale, ctx.app_config)
    };
    let themes = [
        ThemePreference::System,
        ThemePreference::Light,
        ThemePreference::Dark,
    ];
    let layouts = [LayoutPreference::Default, LayoutPreference::Compact];
    let pool_attempted = thread_pool::initialization_attempted();
    let shown_thread_count = if pool_attempted {
        rayon::current_num_threads()
    } else {
        app_config
            .num_threads
            .unwrap_or_else(thread_pool::default_thread_count)
            .get()
    };

    rsx! {
        Modal {
            title: strip_icon(t!(locale, "⚙ Settings")),
            size: "sm",
            onclose: move |_| dialogs.write().settings = false,
            div { class: "settings-grid",
                label { {t!(locale, "Theme")} }
                Segmented {
                    selected: themes.iter().position(|theme| *theme == app_config.theme).unwrap_or_default(),
                    options: vec![
                        t!(locale, "System").to_string(),
                        t!(locale, "Light").to_string(),
                        t!(locale, "Dark").to_string(),
                    ],
                    onchange: move |index: usize| app_context.write().app_config.theme = themes[index],
                }

                label { {t!(locale, "Layout")} }
                Segmented {
                    selected: layouts.iter().position(|layout| *layout == app_config.layout).unwrap_or_default(),
                    options: vec![
                        t!(locale, "Default").to_string(),
                        t!(locale, "Compact").to_string(),
                    ],
                    onchange: move |index: usize| app_context.write().app_config.layout = layouts[index],
                }

                if !platform::IS_WEB {
                    {zoom_setting(locale, app_context)}
                }

                label { {t!(locale, "Max solver threads")} }
                div { class: "inline-row",
                    Checkbox {
                        checked: app_config.num_threads.is_none(),
                        disabled: pool_attempted,
                        label: t!(locale, "Auto"),
                        onchange: move |auto: bool| {
                            app_context.write().app_config.num_threads = if auto {
                                None
                            } else {
                                Some(thread_pool::default_thread_count())
                            };
                        },
                    }
                    NumberInput {
                        value: shown_thread_count as i64,
                        min: 1,
                        max: 256,
                        disabled: pool_attempted || app_config.num_threads.is_none(),
                        onchange: move |count: i64| {
                            app_context.write().app_config.num_threads = NonZeroUsize::new(count as usize);
                        },
                    }
                }
            }
            if pool_attempted {
                Warning { text: t!(locale, "⚠ Unavailable after the solver was started."),
                    p {
                        if platform::IS_WEB {
                            {t!(locale, "Reload the page to change max solver threads.")}
                        } else {
                            {t!(locale, "Restart the app to change max solver threads.")}
                        }
                    }
                }
            }
        }
    }
}

/// Zoom level of the webview. The web version relies on the browser's zoom instead.
#[cfg(not(target_arch = "wasm32"))]
fn zoom_setting(locale: raphael_data::Locale, mut app_context: Signal<AppContext>) -> Element {
    use crate::config::DEFAULT_ZOOM_PERCENTAGE;
    const MIN_ZOOM: i64 = 50;
    const MAX_ZOOM: i64 = 500;
    const ZOOM_STEP: i64 = 10;

    let zoom = i64::from(app_context.read().app_config.webview_zoom_percentage);
    let mut set_zoom = move |zoom: i64| {
        app_context.write().app_config.webview_zoom_percentage =
            zoom.clamp(MIN_ZOOM, MAX_ZOOM) as u16;
    };
    rsx! {
        label { {t!(locale, "Zoom")} }
        div { class: "inline-row",
            button {
                class: "btn icon-btn mono",
                disabled: zoom <= MIN_ZOOM,
                onclick: move |_| set_zoom(zoom - ZOOM_STEP),
                "−"
            }
            button {
                class: "btn",
                disabled: zoom == i64::from(DEFAULT_ZOOM_PERCENTAGE),
                onclick: move |_| set_zoom(i64::from(DEFAULT_ZOOM_PERCENTAGE)),
                {t!(locale, "Reset")}
            }
            button {
                class: "btn icon-btn mono",
                disabled: zoom >= MAX_ZOOM,
                onclick: move |_| set_zoom(zoom + ZOOM_STEP),
                "+"
            }
            NumberInput {
                value: zoom,
                min: MIN_ZOOM,
                max: MAX_ZOOM,
                lazy: true,
                suffix: "%",
                onchange: move |zoom: i64| set_zoom(zoom),
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn zoom_setting(_locale: raphael_data::Locale, _app_context: Signal<AppContext>) -> Element {
    rsx! {}
}

#[component]
fn MissingStatsDialog() -> Element {
    let app_context = use_app_context();
    let mut dialogs = use_dialogs();
    let ctx = app_context.read();
    let locale = ctx.locale;
    let req_cms = ctx.recipe_config.recipe().req_craftsmanship;
    let req_ctrl = ctx.recipe_config.recipe().req_control;
    drop(ctx);
    let mut close = move || dialogs.write().missing_stats = false;

    rsx! {
        Modal {
            title: t!(locale, "Error"),
            size: "sm",
            onclose: move |_| close(),
            footer: rsx! {
                button { class: "btn primary block", onclick: move |_| close(), {t!(locale, "Close")} }
            },
            p { {t!(locale, "Your stats are below the minimum requirement for this recipe.")} }
            p { {t_format!(locale, "Requirement: {req_cms} Craftsmanship, {req_ctrl} Control.")} }
        }
    }
}

#[component]
fn SolverDialogs() -> Element {
    let app_context = use_app_context();
    let mut solve_state = use_solve_state();
    let locale = app_context.read().locale;
    let state = solve_state.read();

    match &state.status {
        SolveStatus::Idle => rsx! {},
        SolveStatus::Starting | SolveStatus::Solving { .. } => rsx! {
            SolvingDialog {}
        },
        SolveStatus::Failed { error } => {
            let error = error.clone();
            drop(state);
            let mut close = move || solve_state.write().resolve_error();
            let (title, body) = match error {
                SolverException::NoSolution => {
                    let ctx = app_context.read();
                    let suggest_lower_target = ctx.solver_config.must_reach_target_quality
                        && ctx.game_settings().max_quality != 0;
                    (
                        t!(locale, "No solution"),
                        rsx! {
                            p { {t!(locale, "Cannot complete synthesis.")} }
                            if suggest_lower_target {
                                p { {t!(locale, "Try lowering target quality.")} }
                            }
                        },
                    )
                }
                SolverException::Interrupted => (t!(locale, "Error"), rsx! {}),
                SolverException::SearchQueueCapacityExceeded => (
                    t!(locale, "Search Queue Capacity Exceeded"),
                    rsx! {
                        p { {t!(locale, "The number of nodes in the search queue exceeded the limit of the 32-bit web version.")} }
                        p { {t!(locale, "Solving this configuration requires a 64-bit version of Raphael.")} }
                        a {
                            href: "https://github.com/KonaeAkira/raphael-rs/releases/latest",
                            target: "_blank",
                            rel: "noopener",
                            {t!(locale, "Download latest release from GitHub")}
                        }
                    },
                ),
                SolverException::InternalError(message) => (
                    t!(locale, "Internal Solver Error"),
                    rsx! {
                        pre { class: "error-text", "{message}" }
                    },
                ),
            };
            rsx! {
                Modal {
                    title,
                    onclose: move |_| close(),
                    footer: rsx! {
                        button { class: "btn primary block", onclick: move |_| close(), {t!(locale, "Close")} }
                    },
                    {body}
                }
            }
        }
    }
}

#[component]
fn SolvingDialog() -> Element {
    let app_context = use_app_context();
    let solve_state = use_solve_state();
    // Re-render periodically to update the elapsed time.
    let mut tick = use_signal(|| 0u32);
    use_future(move || async move {
        loop {
            platform::sleep(web_time::Duration::from_millis(33)).await;
            tick += 1;
        }
    });
    let _ = tick();

    let locale = app_context.read().locale;
    let state = solve_state.read();
    let interrupted = state.interrupted();
    let (elapsed, progress) = match state.running_solve_info() {
        Some(info) => (
            Some(info.start_time.elapsed().as_secs_f32()),
            info.solver_progress,
        ),
        None => (None, 0),
    };
    let can_cancel = elapsed.is_some() && !interrupted;
    drop(state);

    rsx! {
        Modal {
            title: if interrupted { t!(locale, "Cancelling ...") } else { t!(locale, "Solving ...") },
            size: "xs",
            closable: false,
            onclose: |_| {},
            footer: rsx! {
                button {
                    class: "btn block",
                    disabled: !can_cancel,
                    onclick: move |_| solve_state.read().interrupt(),
                    {t!(locale, "Cancel")}
                }
            },
            div { class: "solving",
                div { class: "spinner" }
                div {
                    if let Some(elapsed) = elapsed {
                        div { class: "solving-time", "{elapsed:.2}s" }
                    }
                    div { class: "muted",
                        if progress == 0 {
                            {t!(locale, "Computing ...")}
                        } else {
                            {
                                let num = format_thousands(progress);
                                t_format!(locale, "{num} nodes visited")
                            }
                        }
                    }
                }
            }
        }
    }
}
