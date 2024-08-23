use serde::{Deserialize, Serialize};

pub const MEALS: &[Consumable] = include!(concat!(env!("OUT_DIR"), "/meals.rs"));
pub const POTIONS: &[Consumable] = include!(concat!(env!("OUT_DIR"), "/potions.rs"));

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

impl Consumable {
    pub fn effect_string(self, craftsmanship: u16, control: u16, cp: u16) -> String {
        let mut effect: String = String::new();
        if self.craft_rel != 0 {
            effect.push_str(&format!(
                "Crafts. +{}% ({}), ",
                self.craft_rel,
                craftsmanship_bonus(craftsmanship, &[Some(self)])
            ));
        }
        if self.control_rel != 0 {
            effect.push_str(&format!(
                "Control +{}% ({}), ",
                self.control_rel,
                control_bonus(control, &[Some(self)])
            ));
        }
        if self.cp_rel != 0 {
            effect.push_str(&format!(
                "CP +{}% ({}), ",
                self.cp_rel,
                cp_bonus(cp, &[Some(self)])
            ));
        }
        effect.pop();
        effect.pop();
        effect
    }
}

pub fn craftsmanship_bonus(base: u16, consumables: &[Option<Consumable>]) -> u16 {
    consumables
        .iter()
        .map(|item| match item {
            Some(item) => {
                let rel_bonus = (base as u32 * item.craft_rel as u32 / 100) as u16;
                std::cmp::min(item.craft_max, rel_bonus)
            }
            None => 0,
        })
        .sum()
}

pub fn control_bonus(base: u16, consumables: &[Option<Consumable>]) -> u16 {
    consumables
        .iter()
        .map(|item| match item {
            Some(item) => {
                let rel_bonus = (base as u32 * item.control_rel as u32 / 100) as u16;
                std::cmp::min(item.control_max, rel_bonus)
            }
            None => 0,
        })
        .sum()
}

pub fn cp_bonus(base: u16, consumables: &[Option<Consumable>]) -> u16 {
    consumables
        .iter()
        .map(|item| match item {
            Some(item) => {
                let rel_bonus = (base as u32 * item.cp_rel as u32 / 100) as u16;
                std::cmp::min(item.cp_max, rel_bonus)
            }
            None => 0,
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use crate::{get_item_name, Locale};

    use super::*;

    fn find_consumable(item_name: &'static str) -> Option<Consumable> {
        MEALS
            .iter()
            .chain(POTIONS.iter())
            .find(|consumable| {
                get_item_name(consumable.item_id, consumable.hq, Locale::EN) == item_name
            })
            .copied()
    }

    #[test]
    fn test_u16_overflow() {
        let consumable = find_consumable("Rroneek Steak \u{e03c}").unwrap();
        // 13108 * 5 mod 1<<16 = 4
        // 2521 * 26 mod 1<<16 = 10
        assert_eq!(
            consumable.effect_string(4021, 13108, 2521),
            "Control +5% (97), CP +26% (92)"
        );
    }

    #[test]
    fn test_rroneek_steak_hq() {
        let consumable = find_consumable("Rroneek Steak \u{e03c}").unwrap();
        assert_eq!(
            consumable.effect_string(4021, 4023, 550),
            "Control +5% (97), CP +26% (92)"
        );
        assert_eq!(
            consumable.effect_string(1000, 1000, 100),
            "Control +5% (50), CP +26% (26)"
        );
    }
}
