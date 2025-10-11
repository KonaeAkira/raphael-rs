use std::{
    fs::File,
    path::{Path, PathBuf},
};

pub const RAPHAEL_TRANSLATIONS_TOML_ENV: &str = "RAPHAEL_TRANSLATIONS_TOML";

pub fn translation_toml_path() -> PathBuf {
    std::env::var(RAPHAEL_TRANSLATIONS_TOML_ENV).map_or(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("translations.toml"),
        |path_string| Path::new(&path_string).to_path_buf(),
    )
}

#[cfg(feature = "update-toml")]
pub fn open_translation_toml_file() -> File {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(translation_toml_path())
        .expect("Failed to open tranlstation TOML file!");
    file.lock().unwrap();
    file
}

#[cfg(not(feature = "update-toml"))]
#[allow(dead_code)] // other implementation is used in `build.rs` when `update-toml` feature is enabled
pub fn open_translation_toml_file() -> File {
    File::open(translation_toml_path()).expect("Failed to open tranlstation TOML file!")
}
