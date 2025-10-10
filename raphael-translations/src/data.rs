use std::{fs::File, io::Read};
#[cfg(feature = "update-toml")]
use std::{
    fs::OpenOptions,
    io::{Seek, Write},
};

#[cfg(feature = "update-toml")]
use proc_macro::TokenTree;
use toml_edit::DocumentMut;

use crate::{Context, util::translation_toml_path};

#[cfg(feature = "update-toml")]
fn source_location_string(token_tree: &proc_macro::TokenTree) -> String {
    let span = token_tree.span();

    format!("{}:{}:{}", span.file(), span.line(), span.column())
}

#[cfg(feature = "update-toml")]
fn open_translation_toml_file() -> File {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(translation_toml_path())
        .expect("Failed to open tranlstation TOML file!");
    file.lock().unwrap();
    file
}

#[cfg(not(feature = "update-toml"))]
fn open_translation_toml_file() -> File {
    File::open(translation_toml_path()).expect("Failed to open tranlstation TOML file!")
}

#[derive(Debug)]
pub struct Translation(pub &'static str, pub String);

pub fn get_translations(ctx: &Context) -> Vec<Translation> {
    #[allow(unused_variables)]
    // `main_arg_token_tree` & `text` are used when `update-toml` feature is enabled
    let Context {
        hash_base64,
        main_arg_token_tree,
        text,
    } = ctx;

    let mut toml_file = open_translation_toml_file();
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
            arr.push(source_location_string(main_arg_token_tree));
            item["appearances"] = toml_edit::value(arr);
        } else {
            let appearances = item["appearances"].as_array_mut().unwrap();
            let source_location = source_location_string(main_arg_token_tree);
            if !appearances
                .iter()
                .any(|value| value.as_str().is_some_and(|str| str == source_location))
            {
                appearances.push(source_location);
            }
        }
    } else {
        #[cfg(feature = "update-toml")]
        {
            doc[hash_base64] = toml_edit::table();
            match main_arg_token_tree {
                TokenTree::Ident(_) => {
                    doc[hash_base64]["identifier"] = toml_edit::value(text);
                    doc[hash_base64]["file"] = toml_edit::value(main_arg_token_tree.span().file());
                }
                TokenTree::Literal(_) => {
                    doc[hash_base64]["en"] = format!("'''{}'''", text.trim_matches('\"'))
                        .parse::<toml_edit::Item>()
                        .unwrap();
                }
                _ => unreachable!(),
            }
            doc[hash_base64]["version"] = toml_edit::value(env!("CARGO_PKG_VERSION"));
            let mut arr = toml_edit::Array::new();
            arr.push(source_location_string(main_arg_token_tree));
            doc[hash_base64]["appearances"] = toml_edit::value(arr);
        }
    }

    #[cfg(feature = "update-toml")]
    {
        toml_file.set_len(0).unwrap();
        toml_file.rewind().unwrap();
        toml_file
            .write_all(doc.to_string().as_bytes())
            .expect("Failed to write to translation TOML file!");
    }

    translations
}
