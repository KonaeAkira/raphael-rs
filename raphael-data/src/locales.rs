use crate::ITEMS;
use raphael_sim::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Locale {
    EN,
    DE,
    FR,
    JP,
    KR,
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EN => write!(f, "EN"),
            Self::DE => write!(f, "DE"),
            Self::FR => write!(f, "FR"),
            Self::JP => write!(f, "JP"),
            Self::KR => write!(f, "KR"),
        }
    }
}

const JOB_NAMES_EN: [&str; 8] = ["CRP", "BSM", "ARM", "GSM", "LTW", "WVR", "ALC", "CUL"];
const JOB_NAMES_DE: [&str; 8] = ["ZMR", "GRS", "PLA", "GLD", "GER", "WEB", "ALC", "GRM"];
const JOB_NAMES_FR: [&str; 8] = ["MEN", "FRG", "ARM", "ORF", "TAN", "COU", "ALC", "CUI"];
const JOB_NAMES_KR: [&str; 8] = ["목수", "대장", "갑주", "보석", "가죽", "재봉", "연금", "요리"];

pub fn get_job_name(job_id: u8, locale: Locale) -> &'static str {
    match locale {
        Locale::EN => JOB_NAMES_EN[job_id as usize],
        Locale::DE => JOB_NAMES_DE[job_id as usize],
        Locale::FR => JOB_NAMES_FR[job_id as usize],
        Locale::JP => JOB_NAMES_EN[job_id as usize], // JP job abbreviations are the same as EN
        Locale::KR => JOB_NAMES_KR[job_id as usize],
    }
}

pub static ITEM_NAMES_EN: phf::Map<u32, &str> = include!("../data/item_names_en.rs");
pub static ITEM_NAMES_DE: phf::Map<u32, &str> = include!("../data/item_names_de.rs");
pub static ITEM_NAMES_FR: phf::Map<u32, &str> = include!("../data/item_names_fr.rs");
pub static ITEM_NAMES_JP: phf::Map<u32, &str> = include!("../data/item_names_jp.rs");
pub static ITEM_NAMES_KR: phf::Map<u32, &str> = include!("../data/item_names_kr.rs");

pub fn get_item_name(item_id: u32, hq: bool, locale: Locale) -> Option<String> {
    let item_name = match locale {
        Locale::EN => ITEM_NAMES_EN.get(&item_id)?.to_owned(),
        Locale::DE => ITEM_NAMES_DE.get(&item_id)?.to_owned(),
        Locale::FR => ITEM_NAMES_FR.get(&item_id)?.to_owned(),
        Locale::JP => ITEM_NAMES_JP.get(&item_id)?.to_owned(),
        Locale::KR => ITEM_NAMES_KR.get(&item_id)?.to_owned(),
    };
    let item_entry = ITEMS.get(&item_id);
    let always_collectable = item_entry.is_some_and(|item| item.always_collectable);
    if !always_collectable {
        match hq {
            true => Some(format!("{} \u{e03c}", item_name)),
            false => Some(item_name.to_string()),
        }
    } else {
        Some(format!("{} \u{e03d}", item_name))
    }
}

pub const fn action_name(action: Action, locale: Locale) -> &'static str {
    match locale {
        Locale::EN => action_name_en(action),
        Locale::DE => action_name_de(action),
        Locale::FR => action_name_fr(action),
        Locale::JP => action_name_jp(action),
        Locale::KR => action_name_kr(action),
    }
}

