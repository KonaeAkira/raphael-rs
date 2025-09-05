use crate::SheetData;

#[derive(Debug, Clone, Copy)]
pub struct Item {
    pub id: u32,
    pub item_level: u32,
    pub item_action: u32,
    pub can_be_hq: bool,
    pub always_collectable: bool,
}

impl SheetData for Item {
    const SHEET: &'static str = "Item";
    const REQUIRED_FIELDS: &[&str] = &[
        "LevelItem@as(raw)",
        "ItemAction@as(raw)",
        "CanBeHq",
        "AlwaysCollectable",
    ];

    fn row_id(&self) -> u32 {
        self.id
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        Some(Self {
            id: value["row_id"].as_u32().unwrap(),
            item_level: fields["LevelItem@as(raw)"].as_u32().unwrap(),
            item_action: fields["ItemAction@as(raw)"].as_u32().unwrap(),
            can_be_hq: fields["CanBeHq"].as_bool().unwrap(),
            always_collectable: fields["AlwaysCollectable"].as_bool().unwrap(),
        })
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item {{ ")?;
        write!(f, "item_level: {}, ", self.item_level)?;
        write!(f, "always_collectable: {}, ", self.always_collectable)?;
        write!(f, "}}")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ItemName {
    pub id: u32,
    pub name: String,
}

impl SheetData for ItemName {
    const SHEET: &'static str = "Item";
    const REQUIRED_FIELDS: &[&str] = &["Name"];

    fn row_id(&self) -> u32 {
        self.id
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        Some(Self {
            id: value["row_id"].as_u32().unwrap(),
            name: fields["Name"].as_str().unwrap().replace('\u{AD}', ""),
        })
    }
}
