use std::collections::HashMap;

use crate::{Item, SheetData};

#[derive(Debug, Clone, Copy)]
pub struct ItemAction {
    pub id: u32,
    pub type_id: u32,
    pub item_food_id: u32,
}

impl SheetData for ItemAction {
    const SHEET: &'static str = "ItemAction";
    const REQUIRED_FIELDS: &[&str] = &["Type", "Data"];

    fn row_id(&self) -> u32 {
        self.id
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        Some(Self {
            id: value["row_id"].as_u32().unwrap(),
            type_id: fields["Type"].as_u32().unwrap(),
            item_food_id: fields["Data"].members().nth(1).unwrap().as_u32().unwrap(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ItemFood {
    pub id: u32,
    pub is_relative: Vec<bool>,
    pub param: Vec<u32>,
    pub max: Vec<i32>,
    pub max_hq: Vec<i32>,
    pub value: Vec<i32>,
    pub value_hq: Vec<i32>,
}

impl SheetData for ItemFood {
    const SHEET: &'static str = "ItemFood";
    const REQUIRED_FIELDS: &[&str] = &[
        "IsRelative",
        "BaseParam",
        "Max",
        "MaxHQ",
        "Value",
        "ValueHQ",
    ];

    fn row_id(&self) -> u32 {
        self.id
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        Some(Self {
            id: value["row_id"].as_u32().unwrap(),
            is_relative: fields["IsRelative"]
                .members()
                .map(|value| value.as_bool().unwrap())
                .collect(),
            param: fields["BaseParam"]
                .members()
                .map(|value| value["value"].as_u32().unwrap())
                .collect(),
            max: fields["Max"]
                .members()
                .map(|value| value.as_i32().unwrap())
                .collect(),
            max_hq: fields["MaxHQ"]
                .members()
                .map(|value| value.as_i32().unwrap())
                .collect(),
            value: fields["Value"]
                .members()
                .map(|value| value.as_i32().unwrap())
                .collect(),
            value_hq: fields["ValueHQ"]
                .members()
                .map(|value| value.as_i32().unwrap())
                .collect(),
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Consumable {
    pub item_id: u32,
    pub item_level: u32,
    pub hq: bool,
    pub craft_rel: i32,
    pub craft_max: i32,
    pub control_rel: i32,
    pub control_max: i32,
    pub cp_rel: i32,
    pub cp_max: i32,
}

impl std::fmt::Display for Consumable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Consumable {{ ")?;
        write!(f, "item_id: {}, ", self.item_id)?;
        write!(f, "item_level: {}, ", self.item_level)?;
        write!(f, "hq: {}, ", self.hq)?;
        write!(f, "craft_rel: {}, ", self.craft_rel)?;
        write!(f, "craft_max: {}, ", self.craft_max)?;
        write!(f, "control_rel: {}, ", self.control_rel)?;
        write!(f, "control_max: {}, ", self.control_max)?;
        write!(f, "cp_rel: {}, ", self.cp_rel)?;
        write!(f, "cp_max: {}, ", self.cp_max)?;
        write!(f, "}}")?;
        Ok(())
    }
}

pub fn instantiate_consumables(
    items: &[Item],
    item_actions: Vec<ItemAction>,
    item_foods: Vec<ItemFood>,
) -> (Vec<Consumable>, Vec<Consumable>) {
    let item_actions: HashMap<u32, ItemAction> = item_actions
        .into_iter()
        .filter(|item_action| VALID_ITEM_ACTION_TYPE_IDS.contains(&item_action.type_id))
        .map(|item_action| (item_action.id, item_action))
        .collect();
    let item_foods: HashMap<u32, ItemFood> = item_foods
        .into_iter()
        .map(|item_food| (item_food.id, item_food))
        .collect();
    let mut meals = Vec::new();
    let mut potions = Vec::new();
    for item in items.iter().rev() {
        if let Some(item_action) = item_actions.get(&item.item_action) {
            let consumables = if item_action.type_id == ITEM_ACTION_DOH_POTION_TYPE_ID {
                &mut potions
            } else {
                &mut meals
            };
            let item_food = item_foods.get(&item_action.item_food_id).unwrap();
            if let Some(consumable) = try_instantiate_consumable(item, item_food, true) {
                consumables.push(consumable);
            }
            if let Some(consumable) = try_instantiate_consumable(item, item_food, false) {
                consumables.push(consumable);
            }
        }
    }
    (meals, potions)
}

fn try_instantiate_consumable(item: &Item, item_food: &ItemFood, hq: bool) -> Option<Consumable> {
    let mut consumable = Consumable {
        item_id: item.id,
        item_level: item.item_level,
        hq,
        ..Consumable::default()
    };
    if !item_food
        .param
        .iter()
        .any(|param| *param == CRAFTSMANSHIP_PARAM || *param == CONTROL_PARAM || *param == CP_PARAM)
    {
        return None;
    }
    let value_iter = if hq {
        item_food.value_hq.iter().zip(item_food.max_hq.iter())
    } else {
        item_food.value.iter().zip(item_food.max.iter())
    };
    for (param, (value, max)) in item_food.param.iter().zip(value_iter) {
        match *param {
            CRAFTSMANSHIP_PARAM => {
                consumable.craft_rel = *value;
                consumable.craft_max = *max;
            }
            CONTROL_PARAM => {
                consumable.control_rel = *value;
                consumable.control_max = *max;
            }
            CP_PARAM => {
                consumable.cp_rel = *value;
                consumable.cp_max = *max;
            }
            _ => (),
        }
    }
    Some(consumable)
}

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
const CRAFTSMANSHIP_PARAM: u32 = 70;
const CONTROL_PARAM: u32 = 71;
const CP_PARAM: u32 = 11;
