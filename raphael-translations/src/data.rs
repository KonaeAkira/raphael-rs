use std::io::Read;
#[cfg(feature = "update-toml")]
use std::io::{Seek, Write};

use toml_edit::DocumentMut;

use crate::{StringLiteralDetails, translation::Translation, util::open_translation_toml_file};

pub fn generate_toml_entry_hash(string_literal_body: &str) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use rustc_stable_hash::FromStableHash;
    use rustc_stable_hash::hashers::{SipHasher128Hash, StableSipHasher128};
    use std::hash::Hasher;

    struct HashU128(u128);
    impl FromStableHash for HashU128 {
        type Hash = SipHasher128Hash;

        fn from(SipHasher128Hash(hash): SipHasher128Hash) -> Self {
            Self(u128::from(hash[0]) | u128::from(hash[1]).unbounded_shl(64))
        }
    }

    let mut hasher = StableSipHasher128::new();
    for byte in string_literal_body.as_bytes() {
        hasher.write_u8(*byte);
    }

    STANDARD.encode(hasher.finish::<HashU128>().0.to_le_bytes())
}

pub fn get_translations(literal_details: StringLiteralDetails) -> Vec<Translation> {
    let hash = &generate_toml_entry_hash(&literal_details.body);

    let mut toml_file = open_translation_toml_file();
    let mut doc = String::new();
    toml_file.read_to_string(&mut doc).unwrap();
    let mut doc = doc
        .parse::<DocumentMut>()
        .expect("Translation TOML file is not valid TOML!");

    let mut translations = Vec::new();
    if let Some(item) = doc.get_mut(hash) {
        for language_key in &["de", "fr", "ja", "chs", "ko"] {
            if let Some(str) = item
                .get(language_key)
                .and_then(|item| item.as_value())
                .and_then(|value| value.as_str())
            {
                translations.push(Translation::new(language_key, str, &literal_details));
            }
        }
        #[cfg(feature = "update-toml")]
        if item["version"].as_str().unwrap() != env!("CARGO_PKG_VERSION") {
            item["version"] = toml_edit::value(env!("CARGO_PKG_VERSION"));
            let mut arr = toml_edit::Array::new();
            arr.push(literal_details.source_location_string());
            item["appearances"] = toml_edit::value(arr);
        } else {
            let appearances = item["appearances"].as_array_mut().unwrap();
            let source_location = literal_details.source_location_string();
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
            doc[hash] = toml_edit::table();
            doc[hash]["en"] = format!("'''{}'''", literal_details.body)
                .parse::<toml_edit::Item>()
                .unwrap();
            doc[hash]["version"] = toml_edit::value(env!("CARGO_PKG_VERSION"));
            let mut arr = toml_edit::Array::new();
            arr.push(literal_details.source_location_string());
            doc[hash]["appearances"] = toml_edit::value(arr);
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
