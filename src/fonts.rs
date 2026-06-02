use std::sync::Arc;

use egui::{FontData, FontDefinitions, FontFamily};
use raphael_data::Locale;

fn add_font_to_definitions(
    definitions: &mut FontDefinitions,
    name: impl ToString,
    font_data: FontData,
    font_families: &[FontFamily],
) {
    definitions
        .font_data
        .insert(name.to_string(), Arc::new(font_data));

    for font_family in font_families {
        definitions
            .families
            .get_mut(font_family)
            .unwrap()
            .push(name.to_string());
    }
}

pub struct FontLoadingState {
    pub loaded_fonts_for_locale: Locale,
}

const fn additional_font_name(locale: Locale) -> Option<&'static str> {
    match locale {
        Locale::EN | Locale::DE | Locale::FR => None,
        Locale::JP => Some("NotoSansJP-Regular"),
        Locale::CN => Some("NotoSansSC-Regular"),
        Locale::KR => Some("NotoSansKR-Regular"),
        Locale::TW => Some("NotoSansTC-Regular"),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn try_load_additional_font_data(_ctx: &egui::Context, locale: Locale) -> Option<FontData> {
    match locale {
        Locale::EN | Locale::DE | Locale::FR => None,
        Locale::JP => Some(FontData::from_static(include_bytes!(
            "../assets/fonts/Noto_Sans_JP/static/NotoSansJP-Regular.ttf"
        ))),
        Locale::CN => Some(FontData::from_static(include_bytes!(
            "../assets/fonts/Noto_Sans_SC/static/NotoSansSC-Regular.ttf"
        ))),
        Locale::KR => Some(FontData::from_static(include_bytes!(
            "../assets/fonts/Noto_Sans_KR/static/NotoSansKR-Regular.ttf"
        ))),
        Locale::TW => Some(FontData::from_static(include_bytes!(
            "../assets/fonts/Noto_Sans_TC/static/NotoSansTC-Regular.ttf"
        ))),
    }
}

#[cfg(target_arch = "wasm32")]
fn try_load_additional_font_data(ctx: &egui::Context, locale: Locale) -> Option<FontData> {
    let uri = match locale {
        Locale::EN | Locale::DE | Locale::FR => return None,
        Locale::JP => concat!(
            env!("BASE_URL"),
            "/fonts/Noto_Sans_JP/static/NotoSansJP-Regular.ttf"
        ),
        Locale::CN => concat!(
            env!("BASE_URL"),
            "/fonts/Noto_Sans_SC/static/NotoSansSC-Regular.ttf"
        ),
        Locale::KR => concat!(
            env!("BASE_URL"),
            "/fonts/Noto_Sans_KR/static/NotoSansKR-Regular.ttf"
        ),
        Locale::TW => concat!(
            env!("BASE_URL"),
            "/fonts/Noto_Sans_TC/static/NotoSansTC-Regular.ttf"
        ),
    };

    if let Ok(egui::load::BytesPoll::Ready { bytes, .. }) = ctx.try_load_bytes(uri) {
        Some(egui::FontData::from_owned(bytes.to_vec()))
    } else {
        None
    }
}

impl FontLoadingState {
    pub fn new(ctx: &egui::Context, locale: Locale) -> Self {
        let mut new = Self {
            loaded_fonts_for_locale: Locale::EN,
        };
        new.load_fonts(ctx, locale, true);
        new
    }

    pub fn load_default_fonts(ctx: &egui::Context) {
        let mut fonts = FontDefinitions::default();

        add_font_to_definitions(
            &mut fonts,
            "XIV_Icon_Recreations",
            FontData::from_static(include_bytes!(
                "../assets/fonts/XIV_Icon_Recreations/XIV_Icon_Recreations.ttf"
            )),
            &[FontFamily::Proportional, FontFamily::Monospace],
        );

        ctx.set_fonts(fonts);
    }

    pub fn load_fonts(&mut self, ctx: &egui::Context, locale: Locale, initial_load: bool) {
        match additional_font_name(locale) {
            Some(name) => {
                let Some(additional_font_data) = (match locale {
                    Locale::EN | Locale::DE | Locale::FR => {
                        unreachable!();
                    }
                    Locale::JP | Locale::CN | Locale::KR | Locale::TW => {
                        try_load_additional_font_data(ctx, locale)
                    }
                }) else {
                    if initial_load {
                        Self::load_default_fonts(ctx);
                        self.loaded_fonts_for_locale = Locale::EN;
                    }
                    return;
                };
                let mut fonts = FontDefinitions::default();
                let font_families = &[FontFamily::Proportional, FontFamily::Monospace];

                add_font_to_definitions(
                    &mut fonts,
                    "XIV_Icon_Recreations",
                    FontData::from_static(include_bytes!(
                        "../assets/fonts/XIV_Icon_Recreations/XIV_Icon_Recreations.ttf"
                    )),
                    font_families,
                );

                add_font_to_definitions(&mut fonts, name, additional_font_data, font_families);

                ctx.set_fonts(fonts);
                self.loaded_fonts_for_locale = locale;
                log::debug!("Additional font loaded: {}", name);
            }
            None => {
                Self::load_default_fonts(ctx);
                self.loaded_fonts_for_locale = locale;
            }
        }
    }
}
