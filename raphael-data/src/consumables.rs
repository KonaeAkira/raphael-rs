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

#[cfg(test)]
mod tests {
    use crate::{Locale, get_item_name};

    use super::*;

    fn find_consumable(item_name: &'static str) -> Option<Consumable> {
        MEALS
            .iter()
            .chain(POTIONS.iter())
            .find(
                |consumable| match get_item_name(consumable.item_id, consumable.hq, Locale::EN) {
                    None => false,
                    Some(name) => name == item_name,
                },
            )
            .copied()
    }

    #[test]
    fn test_u16_overflow() {
        let consumable = find_consumable("Rroneek Steak \u{e03c}").unwrap();
        // 13108 * 5 mod 1<<16 = 4
        // 2521 * 26 mod 1<<16 = 10
        assert_eq!(
            stat_bonuses([4021, 13108, 2521], &[Some(consumable)]),
            [0, 97, 92]
        );
    }

    #[test]
    fn test_rroneek_steak_hq() {
        let consumable = find_consumable("Rroneek Steak \u{e03c}").unwrap();
        assert_eq!(
            stat_bonuses([4021, 4032, 550], &[Some(consumable)]),
            [0, 97, 92]
        );
        assert_eq!(
            stat_bonuses([1000, 1000, 100], &[Some(consumable)]),
            [0, 50, 26]
        );
    }
}
