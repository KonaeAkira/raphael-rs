use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::records::ItemRecord;
use crate::utils::read_csv_data;

pub fn import_item_records() -> Result<(), Box<dyn std::error::Error>> {
    let mut item_stats = phf_codegen::OrderedMap::new();
    for item in read_csv_data::<ItemRecord>("data/en/Item.csv") {
        item_stats.entry(item.id, &format!(
            "Item {{ item_level: {item_level}, can_be_hq: {can_be_hq}, is_collectable: {is_collectable} }}",
            item_level = item.item_level,
            can_be_hq = item.can_be_hq,
            is_collectable = item.is_collectable,
        ));
    }
    let out_path = Path::new(&std::env::var("OUT_DIR")?).join("items.rs");
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writeln!(writer, "{}", item_stats.build())?;

    import_item_names("en")?;
    import_item_names("de")?;
    import_item_names("fr")?;
    import_item_names("jp")?;

    Ok(())
}

fn import_item_names(lang: &'static str) -> Result<(), Box<dyn std::error::Error>> {
    let mut item_names = phf_codegen::Map::new();
    for item in read_csv_data::<ItemRecord>(format!("data/{}/Item.csv", lang)) {
        item_names.entry(item.id, &format!("\"{}\"", item.name));
    }
    let out_path = Path::new(&std::env::var("OUT_DIR")?).join(format!("item_names_{}.rs", lang));
    let mut writer = BufWriter::new(File::create(out_path).unwrap());
    writeln!(writer, "{}", item_names.build())?;
    Ok(())
}
