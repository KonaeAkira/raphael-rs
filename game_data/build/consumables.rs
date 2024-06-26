use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::records::{ItemActionRecord, ItemFoodRecord, ItemRecord};
use crate::utils::read_csv_data;

// https://github.com/xivapi/ffxiv-datamining/blob/35e435494317723be856f18fb3b48f526316656e/docs/ItemActions.md#845
const ITEM_ACTION_BATTLE_FOOD_TYPE_ID: u32 = 844;
const ITEM_ACTION_DOH_FOOD_TYPE_ID: u32 = 845;
const ITEM_ACTION_DOH_POTION_TYPE_ID: u32 = 846;
const VALID_ITEM_ACTION_TYPE_IDS: &[u32] = &[
    ITEM_ACTION_BATTLE_FOOD_TYPE_ID,
    ITEM_ACTION_DOH_FOOD_TYPE_ID,
    ITEM_ACTION_DOH_POTION_TYPE_ID,
];

// https://github.com/xivapi/ffxiv-datamining/blob/35e435494317723be856f18fb3b48f526316656e/csv/BaseParam.csv
const CRAFTSMANSHIP_PARAM_ID: u32 = 70;
const CONTROL_PARAM_ID: u32 = 71;
const CP_PARAM_ID: u32 = 11;
const VALID_PARAMS: &[u32] = &[CRAFTSMANSHIP_PARAM_ID, CONTROL_PARAM_ID, CP_PARAM_ID];

pub fn import_consumable_records() -> Result<(), Box<dyn std::error::Error>> {
    let mut relevant_items = HashMap::new();

    let mut item_food_by_id = HashMap::new();

    for item_food in read_csv_data::<ItemFoodRecord>("data/ItemFood.csv") {
        item_food_by_id.insert(item_food.id, item_food);
    }

    let mut consumable_by_item_action_id = HashMap::new();
    for item_action in read_csv_data::<ItemActionRecord>("data/ItemAction.csv") {
        if !VALID_ITEM_ACTION_TYPE_IDS.contains(&item_action.type_id) {
            continue;
        }

        let is_potion = item_action.type_id == ITEM_ACTION_DOH_POTION_TYPE_ID;
        let item_food = *item_food_by_id.get(&item_action.data_1).unwrap();

        let params: &[u32] = &[item_food.param_0, item_food.param_1, item_food.param_2];
        if params.iter().any(|param| VALID_PARAMS.contains(param)) {
            consumable_by_item_action_id.insert(
                item_action.id,
                Consumable {
                    is_potion,
                    item_food,
                },
            );
        }
    }

    let mut meals = vec![];
    let mut potions = vec![];
    for item in read_csv_data::<ItemRecord>("data/Item.csv") {
        if let Some(consumable) = consumable_by_item_action_id.get(&item.item_action) {
            relevant_items.insert(item.id, item.name.clone());

            let Consumable {
                is_potion,
                item_food,
            } = *consumable;

            let (craftsmanship, control, cp) = get_stats(item_food);

            let consumable = ConsumableOutput {
                item_level: item.item_level,
                name: item.name,
                craftsmanship,
                control,
                cp,
            };

            if is_potion {
                potions.push(consumable);
            } else {
                meals.push(consumable);
            }
        }
    }

    // sort by item level decreasing
    meals.sort_by(|a, b| b.item_level.cmp(&a.item_level));
    let mut writer = BufWriter::new(
        File::create(Path::new(&std::env::var("OUT_DIR")?).join("meals.rs")).unwrap(),
    );
    writeln!(writer, "&[")?;
    for meal in meals.into_iter() {
        writeln!(writer, "{},", meal.export(true))?;
        writeln!(writer, "{},", meal.export(false))?;
    }
    writeln!(writer, "]")?;

    // sort by item level decreasing
    potions.sort_by(|a, b| b.item_level.cmp(&a.item_level));
    let mut writer = BufWriter::new(
        File::create(Path::new(&std::env::var("OUT_DIR")?).join("potions.rs")).unwrap(),
    );
    writeln!(writer, "&[")?;
    for potion in potions.into_iter() {
        writeln!(writer, "{},", potion.export(true))?;
        writeln!(writer, "{},", potion.export(false))?;
    }
    writeln!(writer, "]")?;

    Ok(())
}

#[allow(clippy::type_complexity)]
#[rustfmt::skip]
fn get_stats(item_food: ItemFoodRecord) -> ([u32; 4], [u32; 4], [u32; 4]) {
    let ItemFoodRecord {
        id: _,
        param_0, param_0_relative, param_0_value, param_0_max, param_0_hq_value, param_0_hq_max,
        param_1, param_1_relative, param_1_value, param_1_max, param_1_hq_value, param_1_hq_max,
        param_2, param_2_relative, param_2_value, param_2_max, param_2_hq_value, param_2_hq_max,
    } = item_food;

    let mut craftsmanship = [0; 4];
    let mut control = [0; 4];
    let mut cp = [0; 4];

    for (param, is_relative, value, max, hq_value, hq_max) in [
        (param_0, param_0_relative, param_0_value, param_0_max, param_0_hq_value, param_0_hq_max),
        (param_1, param_1_relative, param_1_value, param_1_max, param_1_hq_value, param_1_hq_max),
        (param_2, param_2_relative, param_2_value, param_2_max, param_2_hq_value, param_2_hq_max),
    ] {
        let values: [u32; 4] = [value as u32, max, hq_value as u32, hq_max];

        if param == CRAFTSMANSHIP_PARAM_ID {
            assert!(is_relative);
            assert!(values.iter().all(|&v| v > 0));
            craftsmanship = values;
        } else if param == CONTROL_PARAM_ID {
            assert!(is_relative);
            assert!(values.iter().all(|&v| v > 0));
            control = values;
        } else if param == CP_PARAM_ID {
            assert!(is_relative);
            assert!(values.iter().all(|&v| v > 0));
            cp = values;
        }
    }

    (craftsmanship, control, cp)
}

struct Consumable {
    is_potion: bool,
    item_food: ItemFoodRecord,
}

#[derive(Debug)]
struct ConsumableOutput {
    item_level: u32,
    name: String,
    craftsmanship: [u32; 4],
    control: [u32; 4],
    cp: [u32; 4],
}

impl ConsumableOutput {
    fn export(&self, hq: bool) -> String {
        match hq {
            true => format!("Consumable {{ item_level: {}, name: \"{} (HQ)\", craft_rel: {}, craft_max: {}, control_rel: {}, control_max: {}, cp_rel: {}, cp_max: {} }}", self.item_level, self.name, self.craftsmanship[2], self.craftsmanship[3], self.control[2], self.control[3], self.cp[2], self.cp[3]),
            false => format!("Consumable {{ item_level: {}, name: \"{}\", craft_rel: {}, craft_max: {}, control_rel: {}, control_max: {}, cp_rel: {}, cp_max: {} }}", self.item_level, self.name, self.craftsmanship[0], self.craftsmanship[1], self.control[0], self.control[1], self.cp[0], self.cp[1])
        }
    }
}
