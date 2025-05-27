use crate::SheetData;

#[derive(Debug, Clone, Copy, Default)]
pub struct LevelAdjustTableEntry {
    pub level: u32,
    pub rlvl: u32,
}

impl SheetData for LevelAdjustTableEntry {
    const SHEET: &'static str = "GathererCrafterLvAdjustTable";
    const REQUIRED_FIELDS: &[&str] = &["RecipeLevel"];

    fn row_id(&self) -> u32 {
        self.level
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        Some(Self {
            level: value["row_id"].as_u32().unwrap(),
            rlvl: fields["RecipeLevel"]["value"].as_u32().unwrap(),
        })
    }
}

impl std::fmt::Display for LevelAdjustTableEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rlvl)?;
        Ok(())
    }
}
