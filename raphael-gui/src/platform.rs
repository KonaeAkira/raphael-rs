//! Small platform abstractions that differ between the web and native builds.

pub const IS_WEB: bool = cfg!(target_arch = "wasm32");

#[cfg(target_arch = "wasm32")]
pub async fn sleep(duration: web_time::Duration) {
    gloo_timers::future::sleep(duration).await;
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn sleep(duration: web_time::Duration) {
    tokio::time::sleep(duration).await;
}

/// Runs `callback` right before the app closes: on page hide on the web, on window close on
/// native. The callback must be synchronous because the app may exit as soon as it returns.
#[cfg(target_arch = "wasm32")]
pub fn use_on_exit(callback: impl FnMut() + 'static) {
    use dioxus::prelude::*;
    use std::rc::Rc;
    use wasm_bindgen::{JsCast, closure::Closure};
    // The closure is kept in the hook state so the browser can still call it for as long as the
    // component lives.
    use_hook(move || {
        let closure = Rc::new(Closure::<dyn FnMut()>::new(callback));
        if let Some(window) = web_sys::window() {
            window.set_onpagehide(Some(closure.as_ref().as_ref().unchecked_ref()));
        }
        closure
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn use_on_exit(mut callback: impl FnMut() + 'static) {
    use dioxus::desktop::{WindowEvent, tao::event::Event, use_wry_event_handler};
    use_wry_event_handler(move |event, _| {
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            callback();
        }
    });
}

#[cfg(target_arch = "wasm32")]
pub fn copy_to_clipboard(text: String) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let promise = window.navigator().clipboard().write_text(&text);
    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = wasm_bindgen_futures::JsFuture::from(promise).await {
            log::error!("Failed to copy to clipboard: {error:?}");
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn copy_to_clipboard(text: String) {
    use std::sync::{LazyLock, Mutex};
    // The clipboard is kept alive for the duration of the program because some platforms
    // (e.g. X11) only serve the clipboard contents for as long as the owner exists.
    static CLIPBOARD: LazyLock<Mutex<Option<arboard::Clipboard>>> =
        LazyLock::new(|| Mutex::new(arboard::Clipboard::new().ok()));
    match CLIPBOARD.lock().unwrap().as_mut() {
        Some(clipboard) => {
            if let Err(error) = clipboard.set_text(text) {
                log::error!("Failed to copy to clipboard: {error}");
            }
        }
        None => log::error!("Clipboard is unavailable"),
    }
}

/// Line separator used for macro text. FFXIV on Windows expects CRLF.
pub fn macro_newline() -> &'static str {
    if is_mac() { "\n" } else { "\r\n" }
}

#[cfg(target_arch = "wasm32")]
fn is_mac() -> bool {
    web_sys::window()
        .and_then(|window| window.navigator().user_agent().ok())
        .is_some_and(|user_agent| user_agent.contains("Mac"))
}

#[cfg(not(target_arch = "wasm32"))]
fn is_mac() -> bool {
    cfg!(target_os = "macos")
}
