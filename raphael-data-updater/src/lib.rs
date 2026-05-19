mod recipe;
use std::{
    fs::{self, File},
    io::{Read, Seek, Write},
    path::Path,
};

pub use recipe::Recipe;

mod rlvl;
pub use rlvl::RecipeLevel;

mod level_adjust_table;
pub use level_adjust_table::LevelAdjustTableEntry;

mod item;
pub use item::{Item, ItemName};

mod consumable;
pub use consumable::{Consumable, ItemAction, ItemFood, instantiate_consumables};

mod stellar_mission;
pub use stellar_mission::{StellarMission, StellarMissionName};
use toml_edit::DocumentMut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    EN,
    DE,
    FR,
    JP,
    CN,
    KR,
    TW,
}

impl Lang {
    /// Language code used for xivapi.
    pub fn xivapi_langcode(self) -> &'static str {
        match self {
            Self::EN => "en",
            Self::DE => "de",
            Self::FR => "fr",
            Self::JP => "ja",
            Self::CN => "chs",
            Self::KR => "ko",
            Self::TW => "tc",
        }
    }

    /// Shortcode used within raphael.
    pub fn shortcode(self) -> &'static str {
        match self {
            Self::EN => "en",
            Self::DE => "de",
            Self::FR => "fr",
            Self::JP => "jp",
            Self::CN => "cn",
            Self::KR => "kr",
            Self::TW => "tw",
        }
    }

    fn api_endpoint(self) -> &'static str {
        match self {
            Self::EN | Self::DE | Self::FR | Self::JP => "https://v2.xivapi.com/api",
            Self::CN => "https://boilmaster-chs.augenfrosch.dev/api",
            Self::KR => "https://boilmaster-ko.augenfrosch.dev/api",
            Self::TW => "https://boilmaster-tc.augenfrosch.dev/api",
        }
    }

    fn schema_override(self) -> Option<&'static str> {
        match self {
            Self::TW => Some("exdschema@2:rev:cc92abc"),
            _ => None,
        }
    }
}

pub struct XivapiVersionInfo {
    pub key: String,
    pub name: Option<String>,
}

pub struct LatestVersions {
    pub global: XivapiVersionInfo,
    pub cn: XivapiVersionInfo,
    pub kr: XivapiVersionInfo,
    pub tw: XivapiVersionInfo,
    file: File,
    toml: DocumentMut,
}

impl LatestVersions {
    pub fn new(
        global: XivapiVersionInfo,
        cn: XivapiVersionInfo,
        kr: XivapiVersionInfo,
        tw: XivapiVersionInfo,
    ) -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("latest-versions.toml");
        let (file, toml) = if path.is_file() {
            let mut file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .expect("Failed to open latest versions TOML file");
            file.lock().unwrap();
            let mut latest_versions = String::new();
            file.read_to_string(&mut latest_versions).unwrap();
            let toml = latest_versions.parse::<DocumentMut>().unwrap();
            (file, toml)
        } else {
            let file = File::create(path).expect("Failed to create latest versions TOML file");
            file.lock().unwrap();
            let toml = DocumentMut::new();
            (file, toml)
        };

        Self {
            global,
            cn,
            kr,
            tw,
            file,
            toml,
        }
    }

    pub fn versions_match_saved(&self) -> bool {
        !self.toml.is_empty()
            && self.toml["global"].as_str() == Some(&self.global.key)
            && self.toml["cn"].as_str() == Some(&self.cn.key)
            && self.toml["kr"].as_str() == Some(&self.kr.key)
            && self.toml["tw"].as_str() == Some(&self.tw.key)
    }

    pub fn write_versions_to_file(mut self) {
        for (version, version_info) in [
            ("global", self.global),
            ("cn", self.cn),
            ("kr", self.kr),
            ("tw", self.tw),
        ] {
            let XivapiVersionInfo { key, name } = version_info;
            let value = toml_edit::Value::String(toml_edit::Formatted::new(key)).decorated(
                " ",
                name.map_or::<toml_edit::RawString, _>("".into(), |name| {
                    format!(" # {}", name).into()
                }),
            );
            self.toml[version] = toml_edit::Item::Value(value);
        }

        self.file.set_len(0).unwrap();
        self.file.rewind().unwrap();
        self.file
            .write_all(self.toml.to_string().as_bytes())
            .unwrap();
    }
}

pub async fn fetch_latest_version_info(lang: Lang) -> Result<XivapiVersionInfo, reqwest::Error> {
    let response_text = reqwest::Client::new()
        .get(format!("{}/version", lang.api_endpoint()))
        .send()
        .await?
        .text()
        .await?;
    let json = json::parse(&response_text).unwrap();
    Ok(json["versions"]
        .members()
        .find_map(|version| {
            if version["names"].contains("latest") {
                Some(XivapiVersionInfo {
                    key: version["key"].as_str().unwrap().to_string(),
                    name: version["names"].members().find_map(|name| {
                        let name = name.as_str().unwrap();
                        if name != "latest" {
                            Some(name.to_string())
                        } else {
                            None
                        }
                    }),
                })
            } else {
                None
            }
        })
        .unwrap())
}

pub trait SheetData: Sized {
    const SHEET: &'static str;
    const REQUIRED_FIELDS: &[&str];
    fn row_id(&self) -> u32;
    fn from_json(value: &json::JsonValue) -> Option<Self>;
}

pub async fn fetch_and_parse<T: SheetData>(lang: Lang) -> Vec<T> {
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ))
        .build()
        .unwrap();
    let get_response_text = async |url: &str| -> Result<String, reqwest::Error> {
        client
            .get(url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await
    };

    let mut rows = Vec::new();
    loop {
        let last_row_id = rows.last().map_or(0, |row: &T| row.row_id());
        let query_url = format!(
            "{}/sheet/{}?limit=1000&fields={}&after={}&language={}{}",
            lang.api_endpoint(),
            T::SHEET,
            T::REQUIRED_FIELDS.join(","),
            last_row_id,
            lang.xivapi_langcode(),
            lang.schema_override()
                .map_or("".to_owned(), |s| format!("&schema={}", s)),
        );

        let mut remaining_attempts = 3;
        let mut retry_cooldown = std::time::Duration::from_secs(2);
        let response_text = loop {
            remaining_attempts -= 1;
            match get_response_text(&query_url).await {
                Ok(response) => break response,
                Err(error) => {
                    let is_client_error = error
                        .status()
                        .is_some_and(|status_code| status_code.is_client_error());
                    if is_client_error {
                        log::error!("{:?}. Error is a client error, not retrying.", error);
                        panic!("Failed to query API.");
                    }
                    if remaining_attempts > 0 {
                        log::warn!("{:?}. Retrying...", error);
                        std::thread::sleep(retry_cooldown);
                        retry_cooldown *= 2;
                    } else {
                        log::error!("{:?}. Retry attempts exhausted.", error);
                        panic!("Failed to query API.");
                    }
                }
            }
        };

        let json = json::parse(&response_text).unwrap();

        let size = rows.len();
        rows.extend(json["rows"].members().filter_map(T::from_json));
        if size == rows.len() {
            log::info!("{} {lang:?}: Done fetching ({} rows)", T::SHEET, rows.len());
            return rows;
        }
        log::debug!("{} {lang:?}: Fetching ({} rows)", T::SHEET, rows.len());
    }
}
