use std::env::consts::{ARCH, OS};
use std::error::Error;
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
    Downloading,
    Error(String),
    Complete,
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
                if asset.name.contains(OS) && asset.name.contains(ARCH) {
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
    let mut update_status = UPDATE_STATUS.lock().unwrap();
    match update_status.clone() {
        UpdateStatus::Available { latest_version, .. } => {
            show_update_prompt(ctx, locale, &mut update_status, latest_version);
        }
        UpdateStatus::Error(error_message) => {
            show_error_message(ctx, locale, &mut update_status, error_message);
        }
        _ => (),
    };
}

fn show_update_prompt(
    ctx: &egui::Context,
    locale: Locale,
    update_status: &mut UpdateStatus,
    latest_version: semver::Version,
) {
    egui::Modal::new(egui::Id::new("UPDATE_DIALOGUE")).show(ctx, |ui| {
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
                let result = update_and_close_application(ctx);
                if let Err(error) = result {
                    log::error!("Error while downloading update: {:?}", &error);
                    *update_status = UpdateStatus::Error(format!("{:?}", error));
                } else {
                    *update_status = UpdateStatus::Complete;
                }
            }
            if ui.button(t!(locale, "Close")).clicked() {
                *update_status = UpdateStatus::None;
            }
        });
    });
}

fn show_error_message(
    ctx: &egui::Context,
    locale: Locale,
    update_status: &mut UpdateStatus,
    error: String,
) {
    egui::Modal::new(egui::Id::new("UPDATE_ERROR_DIALOGUE")).show(ctx, |ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
        ui.label(egui::RichText::new(t!(locale, "Error")).strong());
        ui.separator();
        ui.monospace(format!("{:?}", error));
        ui.separator();
        ui.vertical_centered_justified(|ui| {
            if ui.button(t!(locale, "Close")).clicked() {
                *update_status = UpdateStatus::None;
            }
        });
    });
}

fn update_and_close_application(ctx: &egui::Context) -> self_update::errors::Result<()> {
    self_update::backends::github::Update::configure()
        .repo_owner("KonaeAkira")
        .repo_name("raphael-s")
        .bin_name("raphael-xiv")
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_output(false)
        .no_confirm(true)
        .build()?
        .update()?;
    // Gracefully close the application.
    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    Ok(())
}
