use std::error::Error;
use std::io::Write;
use std::sync::{LazyLock, Mutex};

use raphael_data::Locale;
use raphael_translations::t;

const GH_RELEASE_API: &str = "https://api.github.com/repos/KonaeAkira/raphael-rs/releases/latest";

static CURRENT_VERSION: LazyLock<semver::Version> =
    LazyLock::new(|| semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap());
static UPDATE_STATUS: LazyLock<Mutex<UpdateStatus>> = LazyLock::new(Mutex::default);

#[derive(Debug, Clone, Default)]
enum UpdateStatus {
    #[default]
    None,
    Available {
        latest_version: semver::Version,
        asset_url: String,
    },
    Updating,
    Error(String),
    Success,
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
        |response: ehttp::Result<ehttp::Response>| -> Result<(), Box<dyn Error>> {
            let parsed_response = response?.json::<ApiResponse>()?;
            let latest_version =
                semver::Version::parse(parsed_response.tag_name.trim_start_matches('v'))?;
            if CURRENT_VERSION.ge(&latest_version) {
                log::info!("Already up-to-date. Latest version: {}.", &latest_version);
                return Ok(());
            }
            for asset in parsed_response.assets {
                if is_compatible_executable(&asset.name) {
                    *UPDATE_STATUS.lock().unwrap() = UpdateStatus::Available {
                        latest_version,
                        asset_url: asset.browser_download_url,
                    };
                    return Ok(());
                }
            }
            Err("Newer version exists but no compatible asset was found.".into())
        };
    ehttp::fetch(
        ehttp::Request::get(GH_RELEASE_API),
        move |result: ehttp::Result<ehttp::Response>| {
            if let Err(error) = process_response(result) {
                log::error!("Error when fetching latest version: {error}");
            }
        },
    );
}

pub fn show_dialogues(ctx: &egui::Context, locale: Locale) {
    let modal_id = egui::Id::new("UPDATE_MODAL");
    let mut update_status = UPDATE_STATUS.lock().unwrap();
    match update_status.clone() {
        UpdateStatus::None => (), // Do nothing.
        UpdateStatus::Available {
            latest_version,
            asset_url,
        } => {
            egui::Modal::new(modal_id.with("AVAILABLE")).show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
                ui.label(egui::RichText::new(t!(locale, "New version available!")).strong());
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("v{} ➡ v{} ", *CURRENT_VERSION, latest_version));
                    ui.add(egui::Hyperlink::from_label_and_url(
                        t!(locale, "(view on GitHub)"),
                        "https://github.com/KonaeAkira/raphael-rs/releases/latest",
                    ));
                });
                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    if ui.button(t!(locale, "Update")).clicked() {
                        *update_status = UpdateStatus::Updating;
                        download_and_replace_executable(&asset_url);
                    }
                    if ui.button(t!(locale, "Close")).clicked() {
                        *update_status = UpdateStatus::None;
                    }
                });
            });
        }
        UpdateStatus::Updating => {
            egui::Modal::new(modal_id.with("UPDATING")).show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(t!(locale, "Updating..."));
                });
            });
        }
        UpdateStatus::Error(error_message) => {
            egui::Modal::new(modal_id.with("ERROR")).show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
                ui.label(egui::RichText::new(t!(locale, "Error")).strong());
                ui.separator();
                ui.monospace(error_message);
                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    if ui.button(t!(locale, "Close")).clicked() {
                        *update_status = UpdateStatus::None;
                    }
                });
            });
        }
        UpdateStatus::Success => {
            egui::Modal::new(modal_id.with("SUCCESS")).show(ctx, |ui| {
                ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
                ui.label(egui::RichText::new(t!(locale, "Success")).strong());
                ui.separator();
                ui.label(t!(
                    locale,
                    "Reopen the application for the updated version."
                ));
                ui.separator();
                ui.vertical_centered_justified(|ui| {
                    if ui.button(t!(locale, "Close")).clicked() {
                        // Gracefully close the application.
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        }
    }
}

fn is_compatible_executable(asset_name: &str) -> bool {
    use std::env::consts::{ARCH, OS};
    asset_name.starts_with("raphael") && asset_name.contains(ARCH) && asset_name.contains(OS)
}

fn download_and_replace_executable(asset_url: &str) {
    let process_response =
        |response: ehttp::Result<ehttp::Response>| -> Result<(), Box<dyn Error>> {
            let bytes = response?.bytes;
            let mut temp_exe = tempfile::NamedTempFile::new()?;
            temp_exe.write_all(&bytes)?;
            self_replace::self_replace(&temp_exe)?;
            *UPDATE_STATUS.lock().unwrap() = UpdateStatus::Success;
            Ok(())
        };
    ehttp::fetch(
        ehttp::Request::get(asset_url),
        move |result: ehttp::Result<ehttp::Response>| {
            if let Err(error) = process_response(result) {
                let error_message = format!("{:?}", error);
                log::error!("Error when updating to latest version: {}", &error_message);
                *UPDATE_STATUS.lock().unwrap() = UpdateStatus::Error(error_message);
            }
        },
    );
}
