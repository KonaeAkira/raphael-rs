use std::io::Write;
use std::{fs::File, io::BufWriter};

use raphael_data_updater::*;

async fn fetch_and_parse<T: SheetData>() -> Vec<T> {
    const XIV_API: &str = "https://v2.xivapi.com/api";
    let mut rows = Vec::new();
    loop {
        let last_row_id = rows.last().map_or(0, |row: &T| row.row_id());
        let query = format!(
            "{XIV_API}/sheet/{}?limit=1000&fields={}&after={}",
            T::SHEET,
            T::REQUIRED_FIELDS.join(","),
            last_row_id,
        );
        let response = reqwest::get(query).await.unwrap();
        let json = json::parse(&response.text().await.unwrap()).unwrap();

        let size = rows.len();
        rows.extend(json["rows"].members().filter_map(T::from_json));
        if size == rows.len() {
            return rows;
        }
        log::debug!("\"{}\": total fetched: {}", T::SHEET, rows.len());
    }
}

fn export_rlvls(rlvls: &[RecipeLevel]) {
    let path = std::path::absolute("./raphael-data/data/rlvls.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(&mut writer, "&[").unwrap();
    writeln!(&mut writer, "{},", RecipeLevel::default()).unwrap(); // index 0
    for rlvl in rlvls.iter() {
        writeln!(&mut writer, "{rlvl},").unwrap();
    }
    writeln!(&mut writer, "]").unwrap();
    log::info!("rlvls exported to \"{}\"", path.display());
}

fn export_recipes(recipes: &[Recipe]) {
    let mut phf_map = phf_codegen::OrderedMap::new();
    for recipe in recipes {
        phf_map.entry(recipe.id, &format!("{recipe}"));
    }
    let path = std::path::absolute("./raphael-data/data/recipes.rs").unwrap();
    let mut writer = BufWriter::new(File::create(&path).unwrap());
    writeln!(writer, "{}", phf_map.build()).unwrap();
    log::info!("recipes exported to \"{}\"", path.display());
}

#[tokio::main]
async fn main() {
    env_logger::builder().format_timestamp(None).init();
    let rlvls = fetch_and_parse::<RecipeLevel>().await;
    export_rlvls(&rlvls);
    let recipes = fetch_and_parse::<Recipe>().await;
    export_recipes(&recipes);
}
