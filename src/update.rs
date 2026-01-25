use std::error::Error;
use std::sync::{LazyLock, Mutex};

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

pub fn show_dialogues(ctx: &egui::Context, locale: Locale) {
    show_update_prompt(ctx, locale);
    show_error_message(ctx, locale);
}

fn show_update_prompt(ctx: &egui::Context, locale: Locale) {
    let mut latest_version = match LATEST_VERSION.try_lock() {
        Ok(mutex_guard) => mutex_guard,
        Err(_) => return,
    };
    if CURRENT_VERSION.ge(&latest_version) {
        return;
    }
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
                let result = update_and_close_application();
                if let Err(error) = result {
                    log::error!("Error while downloading update: {:?}", &error);
                    ctx.data_mut(|mem| {
                        // The error cannot be stored because it does not implement Clone.
                        // Therefore we need to stringify it.
                        let error_message = format!("{:?}", error);
                        mem.insert_temp(egui::Id::new("UPDATE_ERROR"), error_message);
                    });
                }
                // Reset the latest version to stop this dialogue from being shown.
                *latest_version = semver::Version::new(0, 0, 0);
            }
            if ui.button(t!(locale, "Close")).clicked() {
                // Reset the latest version to stop this dialogue from being shown.
                *latest_version = semver::Version::new(0, 0, 0);
            }
        });
    });
}

fn show_error_message(ctx: &egui::Context, locale: Locale) {
    let Some(error) = ctx.data(|mem| mem.get_temp::<String>(egui::Id::new("UPDATE_ERROR"))) else {
        return;
    };
    egui::Modal::new(egui::Id::new("UPDATE_ERROR_DIALOGUE")).show(ctx, |ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 3.0);
        ui.label(egui::RichText::new(t!(locale, "Error")).strong());
        ui.separator();
        ui.monospace(error);
        ui.separator();
        ui.vertical_centered_justified(|ui| {
            if ui.button(t!(locale, "Close")).clicked() {
                ctx.data_mut(|mem| mem.remove_temp::<String>(egui::Id::new("UPDATE_ERROR")));
            }
        });
    });
}

fn update_and_close_application() -> self_update::errors::Result<()> {
    self_update::backends::github::Update::configure()
        .repo_owner("KonaeAkira")
        .repo_name("raphael-rs")
        .bin_name("raphael-xiv")
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_output(false)
        .no_confirm(true)
        .build()?
        .update()?;
    std::process::exit(0);
}
