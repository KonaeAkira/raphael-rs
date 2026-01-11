use std::{
    error::Error,
    sync::{LazyLock, Mutex},
};

use raphael_data::Locale;
use raphael_translations::t;

const GH_RELEASE_API: &str = "https://api.github.com/repos/KonaeAkira/raphael-rs/releases/latest";

static CURRENT_VERSION: LazyLock<semver::Version> =
    LazyLock::new(|| semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap());
static LATEST_VERSION: LazyLock<Mutex<semver::Version>> =
    LazyLock::new(|| Mutex::new(semver::Version::new(0, 0, 0)));

pub fn fetch_latest_version() {
    #[derive(serde::Deserialize)]
    struct ApiResponse {
        tag_name: String,
    }
    let process_response =
        |response: ehttp::Result<ehttp::Response>| -> Result<semver::Version, Box<dyn Error>> {
            let json = response?.json::<ApiResponse>()?;
            let version = semver::Version::parse(json.tag_name.trim_start_matches('v'))?;
            Ok(version)
        };
    ehttp::fetch(
        ehttp::Request::get(GH_RELEASE_API),
        move |result: ehttp::Result<ehttp::Response>| match process_response(result) {
            Ok(version) => *LATEST_VERSION.lock().unwrap() = version,
            Err(error) => log::error!("Error when fetching latest version: {error}"),
        },
    );
}

pub fn show_update_dialogue(ctx: &egui::Context, locale: Locale) {
    let mut latest_version = match LATEST_VERSION.try_lock() {
        Ok(mutex_guard) => mutex_guard,
        Err(_) => return,
    };
    if CURRENT_VERSION.ge(&latest_version) {
        return;
    }
    egui::Modal::new(egui::Id::new("UPDATE_DIALOGUE")).show(ctx, |ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(t!(locale, "New version available!")).strong());
            ui.label(format!("(v{})", latest_version));
        });
        ui.add(egui::Hyperlink::from_label_and_url(
            t!(locale, "Download from GitHub"),
            "https://github.com/KonaeAkira/raphael-rs/releases/latest",
        ));
        ui.separator();
        ui.vertical_centered_justified(|ui| {
            if ui.button("Close").clicked() {
                *latest_version = semver::Version::new(0, 0, 0);
            }
        });
    });
}
