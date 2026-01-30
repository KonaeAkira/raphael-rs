use crate::{CL_ICON_CHAR, HQ_ICON_CHAR, ITEMS, NciArray, nci_array};
use raphael_sim::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Locale {
    EN,
    DE,
    FR,
    JP,
    CN,
    KR,
    TW,
}

impl Locale {
    pub fn short_code(self) -> &'static str {
        match self {
            Self::EN => "EN",
            Self::DE => "DE",
            Self::FR => "FR",
            Self::JP => "JP",
            Self::CN => "CN",
            Self::KR => "KR",
            Self::TW => "TW",
        }
    }
}

pub const JOB_NAMES_EN: [&str; 8] = ["CRP", "BSM", "ARM", "GSM", "LTW", "WVR", "ALC", "CUL"];
pub const JOB_NAMES_DE: [&str; 8] = ["ZMR", "GRS", "PLA", "GLD", "GER", "WEB", "ALC", "GRM"];
pub const JOB_NAMES_FR: [&str; 8] = ["MEN", "FRG", "ARM", "ORF", "TAN", "COU", "ALC", "CUI"];
pub const JOB_NAMES_JP: [&str; 8] = [
    "木工師",
    "鍛冶師",
    "甲冑師",
    "彫金師",
    "革細工師",
    "裁縫師",
    "錬金術師",
    "調理師",
];
pub const JOB_NAMES_CN: [&str; 8] = [
    "刻木", "锻铁", "铸甲", "雕金", "制革", "裁衣", "炼金", "烹调",
];
pub const JOB_NAMES_KR: [&str; 8] = [
    "목수", "대장", "갑주", "보석", "가죽", "재봉", "연금", "요리",
];
pub const JOB_NAMES_TW: [&str; 8] = [
    "木工", "鍛造", "甲冑", "金工", "皮革", "裁縫", "鍊金", "烹調",
];

pub fn get_job_name(job_id: u8, locale: Locale) -> &'static str {
    match locale {
        Locale::EN => JOB_NAMES_EN[job_id as usize],
        Locale::DE => JOB_NAMES_DE[job_id as usize],
        Locale::FR => JOB_NAMES_FR[job_id as usize],
        Locale::JP => JOB_NAMES_JP[job_id as usize],
        Locale::CN => JOB_NAMES_CN[job_id as usize],
        Locale::KR => JOB_NAMES_KR[job_id as usize],
        Locale::TW => JOB_NAMES_TW[job_id as usize],
    }
}

pub const ITEM_NAMES_EN: NciArray<u32, &str> = include!("../data/item_names_en.rs");
pub const ITEM_NAMES_DE: NciArray<u32, &str> = include!("../data/item_names_de.rs");
pub const ITEM_NAMES_FR: NciArray<u32, &str> = include!("../data/item_names_fr.rs");
pub const ITEM_NAMES_JP: NciArray<u32, &str> = include!("../data/item_names_jp.rs");
pub const ITEM_NAMES_CN: NciArray<u32, &str> = include!("../data/item_names_cn.rs");
pub const ITEM_NAMES_KR: NciArray<u32, &str> = include!("../data/item_names_kr.rs");
pub const ITEM_NAMES_TW: NciArray<u32, &str> = include!("../data/item_names_tw.rs");

pub fn get_raw_item_name(item_id: u32, locale: Locale) -> Option<&'static str> {
    match locale {
        Locale::EN => ITEM_NAMES_EN.get(item_id).copied(),
        Locale::DE => ITEM_NAMES_DE.get(item_id).copied(),
        Locale::FR => ITEM_NAMES_FR.get(item_id).copied(),
        Locale::JP => ITEM_NAMES_JP.get(item_id).copied(),
        Locale::CN => ITEM_NAMES_CN.get(item_id).copied(),
        Locale::KR => ITEM_NAMES_KR.get(item_id).copied(),
        Locale::TW => ITEM_NAMES_TW.get(item_id).copied(),
    }
}

pub fn get_item_name(item_id: u32, hq: bool, locale: Locale) -> Option<String> {
    let raw_item_name = get_raw_item_name(item_id, locale)?;
    if ITEMS.get(item_id)?.always_collectable {
        Some(format!("{} {}", raw_item_name, CL_ICON_CHAR))
    } else if hq {
        Some(format!("{} {}", raw_item_name, HQ_ICON_CHAR))
    } else {
        Some(raw_item_name.into())
    }
}

pub static STELLAR_MISSION_NAMES_EN: NciArray<u32, &str> =
    include!("../data/stellar_mission_names_en.rs");
pub static STELLAR_MISSION_NAMES_DE: NciArray<u32, &str> =
    include!("../data/stellar_mission_names_de.rs");
pub static STELLAR_MISSION_NAMES_FR: NciArray<u32, &str> =
    include!("../data/stellar_mission_names_fr.rs");
pub static STELLAR_MISSION_NAMES_JP: NciArray<u32, &str> =
    include!("../data/stellar_mission_names_jp.rs");
pub static STELLAR_MISSION_NAMES_CN: NciArray<u32, &str> =
    include!("../data/stellar_mission_names_cn.rs");
