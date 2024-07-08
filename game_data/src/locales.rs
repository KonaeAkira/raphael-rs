#[derive(Debug, Clone, Copy)]
pub enum Locale {
    EN,
    DE,
    FR,
    JP,
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EN => write!(f, "English"),
            Self::DE => write!(f, "Deutsch"),
            Self::FR => write!(f, "Français"),
            Self::JP => write!(f, "日本語"),
        }
    }
}

static ITEM_NAMES_EN: phf::Map<u32, &'static str> =
    include!(concat!(env!("OUT_DIR"), "/item_names_en.rs"));
static ITEM_NAMES_DE: phf::Map<u32, &'static str> =
    include!(concat!(env!("OUT_DIR"), "/item_names_de.rs"));
static ITEM_NAMES_FR: phf::Map<u32, &'static str> =
    include!(concat!(env!("OUT_DIR"), "/item_names_fr.rs"));
static ITEM_NAMES_JP: phf::Map<u32, &'static str> =
    include!(concat!(env!("OUT_DIR"), "/item_names_jp.rs"));

pub fn get_item_name(item_id: u32, locale: Locale) -> &'static str {
    match locale {
        Locale::EN => ITEM_NAMES_EN
            .get(&item_id)
            .copied()
            .unwrap_or("Unknown item"),
        Locale::DE => ITEM_NAMES_DE
            .get(&item_id)
            .copied()
            .unwrap_or("Unknown item"),
        Locale::FR => ITEM_NAMES_FR
            .get(&item_id)
            .copied()
            .unwrap_or("Unknown item"),
        Locale::JP => ITEM_NAMES_JP
            .get(&item_id)
            .copied()
            .unwrap_or("Unknown item"),
    }
}
