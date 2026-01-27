use std::collections::HashSet;
use std::io::Write;
use std::time::Duration;
use std::{fs::File, io::BufWriter};

use raphael_data_updater::*;

async fn fetch_and_parse<T: SheetData>(lang: &str, schema_override: Option<&str>) -> Vec<T> {
    const XIV_API: &str = "https://v2.xivapi.com/api";
    const BOILMASTER_CN: &str = "https://boilmaster-chs.augenfrosch.dev/api";
    const BOILMASTER_KR: &str = "https://boilmaster-ko.augenfrosch.dev/api";
    const BOILMASTER_TW: &str = "https://boilmaster-tc.augenfrosch.dev/api";
    let api_endpoint = match lang {
        "chs" => BOILMASTER_CN,
        "ko" => BOILMASTER_KR,
        "tc" => BOILMASTER_TW,
        _ => XIV_API,
    };
    let mut rows = Vec::new();
    loop {
        let last_row_id = rows.last().map_or(0, |row: &T| row.row_id());
        let query = format!(
            "{api_endpoint}/sheet/{}?limit=1000&fields={}&after={}&language={}{}",
            T::SHEET,
            T::REQUIRED_FIELDS.join(","),
            last_row_id,
            lang,
            schema_override.map_or("".to_owned(), |s| format!("&schema={}", s)),
        );

        let mut remaining_attempts = 3;
        let mut retry_cooldown = Duration::from_secs(2);
        let response = loop {
            remaining_attempts -= 1;
            match reqwest::get(&query).await {
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

        let json = json::parse(&response.text().await.unwrap()).unwrap();

        let size = rows.len();
        rows.extend(json["rows"].members().filter_map(T::from_json));
        if size == rows.len() {
            return rows;
        }
        log::info!("\"{}\": total fetched: {}", T::SHEET, rows.len());
    }
}

fn export_rlvls(rlvls: &[RecipeLevel]) {
    let path = std::path::absolute("./raphael-data/data/rlvls.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(&mut writer, "&[").unwrap();
    writeln!(&mut writer, "{},", RecipeLevel::default()).unwrap(); // index 0
    for rlvl in rlvls {
        writeln!(&mut writer, "{rlvl},").unwrap();
    }
    writeln!(&mut writer, "]").unwrap();
    log::info!("rlvls exported to \"{}\"", path.display());
}

fn export_level_adjust_table(level_adjust_table_entries: &[LevelAdjustTableEntry]) {
    let path = std::path::absolute("./raphael-data/data/level_adjust_table.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(&mut writer, "&[").unwrap();
    writeln!(&mut writer, "{},", u16::default()).unwrap(); // index 0
    for entry in level_adjust_table_entries {
        writeln!(&mut writer, "{entry},").unwrap();
    }
    writeln!(&mut writer, "]").unwrap();
    log::info!("Level adjust table exported to \"{}\"", path.display());
}

fn export_recipes(recipes: &[Recipe]) {
    let path = std::path::absolute("./raphael-data/data/recipes.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(writer, "nci_array! {{").unwrap();
    for recipe in recipes {
        writeln!(writer, "{} => {},", recipe.id, recipe,).unwrap();
    }
    writeln!(writer, "}}").unwrap();
    log::info!("recipes exported to \"{}\"", path.display());
}

fn export_items(items: &[Item]) {
    let path = std::path::absolute("./raphael-data/data/items.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(writer, "nci_array! {{").unwrap();
    for item in items {
        writeln!(writer, "{} => {},", item.id, item,).unwrap();
    }
    writeln!(writer, "}}").unwrap();
    log::info!("items exported to \"{}\"", path.display());
}

fn export_meals(consumables: &[Consumable]) {
    let path = std::path::absolute("./raphael-data/data/meals.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(&mut writer, "&[").unwrap();
    for consumable in consumables {
        writeln!(&mut writer, "{consumable},").unwrap();
    }
    writeln!(&mut writer, "]").unwrap();
    log::info!("meals exported to \"{}\"", path.display());
}

fn export_potions(consumables: &[Consumable]) {
    let path = std::path::absolute("./raphael-data/data/potions.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(&mut writer, "&[").unwrap();
    for consumable in consumables {
        writeln!(&mut writer, "{consumable},").unwrap();
    }
    writeln!(&mut writer, "]").unwrap();
    log::info!("potions exported to \"{}\"", path.display());
}

fn export_stellar_missions(stellar_missions: &[StellarMission]) {
    let path = std::path::absolute("./raphael-data/data/stellar_missions.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(writer, "nci_array! {{").unwrap();
    for stellar_mission in stellar_missions {
        writeln!(writer, "{} => {},", stellar_mission.id, stellar_mission,).unwrap();
    }
    writeln!(writer, "}}").unwrap();
    log::info!("stellar missions exported to \"{}\"", path.display());
}

fn export_item_names(item_names: &[ItemName], lang: &str) {
    let path = std::path::absolute(format!("./raphael-data/data/item_names_{lang}.rs")).unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(writer, "nci_array! {{").unwrap();
    for item_name in item_names {
        writeln!(writer, "{} => {:?},", item_name.id, item_name.name,).unwrap();
    }
    writeln!(writer, "}}").unwrap();
    log::info!("item names exported to \"{}\"", path.display());
}

fn export_stellar_mission_names(stellar_mission_names: &[StellarMissionName], lang: &str) {
    let path = std::path::absolute(format!(
        "./raphael-data/data/stellar_mission_names_{lang}.rs"
    ))
    .unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(writer, "nci_array! {{").unwrap();
    for stellar_mission_name in stellar_mission_names {
        writeln!(
            writer,
            "{} => {:?},",
            stellar_mission_name.id, stellar_mission_name.name,
        )
        .unwrap();
    }
    writeln!(writer, "}}").unwrap();
    log::info!("stellar mission names exported to \"{}\"", path.display());
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();

    let rlvls = tokio::spawn(async { fetch_and_parse::<RecipeLevel>("en", None).await });
    let level_adjust_table_entries =
        tokio::spawn(async { fetch_and_parse::<LevelAdjustTableEntry>("en", None).await });
    let recipes = tokio::spawn(async { fetch_and_parse::<Recipe>("en", None).await });
    let items = tokio::spawn(async { fetch_and_parse::<Item>("en", None).await });
    let item_actions = tokio::spawn(async { fetch_and_parse::<ItemAction>("en", None).await });
    let item_foods = tokio::spawn(async { fetch_and_parse::<ItemFood>("en", None).await });
    let stellar_missions =
        tokio::spawn(async { fetch_and_parse::<StellarMission>("en", None).await });

    let item_names_en = tokio::spawn(async { fetch_and_parse::<ItemName>("en", None).await });
    let item_names_de = tokio::spawn(async { fetch_and_parse::<ItemName>("de", None).await });
    let item_names_fr = tokio::spawn(async { fetch_and_parse::<ItemName>("fr", None).await });
    let item_names_jp = tokio::spawn(async { fetch_and_parse::<ItemName>("ja", None).await });
    let item_names_cn = tokio::spawn(async { fetch_and_parse::<ItemName>("chs", None).await });
    let item_names_kr = tokio::spawn(async { fetch_and_parse::<ItemName>("ko", None).await });
    let item_names_tw = tokio::spawn(async { fetch_and_parse::<ItemName>("tc", None).await });

    let stellar_mission_names_en =
        tokio::spawn(async { fetch_and_parse::<StellarMissionName>("en", None).await });
    let stellar_mission_names_de =
        tokio::spawn(async { fetch_and_parse::<StellarMissionName>("de", None).await });
    let stellar_mission_names_fr =
        tokio::spawn(async { fetch_and_parse::<StellarMissionName>("fr", None).await });
    let stellar_mission_names_jp =
        tokio::spawn(async { fetch_and_parse::<StellarMissionName>("ja", None).await });
    let stellar_mission_names_cn =
        tokio::spawn(async { fetch_and_parse::<StellarMissionName>("chs", None).await });
    let stellar_mission_names_kr =
        tokio::spawn(async { fetch_and_parse::<StellarMissionName>("ko", None).await });
    let stellar_mission_names_tw = tokio::spawn(async {
        fetch_and_parse::<StellarMissionName>("tc", Some("exdschema@2:rev:cc92abc")).await
    });

    let rlvls = rlvls.await.unwrap();
    let level_adjust_table_entries = level_adjust_table_entries.await.unwrap();
    let mut recipes = recipes.await.unwrap();
    let mut items = items.await.unwrap();

    let item_actions = item_actions.await.unwrap();
    let item_foods = item_foods.await.unwrap();
    let (meals, potions) = instantiate_consumables(&items, item_actions, item_foods);

    let stellar_missions = stellar_missions.await.unwrap();

    let mut item_names_en = item_names_en.await.unwrap();
    let mut item_names_de = item_names_de.await.unwrap();
    let mut item_names_fr = item_names_fr.await.unwrap();
    let mut item_names_jp = item_names_jp.await.unwrap();
    let mut item_names_cn = item_names_cn.await.unwrap();
    let mut item_names_kr = item_names_kr.await.unwrap();
    let mut item_names_tw = item_names_tw.await.unwrap();

    let mut stellar_mission_names_en = stellar_mission_names_en.await.unwrap();
    let mut stellar_mission_names_de = stellar_mission_names_de.await.unwrap();
    let mut stellar_mission_names_fr = stellar_mission_names_fr.await.unwrap();
    let mut stellar_mission_names_jp = stellar_mission_names_jp.await.unwrap();
    let mut stellar_mission_names_cn = stellar_mission_names_cn.await.unwrap();
    let mut stellar_mission_names_kr = stellar_mission_names_kr.await.unwrap();
    let mut stellar_mission_names_tw = stellar_mission_names_tw.await.unwrap();

    // For some reason some recipes have items with ID 0 as their result
    recipes.retain(|recipe| recipe.item_id != 0);

    // Remove recipe ingredients that cannot be HQ as those aren't used when calculating initial Quality
    let hq_items: HashSet<_> = items
        .iter()
        .filter_map(|item| if item.can_be_hq { Some(item.id) } else { None })
        .collect();
    for recipe in &mut recipes {
        recipe
            .ingredients
            .retain(|ingredient| hq_items.contains(&ingredient.item_id));
    }

    // Only retain necessary items to reduce binary size
    let mut necessary_items: HashSet<u32> = HashSet::new();
    for recipe in &recipes {
        necessary_items.insert(recipe.item_id);
        necessary_items.extend(
            recipe
                .ingredients
                .iter()
                .map(|ingredient| ingredient.item_id),
        );
    }
    necessary_items.extend(
        meals
            .iter()
            .chain(potions.iter())
            .map(|consumable| consumable.item_id),
    );
    items.retain(|item| necessary_items.contains(&item.id));
    item_names_en.retain(|item_name| necessary_items.contains(&item_name.id));
    item_names_de.retain(|item_name| necessary_items.contains(&item_name.id));
    item_names_fr.retain(|item_name| necessary_items.contains(&item_name.id));
    item_names_jp.retain(|item_name| necessary_items.contains(&item_name.id));
    item_names_cn
        .retain(|item_name| necessary_items.contains(&item_name.id) && !item_name.name.is_empty());
    item_names_kr
        .retain(|item_name| necessary_items.contains(&item_name.id) && !item_name.name.is_empty());
    item_names_tw
        .retain(|item_name| necessary_items.contains(&item_name.id) && !item_name.name.is_empty());

    // Parsing of stellar mission json already filters out gatherer only missions
    let crafter_stellar_missions: HashSet<u32> = stellar_missions
        .iter()
        .map(|stellar_mission| stellar_mission.id)
        .collect();
    stellar_mission_names_en
        .retain(|stellar_mission_name| crafter_stellar_missions.contains(&stellar_mission_name.id));
    stellar_mission_names_de
        .retain(|stellar_mission_name| crafter_stellar_missions.contains(&stellar_mission_name.id));
    stellar_mission_names_fr
        .retain(|stellar_mission_name| crafter_stellar_missions.contains(&stellar_mission_name.id));
    stellar_mission_names_jp
        .retain(|stellar_mission_name| crafter_stellar_missions.contains(&stellar_mission_name.id));
    stellar_mission_names_cn.retain(|stellar_mission_name| {
        crafter_stellar_missions.contains(&stellar_mission_name.id)
            && !stellar_mission_name.name.is_empty()
    });
    stellar_mission_names_kr.retain(|stellar_mission_name| {
        crafter_stellar_missions.contains(&stellar_mission_name.id)
            && !stellar_mission_name.name.is_empty()
    });
    stellar_mission_names_tw.retain(|stellar_mission_name| {
        crafter_stellar_missions.contains(&stellar_mission_name.id)
            && !stellar_mission_name.name.is_empty()
    });

    export_rlvls(&rlvls);
    export_level_adjust_table(&level_adjust_table_entries);
    export_recipes(&recipes);
    export_meals(&meals);
    export_potions(&potions);
    export_items(&items);
    export_stellar_missions(&stellar_missions);

    export_item_names(&item_names_en, "en");
    export_item_names(&item_names_de, "de");
    export_item_names(&item_names_fr, "fr");
    export_item_names(&item_names_jp, "jp");
    export_item_names(&item_names_cn, "cn");
    export_item_names(&item_names_kr, "kr");
    export_item_names(&item_names_tw, "tw");

    export_stellar_mission_names(&stellar_mission_names_en, "en");
    export_stellar_mission_names(&stellar_mission_names_de, "de");
    export_stellar_mission_names(&stellar_mission_names_fr, "fr");
    export_stellar_mission_names(&stellar_mission_names_jp, "jp");
    export_stellar_mission_names(&stellar_mission_names_cn, "cn");
    export_stellar_mission_names(&stellar_mission_names_kr, "kr");
    export_stellar_mission_names(&stellar_mission_names_tw, "tw");
}