pub static STELLAR_MISSION_NAMES_KR: NciArray<u32, &str> =
    include!("../data/stellar_mission_names_kr.rs");
pub static STELLAR_MISSION_NAMES_TW: NciArray<u32, &str> =
    include!("../data/stellar_mission_names_tw.rs");

pub fn get_stellar_mission_name(mission_id: u32, locale: Locale) -> Option<&'static str> {
    match locale {
        Locale::EN => STELLAR_MISSION_NAMES_EN.get(mission_id).copied(),
        Locale::DE => STELLAR_MISSION_NAMES_DE.get(mission_id).copied(),
        Locale::FR => STELLAR_MISSION_NAMES_FR.get(mission_id).copied(),
        Locale::JP => STELLAR_MISSION_NAMES_JP.get(mission_id).copied(),
        Locale::CN => STELLAR_MISSION_NAMES_CN.get(mission_id).copied(),
        Locale::KR => STELLAR_MISSION_NAMES_KR.get(mission_id).copied(),
        Locale::TW => STELLAR_MISSION_NAMES_TW.get(mission_id).copied(),
    }
}

pub const fn action_name(action: Action, locale: Locale) -> &'static str {
    match locale {
        Locale::EN => action_name_en(action),
        Locale::DE => action_name_de(action),
        Locale::FR => action_name_fr(action),
        Locale::JP => action_name_jp(action),
        Locale::CN => action_name_cn(action),
        Locale::KR => action_name_kr(action),
        Locale::TW => action_name_tw(action),
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
        Action::StellarSteadyHand => "Stellar Steady Hand",
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
        Action::StellarSteadyHand => todo!(),
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
        Action::StellarSteadyHand => todo!(),
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
        Action::StellarSteadyHand => todo!(),
    }
}

const fn action_name_cn(action: Action) -> &'static str {
    match action {
        Action::BasicSynthesis => "制作",
        Action::BasicTouch => "加工",
        Action::MasterMend => "精修",
        Action::Observe => "观察",
        Action::TricksOfTheTrade => "秘诀",
        Action::WasteNot => "俭约",
        Action::Veneration => "崇敬",
        Action::StandardTouch => "中级加工",
        Action::GreatStrides => "阔步",
        Action::Innovation => "改革",
        Action::WasteNot2 => "长期俭约",
        Action::ByregotsBlessing => "比尔格的祝福",
        Action::PreciseTouch => "集中加工",
        Action::MuscleMemory => "坚信",
        Action::CarefulSynthesis => "模范制作",
        Action::Manipulation => "掌握",
        Action::PrudentTouch => "俭约加工",
        Action::AdvancedTouch => "上级加工",
        Action::Reflect => "闲静",
        Action::PreparatoryTouch => "坯料加工",
        Action::Groundwork => "坯料制作",
        Action::DelicateSynthesis => "精密制作",
        Action::IntensiveSynthesis => "集中制作",
        Action::HeartAndSoul => "专心致志",
        Action::PrudentSynthesis => "俭约制作",
        Action::TrainedFinesse => "工匠的神技",
        Action::RefinedTouch => "精炼加工",
        Action::ImmaculateMend => "巧夺天工",
        Action::TrainedPerfection => "工匠的绝技",
        Action::TrainedEye => "工匠的神速技巧",
        Action::QuickInnovation => "快速改革",
        Action::StellarSteadyHand => todo!(),
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
        Action::StellarSteadyHand => todo!(),
    }
}

const fn action_name_tw(action: Action) -> &'static str {
    match action {
        Action::BasicSynthesis => "製作",
        Action::BasicTouch => "加工",
        Action::MasterMend => "精修",
        Action::Observe => "觀察",
        Action::TricksOfTheTrade => "秘訣",
        Action::WasteNot => "儉約",
        Action::Veneration => "崇敬",
        Action::StandardTouch => "中級加工",
        Action::GreatStrides => "闊步",
        Action::Innovation => "改革",
        Action::WasteNot2 => "長期儉約",
        Action::ByregotsBlessing => "比爾格的祝福",
        Action::PreciseTouch => "集中加工",
        Action::MuscleMemory => "堅信",
        Action::CarefulSynthesis => "模範製作",
        Action::Manipulation => "掌握",
        Action::PrudentTouch => "儉約加工",
        Action::AdvancedTouch => "上級加工",
        Action::Reflect => "閒靜",
        Action::PreparatoryTouch => "坯料加工",
        Action::Groundwork => "坯料製作",
        Action::DelicateSynthesis => "精密製作",
        Action::IntensiveSynthesis => "集中製作",
        Action::HeartAndSoul => "專心致志",
        Action::PrudentSynthesis => "儉約製作",
        Action::TrainedFinesse => "工匠的神技",
        Action::RefinedTouch => "精煉加工",
        Action::ImmaculateMend => "巧奪天工",
        Action::TrainedPerfection => "工匠的絕技",
        Action::TrainedEye => "工匠的神速技巧",
        Action::QuickInnovation => "快速改革",
        Action::StellarSteadyHand => todo!(),
    }
}
