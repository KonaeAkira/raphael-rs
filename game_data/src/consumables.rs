pub const MEALS: &[Consumable] = include!(concat!(env!("OUT_DIR"), "/meals.rs"));
pub const POTIONS: &[Consumable] = include!(concat!(env!("OUT_DIR"), "/potions.rs"));

#[derive(Debug, Clone, Copy)]
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
            Some(item) => std::cmp::min(item.craft_max, base * item.craft_rel / 100),
            None => 0,
        })
        .sum()
}

pub fn control_bonus(base: u16, consumables: &[Option<Consumable>]) -> u16 {
    consumables
        .iter()
        .map(|item| match item {
            Some(item) => std::cmp::min(item.control_max, base * item.control_rel / 100),
            None => 0,
        })
        .sum()
}

pub fn cp_bonus(base: u16, consumables: &[Option<Consumable>]) -> u16 {
    consumables
        .iter()
        .map(|item| match item {
            Some(item) => std::cmp::min(item.cp_max, base * item.cp_rel / 100),
            None => 0,
        })
        .sum()
}
