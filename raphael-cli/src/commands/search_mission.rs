use clap::{Args, ValueEnum};
use raphael_data::{
    Locale, RECIPES, STELLAR_MISSIONS, StellarMission, get_job_name, get_stellar_mission_name,
};

#[derive(Args, Debug)]
pub struct SearchArgs {
    /// Search string to use, can be partial name
    #[arg(short, long, required_unless_present_any(["recipe_id", "item_id", "mission_id"]), conflicts_with_all(["recipe_id", "item_id"]))]
    pub pattern: Option<String>,

    /// Recipe ID to search for
    #[arg(short, long, required_unless_present_any(["pattern", "item_id", "mission_id"]), conflicts_with = "item_id")]
    pub recipe_id: Option<u32>,

    /// Item ID to search for
    #[arg(short, long, required_unless_present_any(["pattern", "recipe_id", "mission_id"]))]
    pub item_id: Option<u32>,

    /// Stellar mission ID to search for
    #[arg(long, required_unless_present_any(["pattern", "recipe_id", "item_id"]), conflicts_with_all(["pattern", "recipe_id", "item_id"]))]
    pub mission_id: Option<u32>,

    /// The delimiter the output uses between fields
    #[arg(long, alias = "OFS", default_value = " ", env = "OFS")]
    output_field_separator: String,

    /// The language the input pattern and output use
    #[arg(short, long, alias = "locale", value_enum, ignore_case = true, default_value_t = SearchLanguage::EN)]
    language: SearchLanguage,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
pub enum SearchLanguage {
    EN,
    DE,
    FR,
    JP,
    KR,
}

impl From<SearchLanguage> for Locale {
    fn from(val: SearchLanguage) -> Self {
        match val {
            SearchLanguage::EN => Locale::EN,
            SearchLanguage::DE => Locale::DE,
            SearchLanguage::FR => Locale::FR,
            SearchLanguage::JP => Locale::JP,
            SearchLanguage::KR => Locale::KR,
        }
    }
}

pub fn execute(args: &SearchArgs) {
    let locale = args.language.into();
    let mut matches: Vec<(u32, &StellarMission)> = Vec::new();

    if let Some(mission_id_arg) = args.mission_id {
        matches.extend(
            STELLAR_MISSIONS
                .entries()
                .find(|(mission_id, _)| *mission_id == mission_id_arg),
        );
    }
    if let Some(pattern_arg) = &args.pattern {
        matches.extend(raphael_data::find_stellar_missions(pattern_arg, locale));
    }
    if let Some(recipe_id_arg) = args.recipe_id {
        matches.extend(
            STELLAR_MISSIONS
                .entries()
                .filter(|(_, mission)| mission.recipe_ids.contains(&recipe_id_arg)),
        );
    }
    if let Some(item_id_arg) = args.item_id {
        matches.extend(STELLAR_MISSIONS.entries().filter(|(mission_id, mission)| {
            mission
                .recipe_ids
                .iter()
                .find(|recipe_id| {
                    if let Some(recipe) = RECIPES.get(**recipe_id) {
                        recipe.item_id == item_id_arg
                    } else {
                        log::warn!(
                            "Mission {} references missing recipe id {}",
                            mission_id,
                            recipe_id
                        );
                        false
                    }
                })
                .is_some()
        }));
    };
    if matches.is_empty() {
        println!("No matches found");
        return;
    }

    for (mission_id, mission) in matches {
        let mission_name =
            get_stellar_mission_name(mission_id, locale).unwrap_or("Unknown Mission");
        println!(
            "{mission_id}{separator}{job_name}{separator}{mission_name}{separator}{recipe_id_0}{separator}{recipe_id_1}{separator}{recipe_id_2}{separator}{recipe_id_3}{separator}{recipe_id_4}",
            job_name = get_job_name(mission.job_id, locale),
            separator = args.output_field_separator,
            recipe_id_0 = mission
                .recipe_ids
                .get(0)
                .map(ToString::to_string)
                .unwrap_or_default(),
            recipe_id_1 = mission
                .recipe_ids
                .get(1)
                .map(ToString::to_string)
                .unwrap_or_default(),
            recipe_id_2 = mission
                .recipe_ids
                .get(2)
                .map(ToString::to_string)
                .unwrap_or_default(),
            recipe_id_3 = mission
                .recipe_ids
                .get(3)
                .map(ToString::to_string)
                .unwrap_or_default(),
            recipe_id_4 = mission
                .recipe_ids
                .get(4)
                .map(ToString::to_string)
                .unwrap_or_default(),
        );
    }
}
