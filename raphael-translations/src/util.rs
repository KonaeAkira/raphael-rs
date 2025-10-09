use std::path::{Path, PathBuf};

pub const RAPHAEL_TRANSLATIONS_TOML_ENV: &str = "RAPHAEL_TRANSLATIONS_TOML";

pub fn translation_toml_path() -> PathBuf {
    std::env::var(RAPHAEL_TRANSLATIONS_TOML_ENV).map_or(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("translations.toml"),
        |path_string| Path::new(&path_string).to_path_buf(),
    )
}
