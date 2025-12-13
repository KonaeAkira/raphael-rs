use raphael_data::Locale;

pub mod ingredients;
pub mod search_mission;
pub mod search_recipe;
pub mod solve;

#[derive(Copy, Clone, clap::ValueEnum, Debug)]
pub enum Language {
    EN,
    DE,
    FR,
    JP,
    CN,
    KR,
    TW,
}

impl From<Language> for Locale {
    fn from(val: Language) -> Self {
        match val {
            Language::EN => Locale::EN,
            Language::DE => Locale::DE,
            Language::FR => Locale::FR,
            Language::JP => Locale::JP,
            Language::CN => Locale::CN,
            Language::KR => Locale::KR,
            Language::TW => Locale::TW,
        }
    }
}
