pub const MEALS: &[Consumable] = include!(concat!(env!("OUT_DIR"), "/meals.rs"));
pub const POTIONS: &[Consumable] = include!(concat!(env!("OUT_DIR"), "/potions.rs"));

#[derive(Debug, Clone, Copy)]
pub struct Consumable {
    pub item_level: u32,
    pub name: &'static str,
    pub craft_rel: u32,
    pub craft_max: u32,
    pub control_rel: u32,
    pub control_max: u32,
    pub cp_rel: u32,
    pub cp_max: u32,
}

impl Consumable {
    pub fn effect_string(self, craftsmanship: u32, control: u32, cp: u32) -> String {
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

pub fn craftsmanship_bonus(base: u32, consumables: &[Option<Consumable>]) -> u32 {
    consumables
        .iter()
        .map(|item| match item {
            Some(item) => std::cmp::min(item.craft_max, base * item.craft_rel / 100),
            None => 0,
        })
        .sum()
}

pub fn control_bonus(base: u32, consumables: &[Option<Consumable>]) -> u32 {
    consumables
        .iter()
        .map(|item| match item {
            Some(item) => std::cmp::min(item.control_max, base * item.control_rel / 100),
            None => 0,
        })
        .sum()
}

pub fn cp_bonus(base: u32, consumables: &[Option<Consumable>]) -> u32 {
    consumables
        .iter()
        .map(|item| match item {
            Some(item) => std::cmp::min(item.cp_max, base * item.cp_rel / 100),
            None => 0,
        })
        .sum()
}
