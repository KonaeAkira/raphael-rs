#[cfg(feature = "update-toml")]
use std::io::{Seek, Write};
use std::{fs::OpenOptions, io::Read};

use toml_edit::DocumentMut;

#[cfg(feature = "update-toml")]
use crate::Occurance;
use crate::{Context, util::translation_toml_path};

pub struct Translation(pub &'static str, pub String);

pub fn get_translations(ctx: &Context) -> Vec<Translation> {
    #[allow(unused_variables)]
    // `text` & `occurance` are used when `update-toml` feature is enabled
    let Context {
        hash_base64,
        text,
        occurance,
    } = ctx;

    let toml_path = translation_toml_path();
    let mut toml_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(toml_path)
        .unwrap();
    toml_file.lock().unwrap();
    let mut doc = String::new();
    toml_file.read_to_string(&mut doc).unwrap();
    let mut doc = doc
        .parse::<DocumentMut>()
        .expect("Translation TOML file is not valid TOML!");

    let mut translations = Vec::new();
    if let Some(item) = doc.get_mut(hash_base64) {
        for language_key in &["de", "fr", "ja", "ko"] {
            if let Some(str) = item
                .get(language_key)
                .and_then(|item| item.as_value())
                .and_then(|value| value.as_str())
            {
                translations.push(Translation(language_key, str.to_owned()));
            }
        }
        #[cfg(feature = "update-toml")]
        if item["version"].as_str().unwrap() != env!("CARGO_PKG_VERSION") {
            item["version"] = toml_edit::value(env!("CARGO_PKG_VERSION"));
            let mut arr = toml_edit::Array::new();
            arr.push(occurance.source_location());
            item["appearances"] = toml_edit::value(arr);
        } else {
            let appearances = item["appearances"].as_array_mut().unwrap();
            let source_location = occurance.source_location();
            if !appearances
                .iter()
                .any(|value| value.as_str().map_or(false, |str| str == source_location))
            {
                appearances.push(source_location);
            }
        }
    } else {
        #[cfg(feature = "update-toml")]
        {
            doc[hash_base64] = toml_edit::table();
            match occurance {
                Occurance::Identifier(span) => {
                    doc[hash_base64]["identifier"] = toml_edit::value(text.trim_matches('\"'));
                    doc[hash_base64]["file"] = toml_edit::value(span.file());
                }
                Occurance::Literal(_) => {
                    doc[hash_base64]["en"] = format!("'''{}'''", text.trim_matches('\"'))
                        .parse::<toml_edit::Item>()
                        .unwrap();
                }
            }
            doc[hash_base64]["version"] = toml_edit::value(env!("CARGO_PKG_VERSION"));
            let mut arr = toml_edit::Array::new();
            arr.push(occurance.source_location());
            doc[hash_base64]["appearances"] = toml_edit::value(arr);
        }
    }

    #[cfg(feature = "update-toml")]
    {
        toml_file.set_len(0).unwrap();
        toml_file.rewind().unwrap();
        toml_file
            .write(doc.to_string().as_bytes())
            .expect("Failed to write to translation TOML file!");
    }

    translations
}
