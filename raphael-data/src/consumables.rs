pub const MEALS: &[Consumable] = include!("../data/meals.rs");
pub const POTIONS: &[Consumable] = include!("../data/potions.rs");

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Consumable {
    pub item_id: u32,
    pub item_level: u32,
    pub hq: bool,
    pub craft_rel: u16,
    pub craft_max: u16,
    pub control_rel: u16,
    pub control_max: u16,
    pub cp_rel: u16,
    pub cp_max: u16,
}

pub fn craftsmanship_bonus(base: u16, consumables: &[Option<Consumable>]) -> u16 {
    consumables
        .iter()
        .flatten()
        .map(|item| {
            let rel_bonus = (base as u32 * item.craft_rel as u32 / 100) as u16;
            std::cmp::min(item.craft_max, rel_bonus)
        })
        .sum()
}

pub fn control_bonus(base: u16, consumables: &[Option<Consumable>]) -> u16 {
    consumables
        .iter()
        .flatten()
        .map(|item| {
            let rel_bonus = (base as u32 * item.control_rel as u32 / 100) as u16;
            std::cmp::min(item.control_max, rel_bonus)
        })
        .sum()
}

pub fn cp_bonus(base: u16, consumables: &[Option<Consumable>]) -> u16 {
    consumables
        .iter()
        .flatten()
        .map(|item| {
            let rel_bonus = (base as u32 * item.cp_rel as u32 / 100) as u16;
            std::cmp::min(item.cp_max, rel_bonus)
        })
        .sum()
}

pub fn stat_bonuses(base_stats: [u16; 3], consumables: &[Option<Consumable>]) -> [u16; 3] {
    consumables
        .iter()
        .flatten()
        .map(|item| {
            let craft_rel_bonus = (base_stats[0] as u32 * item.craft_rel as u32 / 100) as u16;
            let craft_bonus = std::cmp::min(item.craft_max, craft_rel_bonus);
            let control_rel_bonus = (base_stats[1] as u32 * item.control_rel as u32 / 100) as u16;
            let control_bonus = std::cmp::min(item.control_max, control_rel_bonus);
            let cp_rel_bonus = (base_stats[2] as u32 * item.cp_rel as u32 / 100) as u16;
            let cp_bonus = std::cmp::min(item.cp_max, cp_rel_bonus);
            [craft_bonus, control_bonus, cp_bonus]
        })
        .fold([0, 0, 0], |acc, bonuses| {
            [
                acc[0] + bonuses[0],
                acc[1] + bonuses[1],
                acc[2] + bonuses[2],
            ]
        })
}

pub fn craftsmanship_unbuffed(buffed: u16, consumables: &[Option<Consumable>]) -> Option<u16> {
    let max_total = consumables
        .iter()
        .flatten()
        .map(|item| item.craft_max)
        .sum();

    (buffed.saturating_sub(max_total)..=buffed)
        .find(|&prediction| prediction + craftsmanship_bonus(prediction, consumables) == buffed)
}

pub fn control_unbuffed(buffed: u16, consumables: &[Option<Consumable>]) -> Option<u16> {
    let max_total = consumables
        .iter()
        .flatten()
        .map(|item| item.control_max)
        .sum();

    (buffed.saturating_sub(max_total)..=buffed)
        .find(|&prediction| prediction + control_bonus(prediction, consumables) == buffed)
}

pub fn cp_unbuffed(buffed: u16, consumables: &[Option<Consumable>]) -> Option<u16> {
    let max_total = consumables.iter().flatten().map(|item| item.cp_max).sum();

    (buffed.saturating_sub(max_total)..=buffed)
        .find(|&prediction| prediction + cp_bonus(prediction, consumables) == buffed)
}
