//! Key-value persistence.
//!
//! Values are stored as RON strings under the same keys the previous (egui-based) versions of the
//! app used, so existing settings, crafter stats and saved rotations carry over:
//! - Web: one `localStorage` entry per key (same layout as eframe's web storage).
//! - Native: a RON-encoded `HashMap<String, String>` in the app's data directory. On first launch
//!   the entries are imported from the previous versions' `app.ron`.

use serde::{Serialize, de::DeserializeOwned};

pub fn load<T: DeserializeOwned>(key: &str) -> Option<T> {
    let value = imp::get_string(key)?;
    match ron::from_str(&value) {
        Ok(value) => Some(value),
        Err(error) => {
            log::warn!("Failed to decode stored value for key {key}: {error}");
            None
        }
    }
}

pub fn load_or_default<T: DeserializeOwned + Default>(key: &str) -> T {
    load(key).unwrap_or_default()
}

/// A set of values to persist together.
#[derive(Default)]
#[must_use = "a batch does nothing until it is committed"]
pub struct Batch {
    entries: Vec<(String, String)>,
}

impl Batch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Serializes `value` and queues it to be stored under `key`.
    pub fn store<T: Serialize>(&mut self, key: &str, value: &T) -> &mut Self {
        match ron::to_string(value) {
            Ok(string) => self.entries.push((key.to_owned(), string)),
            Err(error) => log::error!("Failed to encode value for key {key}: {error}"),
        }
        self
    }

    /// Writes all queued entries to storage. Entries whose value is unchanged are skipped.
    pub fn commit(self) {
        if !self.entries.is_empty() {
            imp::set_strings(self.entries);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod imp {
    fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub fn get_string(key: &str) -> Option<String> {
        local_storage()?.get_item(key).ok()?
    }

    pub fn set_strings(entries: Vec<(String, String)>) {
        let Some(storage) = local_storage() else {
            return;
        };
        for (key, value) in entries {
            if storage.get_item(&key).ok().flatten().as_ref() != Some(&value) {
                let _ = storage.set_item(&key, &value);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::{LazyLock, Mutex},
    };

    const FILE_NAME: &str = "settings.ron";
    const LEGACY_FILE_NAME: &str = "app.ron";

    struct FileStorage {
        path: Option<PathBuf>,
        kv: HashMap<String, String>,
    }

    static STORAGE: LazyLock<Mutex<FileStorage>> = LazyLock::new(|| {
        let dir = super::data_dir();
        let read = |name: &str| -> Option<HashMap<String, String>> {
            let file = std::fs::File::open(dir.as_ref()?.join(name)).ok()?;
            ron::de::from_reader(std::io::BufReader::new(file))
                .inspect_err(|error| log::warn!("Failed to parse {name}: {error}"))
                .ok()
        };
        let kv = read(FILE_NAME)
            .or_else(|| read(LEGACY_FILE_NAME))
            .unwrap_or_default();
        Mutex::new(FileStorage {
            path: dir.map(|dir| dir.join(FILE_NAME)),
            kv,
        })
    });

    pub fn get_string(key: &str) -> Option<String> {
        STORAGE.lock().unwrap().kv.get(key).cloned()
    }

    pub fn set_strings(entries: Vec<(String, String)>) {
        let mut storage = STORAGE.lock().unwrap();
        let mut changed = false;
        for (key, value) in entries {
            if storage.kv.get(&key) != Some(&value) {
                storage.kv.insert(key, value);
                changed = true;
            }
        }
        if !changed {
            return;
        }
        let Some(path) = storage.path.clone() else {
            return;
        };
        let serialized =
            match ron::ser::to_string_pretty(&storage.kv, ron::ser::PrettyConfig::default()) {
                Ok(serialized) => serialized,
                Err(error) => {
                    log::error!("Failed to serialize settings: {error}");
                    return;
                }
            };
        drop(storage);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // Write to a temporary file first so that a crash mid-write doesn't corrupt the settings.
        let temp_path = path.with_extension("ron.tmp");
        let result = std::fs::write(&temp_path, serialized)
            .and_then(|()| std::fs::rename(&temp_path, &path));
        if let Err(error) = result {
            log::error!("Failed to write {}: {error}", path.display());
        }
    }
}

/// Directory used for persisted data, shared with the previous versions of the app.
#[cfg(not(target_arch = "wasm32"))]
pub fn data_dir() -> Option<std::path::PathBuf> {
    directories::ProjectDirs::from("", "", "Raphael XIV").map(|dirs| dirs.data_dir().to_path_buf())
}
