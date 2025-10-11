#[cfg(feature = "update-toml")]
use std::io::{Read, Seek, Write};

#[path = "src/util.rs"]
mod util;

fn main() {
    let toml_path = util::translation_toml_path();
    println!(
        "cargo:rerun-if-env-changed={}",
        util::RAPHAEL_TRANSLATIONS_TOML_ENV
    );
    println!(
        "cargo::rerun-if-changed={}",
        toml_path
            .to_str()
            .expect("Translation TOML file path is not valid unicode!")
    );

    #[cfg(feature = "update-toml")]
    if std::env::var("RAPHAEL_TRANSLATIONS_RESET_APPEARANCES").is_ok() {
        let mut toml_file = util::open_translation_toml_file();
        let mut doc = String::new();
        toml_file.read_to_string(&mut doc).unwrap();
        let mut doc = doc
            .parse::<toml_edit::DocumentMut>()
            .expect("Translation TOML file is not valid TOML!");

        for (_key, item) in doc.iter_mut() {
            if let Some(version) = item.get("version").and_then(|item| item.as_str())
                && version == env!("CARGO_PKG_VERSION")
                && let Some(array) = item["appearances"].as_array_mut()
            {
                array.clear();
            }
        }

        toml_file.set_len(0).unwrap();
        toml_file.rewind().unwrap();
        toml_file
            .write_all(doc.to_string().as_bytes())
            .expect("Failed to write to translation TOML file!");
    }
}
