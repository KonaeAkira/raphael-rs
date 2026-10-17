mod config_panel;
mod consumables;
mod dialogs;
mod header;
mod icons;
mod macro_panel;
mod recipe;
mod saved_rotations;
mod simulator;
mod stats_edit;
mod virtual_list;
mod widgets;

#[cfg(not(target_arch = "wasm32"))]
pub use widgets::Modal;

use dioxus::{core::Task, prelude::*};
use raphael_data::Locale;

use crate::{
    config::ThemePreference,
    context::{AppContext, SearchText},
    platform,
    solve::SolveState,
};

const STYLE: &str = include_str!("../style.css");

/// Visibility of the app's dialogs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DialogState {
    pub settings: bool,
    pub stats_edit: bool,
    pub saved_rotations: bool,
    pub missing_stats: bool,
    pub recipe_filters: bool,
    pub macro_config: bool,
}

pub fn use_app_context() -> Signal<AppContext> {
    use_context()
}

pub fn use_search_text() -> Signal<SearchText> {
    use_context()
}

pub fn use_solve_state() -> Signal<SolveState> {
    use_context()
}

pub fn use_dialogs() -> Signal<DialogState> {
    use_context()
}

fn lang_code(locale: Locale) -> &'static str {
    match locale {
        Locale::EN => "en",
        Locale::DE => "de",
        Locale::FR => "fr",
        Locale::JP => "ja",
        Locale::CN => "zh-Hans",
        Locale::KR => "ko",
        Locale::TW => "zh-Hant",
    }
}

/// How long the app state must stay unchanged before it is written to storage.
const SAVE_DEBOUNCE: web_time::Duration = web_time::Duration::from_millis(500);

/// Persists the app state once it stops changing, and right before the app closes.
///
/// Saving serializes every stored key, including the whole solve history, so doing it on every
/// keystroke or slider tick would be too expensive.
fn use_autosave(app_context: Signal<AppContext>) {
    let mut pending_save: Signal<Option<Task>> = use_signal(|| None);
    use_effect(move || {
        // Subscribe to every change of the app state.
        let _ = &*app_context.read();
        if let Some(task) = pending_save.take() {
            task.cancel();
        }
        let task = spawn(async move {
            platform::sleep(SAVE_DEBOUNCE).await;
            // Clear the handle before saving so a later change can't cancel a finished task
            // whose id may already have been reused.
            pending_save.set(None);
            app_context.peek().save();
        });
        pending_save.set(Some(task));
    });
    // Stored values that haven't changed are skipped, so this is cheap when nothing is pending.
    platform::use_on_exit(move || app_context.peek().save());
}

#[component]
pub fn App() -> Element {
    let app_context = use_context_provider(|| Signal::new(AppContext::load()));
    use_context_provider(|| Signal::new(SearchText::default()));
    use_context_provider(|| Signal::new(SolveState::default()));
    use_context_provider(|| Signal::new(DialogState::default()));

    let mut app_context_mut = app_context;
    use_effect(move || {
        let sanitized = app_context_mut.read().sanitized_solver_config();
        if let Some(solver_config) = sanitized {
            app_context_mut.write().solver_config = solver_config;
        }
    });

    use_autosave(app_context);

    #[cfg(not(target_arch = "wasm32"))]
    crate::native::use_native_asset_handlers();

    let (locale, app_config) = {
        let ctx = app_context.read();
        (ctx.locale, ctx.app_config)
    };
    let theme = match app_config.theme {
        ThemePreference::System => "system",
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
    };

    rsx! {
        style { {STYLE} }
        div {
            class: "app",
            "data-theme": theme,
            lang: lang_code(locale),
            header::Header {}
            main { class: "main",
                simulator::SimulatorPanel {}
                div { class: "columns",
                    div { class: "col col-select",
                        recipe::RecipePanel {}
                        consumables::ConsumablePanel { kind: consumables::ConsumableKind::Food }
                        consumables::ConsumablePanel { kind: consumables::ConsumableKind::Potion }
                    }
                    div { class: "col col-config", config_panel::ConfigPanel {} }
                    div { class: "col col-macro", macro_panel::MacroPanel {} }
                }
            }
            dialogs::Dialogs {}
        }
    }
}
