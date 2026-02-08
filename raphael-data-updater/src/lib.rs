mod recipe;
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

pub trait SheetData: Sized {
    const SHEET: &'static str;
    const REQUIRED_FIELDS: &[&str];
    fn row_id(&self) -> u32;
    fn from_json(value: &json::JsonValue) -> Option<Self>;
}

pub async fn fetch_and_parse<T: SheetData>(lang: &str, schema_override: Option<&str>) -> Vec<T> {
    let api_endpoint = match lang {
        "chs" => "https://boilmaster-chs.augenfrosch.dev/api",
        "ko" => "https://boilmaster-ko.augenfrosch.dev/api",
        "tc" => "https://boilmaster-tc.augenfrosch.dev/api",
        _ => "https://v2.xivapi.com/api",
    };

    let client = reqwest::Client::new();
    let get_response_text = async move |url: &str| -> Result<String, reqwest::Error> {
        client
            .get(url)
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
            "{api_endpoint}/sheet/{}?limit=1000&fields={}&after={}&language={}{}",
            T::SHEET,
            T::REQUIRED_FIELDS.join(","),
            last_row_id,
            lang,
            schema_override.map_or("".to_owned(), |s| format!("&schema={}", s)),
        );

        let mut remaining_attempts = 3;
        let mut retry_cooldown = std::time::Duration::from_secs(2);
        let response_text = loop {
            remaining_attempts -= 1;
            match get_response_text(&query_url).await {
                Ok(response) => break response,
                Err(error) => {
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
            return rows;
        }
        log::debug!("\"{}\": total fetched: {}", T::SHEET, rows.len());
    }
}