const fn action_name_en(action: Action) -> &'static str {
    match action {
        Action::BasicSynthesis => "Basic Synthesis",
        Action::BasicTouch => "Basic Touch",
        Action::MasterMend => "Master's Mend",
        Action::Observe => "Observe",
        Action::TricksOfTheTrade => "Tricks of the Trade",
        Action::WasteNot => "Waste Not",
        Action::Veneration => "Veneration",
        Action::StandardTouch => "Standard Touch",
        Action::GreatStrides => "Great Strides",
        Action::Innovation => "Innovation",
        Action::WasteNot2 => "Waste Not II",
        Action::ByregotsBlessing => "Byregot's Blessing",
        Action::PreciseTouch => "Precise Touch",
        Action::MuscleMemory => "Muscle Memory",
        Action::CarefulSynthesis => "Careful Synthesis",
        Action::Manipulation => "Manipulation",
        Action::PrudentTouch => "Prudent Touch",
        Action::AdvancedTouch => "Advanced Touch",
        Action::Reflect => "Reflect",
        Action::PreparatoryTouch => "Preparatory Touch",
        Action::Groundwork => "Groundwork",
        Action::DelicateSynthesis => "Delicate Synthesis",
        Action::IntensiveSynthesis => "Intensive Synthesis",
        Action::HeartAndSoul => "Heart and Soul",
        Action::PrudentSynthesis => "Prudent Synthesis",
        Action::TrainedFinesse => "Trained Finesse",
        Action::RefinedTouch => "Refined Touch",
        Action::ImmaculateMend => "Immaculate Mend",
        Action::TrainedPerfection => "Trained Perfection",
        Action::TrainedEye => "Trained Eye",
        Action::QuickInnovation => "Quick Innovation",
    }
}

const fn action_name_de(action: Action) -> &'static str {
    match action {
        Action::BasicSynthesis => "Bearbeiten",
        Action::BasicTouch => "Veredelung",
        Action::MasterMend => "Wiederherstellung",
        Action::Observe => "Beobachten",
        Action::TricksOfTheTrade => "Kunstgriff",
        Action::WasteNot => "Nachhaltigkeit",
        Action::Veneration => "Ehrfurcht",
        Action::StandardTouch => "Solide Veredelung",
        Action::GreatStrides => "Große Schritte",
        Action::Innovation => "Innovation",
        Action::WasteNot2 => "Nachhaltigkeit II",
        Action::ByregotsBlessing => "Byregots Benediktion",
        Action::PreciseTouch => "Präzise Veredelung",
        Action::MuscleMemory => "Motorisches Gedächtnis",
        Action::CarefulSynthesis => "Sorgfältige Bearbeitung",
        Action::Manipulation => "Manipulation",
        Action::PrudentTouch => "Nachhaltige Veredelung",
        Action::AdvancedTouch => "Höhere Veredelung",
        Action::Reflect => "Einkehr",
        Action::PreparatoryTouch => "Basisveredelung",
        Action::Groundwork => "Vorarbeit",
        Action::DelicateSynthesis => "Akribische Bearbeitung",
        Action::IntensiveSynthesis => "Fokussierte Bearbeitung",
        Action::HeartAndSoul => "Mit Leib und Seele",
        Action::PrudentSynthesis => "Rationelle Bearbeitung",
        Action::TrainedFinesse => "Götter Werk",
        Action::RefinedTouch => "Raffinierte Veredelung",
        Action::ImmaculateMend => "Winkelzug",
        Action::TrainedPerfection => "Meisters Beitrag",
        Action::TrainedEye => "Flinke Hand",
        Action::QuickInnovation => "Spontane Innovation",
    }
}

const fn action_name_fr(action: Action) -> &'static str {
    match action {
        Action::BasicSynthesis => "Travail de base",
        Action::BasicTouch => "Ouvrage de base",
        Action::MasterMend => "Réparation de maître",
        Action::Observe => "Observation",
        Action::TricksOfTheTrade => "Ficelles du métier",
        Action::WasteNot => "Parcimonie",
        Action::Veneration => "Vénération",
        Action::StandardTouch => "Ouvrage standard",
        Action::GreatStrides => "Grands progrès",
        Action::Innovation => "Innovation",
        Action::WasteNot2 => "Parcimonie pérenne",
        Action::ByregotsBlessing => "Bénédiction de Byregot",
        Action::PreciseTouch => "Ouvrage précis",
        Action::MuscleMemory => "Mémoire musculaire",
        Action::CarefulSynthesis => "Travail prudent",
        Action::Manipulation => "Manipulation",
        Action::PrudentTouch => "Ouvrage parcimonieux",
        Action::AdvancedTouch => "Ouvrage avancé",
        Action::Reflect => "Véritable valeur",
        Action::PreparatoryTouch => "Ouvrage préparatoire",
        Action::Groundwork => "Travail préparatoire",
        Action::DelicateSynthesis => "Travail minutieux",
        Action::IntensiveSynthesis => "Travail vigilant",
        Action::HeartAndSoul => "Attention totale",
        Action::PrudentSynthesis => "Travail économe",
        Action::TrainedFinesse => "Main divine",
        Action::RefinedTouch => "Ouvrage raffiné",
        Action::ImmaculateMend => "Réparation totale",
        Action::TrainedPerfection => "Main suprême",
        Action::TrainedEye => "Main preste",
        Action::QuickInnovation => "Innovation instantanée",
    }
}

