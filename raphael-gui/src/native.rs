//! Native-only functionality: serving bundled assets to the webview and self-updating.

use std::error::Error;
use std::io::Write;
use std::sync::{LazyLock, Mutex};

use dioxus::desktop::{use_asset_handler, wry::http::Response};
use dioxus::prelude::*;
use include_dir::{Dir, include_dir};
use raphael_translations::t;

use crate::ui::use_app_context;

static ACTION_ICONS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../assets/action-icons");
static ICON_FONT: &[u8] =
    include_bytes!("../../assets/fonts/XIV_Icon_Recreations/XIV_Icon_Recreations.ttf");

/// Decodes `%XX` escape sequences in a URL path.
fn percent_decode(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && let Some(hex) = path.get(index + 1..index + 3)
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            decoded.push(byte);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

fn respond(content_type: &str, body: Option<&'static [u8]>) -> Response<Vec<u8>> {
    match body {
        Some(body) => Response::builder()
            .header("Content-Type", content_type)
            .body(body.to_vec())
            .unwrap(),
        None => Response::builder().status(404).body(Vec::new()).unwrap(),
    }
}

/// Serves the files that the web version loads from the server.
pub fn use_native_asset_handlers() {
    use_asset_handler("action-icons", |request, responder| {
        let path = percent_decode(request.uri().path());
        let file = path
            .trim_start_matches('/')
            .strip_prefix("action-icons/")
            .and_then(|relative_path| ACTION_ICONS.get_file(relative_path));
        responder.respond(respond("image/webp", file.map(include_dir::File::contents)));
    });
    use_asset_handler("fonts", |request, responder| {
        let body = request
            .uri()
            .path()
            .ends_with("XIV_Icon_Recreations.ttf")
            .then_some(ICON_FONT);
        responder.respond(respond("font/ttf", body));
    });
}

const GH_RELEASE_API: &str = "https://api.github.com/repos/KonaeAkira/raphael-rs/releases/latest";
const GH_RELEASE_URL: &str = "https://github.com/KonaeAkira/raphael-rs/releases/latest";

static CURRENT_VERSION: LazyLock<semver::Version> =
    LazyLock::new(|| semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap());
static UPDATE_STATUS: LazyLock<Mutex<UpdateStatus>> = LazyLock::new(Mutex::default);

#[derive(Debug, Clone, Default, PartialEq)]
enum UpdateStatus {
    #[default]
    None,
    Available {
        latest_version: semver::Version,
        asset_url: Option<String>,
    },
    Updating,
    Error(String),
    Success,
}

fn set_update_status(status: UpdateStatus) {
    *UPDATE_STATUS.lock().unwrap() = status;
}

pub fn check_for_update() {
    #[derive(serde::Deserialize)]
    struct Asset {
        name: String,
        browser_download_url: String,
    }
    #[derive(serde::Deserialize)]
    struct ApiResponse {
        tag_name: String,
        assets: Vec<Asset>,
    }
    let process_response =
        |fetch_result: ehttp::Result<ehttp::Response>| -> Result<(), Box<dyn Error>> {
            let response = fetch_result?;
            if !response.ok {
                return Err(format!("{} {}", response.status, response.status_text).into());
            }
            let parsed_response = response.json::<ApiResponse>()?;
            let latest_version =
                semver::Version::parse(parsed_response.tag_name.trim_start_matches('v'))?;
            if CURRENT_VERSION.ge(&latest_version) {
                log::info!("Already up-to-date. Latest version: {latest_version}.");
                return Ok(());
            }
            let asset_url = parsed_response
                .assets
                .into_iter()
                .find(|asset| is_compatible_executable(&asset.name))
                .map(|asset| asset.browser_download_url);
            set_update_status(UpdateStatus::Available {
                latest_version,
                asset_url,
            });
            Ok(())
        };
    ehttp::fetch(ehttp::Request::get(GH_RELEASE_API), move |result| {
        if let Err(error) = process_response(result) {
            log::error!("Error when fetching latest version: {error}");
        }
    });
}

fn is_compatible_executable(asset_name: &str) -> bool {
    use std::env::consts::{ARCH, OS};
    asset_name.starts_with("raphael") && asset_name.contains(ARCH) && asset_name.contains(OS)
}

fn download_and_replace_executable(asset_url: &str) {
    let process_response =
        |fetch_result: ehttp::Result<ehttp::Response>| -> Result<(), Box<dyn Error>> {
            let response = fetch_result?;
            if !response.ok {
                return Err(format!("{} {}", response.status, response.status_text).into());
            }
            let mut temp_exe = tempfile::NamedTempFile::new()?;
            temp_exe.write_all(&response.bytes)?;
            self_replace::self_replace(&temp_exe)?;
            set_update_status(UpdateStatus::Success);
            Ok(())
        };
    ehttp::fetch(ehttp::Request::get(asset_url), move |result| {
        if let Err(error) = process_response(result) {
            let error_message = format!("{error:?}");
            log::error!("Error when updating to latest version: {error_message}");
            set_update_status(UpdateStatus::Error(error_message));
        }
    });
}

#[component]
pub fn UpdateDialogs() -> Element {
    let locale = use_app_context().read().locale;
    // The update status is changed from background threads, so it is polled.
    let mut status = use_signal(UpdateStatus::default);
    use_future(move || async move {
        loop {
            let current = UPDATE_STATUS.lock().unwrap().clone();
            if *status.peek() != current {
                status.set(current);
            }
            crate::platform::sleep(web_time::Duration::from_millis(250)).await;
        }
    });
    let close = move || set_update_status(UpdateStatus::None);

    match status() {
        UpdateStatus::None => rsx! {},
        UpdateStatus::Available {
            latest_version,
            asset_url,
        } => rsx! {
            crate::ui::Modal {
                title: t!(locale, "New version available!"),
                size: "sm",
                onclose: move |_| close(),
                footer: rsx! {
                    if let Some(asset_url) = asset_url {
                        button {
                            class: "btn primary",
                            onclick: move |_| {
                                set_update_status(UpdateStatus::Updating);
                                download_and_replace_executable(&asset_url);
                            },
                            {t!(locale, "Update")}
                        }
                    }
                    button { class: "btn", onclick: move |_| close(), {t!(locale, "Close")} }
                },
                p {
                    "v{*CURRENT_VERSION} → v{latest_version} "
                    a { href: GH_RELEASE_URL, target: "_blank", {t!(locale, "(view on GitHub)")} }
                }
            }
        },
        UpdateStatus::Updating => rsx! {
            crate::ui::Modal {
                title: t!(locale, "Updating..."),
                size: "xs",
                closable: false,
                onclose: |_| {},
                div { class: "solving",
                    div { class: "spinner" }
                }
            }
        },
        UpdateStatus::Error(message) => rsx! {
            crate::ui::Modal {
                title: t!(locale, "Error"),
                size: "sm",
                onclose: move |_| close(),
                footer: rsx! {
                    button { class: "btn primary block", onclick: move |_| close(), {t!(locale, "Close")} }
                },
                pre { class: "error-text", "{message}" }
            }
        },
        UpdateStatus::Success => rsx! {
            crate::ui::Modal {
                title: t!(locale, "Success"),
                size: "sm",
                onclose: move |_| close(),
                footer: rsx! {
                    button {
                        class: "btn primary block",
                        // Close the application so that the user reopens the updated version.
                        onclick: move |_| dioxus::desktop::window().close(),
                        {t!(locale, "Close")}
                    }
                },
                p { {t!(locale, "Reopen the application for the updated version.")} }
            }
        },
    }
}