const fn action_name_jp(action: Action) -> &'static str {
    match action {
        Action::BasicSynthesis => "作業",
        Action::BasicTouch => "加工",
        Action::MasterMend => "マスターズメンド",
        Action::Observe => "経過観察",
        Action::TricksOfTheTrade => "秘訣",
        Action::WasteNot => "倹約",
        Action::Veneration => "ヴェネレーション",
        Action::StandardTouch => "中級加工",
        Action::GreatStrides => "グレートストライド",
        Action::Innovation => "イノベーション",
        Action::WasteNot2 => "長期倹約",
        Action::ByregotsBlessing => "ビエルゴの祝福",
        Action::PreciseTouch => "集中加工",
        Action::MuscleMemory => "確信",
        Action::CarefulSynthesis => "模範作業",
        Action::Manipulation => "マニピュレーション",
        Action::PrudentTouch => "倹約加工",
        Action::AdvancedTouch => "上級加工",
        Action::Reflect => "真価",
        Action::PreparatoryTouch => "下地加工",
        Action::Groundwork => "下地作業",
        Action::DelicateSynthesis => "精密作業",
        Action::IntensiveSynthesis => "集中作業",
        Action::HeartAndSoul => "一心不乱",
        Action::PrudentSynthesis => "倹約作業",
        Action::TrainedFinesse => "匠の神業",
        Action::RefinedTouch => "洗練加工",
        Action::ImmaculateMend => "パーフェクトメンド",
        Action::TrainedPerfection => "匠の絶技",
        Action::TrainedEye => "匠の早業",
        Action::QuickInnovation => "クイックイノベーション",
    }
}

const fn action_name_kr(action: Action) -> &'static str {
    match action {
        Action::BasicSynthesis => "작업",
        Action::BasicTouch => "가공",
        Action::MasterMend => "능숙한 땜질",
        Action::Observe => "경과 관찰",
        Action::TricksOfTheTrade => "비결",
        Action::WasteNot => "근검절약",
        Action::Veneration => "공경",
        Action::StandardTouch => "중급 가공",
        Action::GreatStrides => "장족의 발전",
        Action::Innovation => "혁신",
        Action::WasteNot2 => "장기 절약",
        Action::ByregotsBlessing => "비레고의 축복",
        Action::PreciseTouch => "집중 가공",
        Action::MuscleMemory => "확신",
        Action::CarefulSynthesis => "모범 작업",
        Action::Manipulation => "교묘한 손놀림",
        Action::PrudentTouch => "절약 가공",
        Action::AdvancedTouch => "상급 가공",
        Action::Reflect => "진가",
        Action::PreparatoryTouch => "밑가공",
        Action::Groundwork => "밑작업",
        Action::DelicateSynthesis => "정밀 작업",
        Action::IntensiveSynthesis => "집중 작업",
        Action::HeartAndSoul => "일심불란",
        Action::PrudentSynthesis => "절약 작업",
        Action::TrainedFinesse => "장인의 황금손",
        Action::RefinedTouch => "세련 가공",
        Action::ImmaculateMend => "완벽한 땜질",
        Action::TrainedPerfection => "장인의 초절 기술",
        Action::TrainedEye => "장인의 날랜손",
        Action::QuickInnovation => "신속한 혁신",
    }
}
