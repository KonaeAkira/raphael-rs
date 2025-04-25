use crate::SheetData;

#[derive(Debug, Clone, Copy, Default)]
pub struct RecipeLevel {
    pub rlvl: u32,
    pub job_level: u32,
    pub max_progress: u32,
    pub max_quality: u32,
    pub max_durability: u32,
    pub progress_div: u32,
    pub quality_div: u32,
    pub progress_mod: u32,
    pub quality_mod: u32,
}

impl SheetData for RecipeLevel {
    const SHEET: &'static str = "RecipeLevelTable";
    const REQUIRED_FIELDS: &[&str] = &[
        "ClassJobLevel",
        "Difficulty",
        "Quality",
        "Durability",
        "ProgressDivider",
        "QualityDivider",
        "ProgressModifier",
        "QualityModifier",
    ];

    fn row_id(&self) -> u32 {
        self.rlvl
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        Some(Self {
            rlvl: value["row_id"].as_u32().unwrap(),
            job_level: fields["ClassJobLevel"].as_u32().unwrap(),
            max_progress: fields["Difficulty"].as_u32().unwrap(),
            max_quality: fields["Quality"].as_u32().unwrap(),
            max_durability: fields["Durability"].as_u32().unwrap(),
            progress_div: fields["ProgressDivider"].as_u32().unwrap(),
            quality_div: fields["QualityDivider"].as_u32().unwrap(),
            progress_mod: fields["ProgressModifier"].as_u32().unwrap(),
            quality_mod: fields["QualityModifier"].as_u32().unwrap(),
        })
    }
}

impl std::fmt::Display for RecipeLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RecipeLevel {{ ")?;
        write!(f, "job_level: {}, ", self.job_level)?;
        write!(f, "max_progress: {}, ", self.max_progress)?;
        write!(f, "max_quality: {}, ", self.max_quality)?;
        write!(f, "max_durability: {}, ", self.max_durability)?;
        write!(f, "progress_div: {}, ", self.progress_div)?;
        write!(f, "quality_div: {}, ", self.quality_div)?;
        write!(f, "progress_mod: {}, ", self.progress_mod)?;
        write!(f, "quality_mod: {}, ", self.quality_mod)?;
        write!(f, "}}")?;
        Ok(())
    }
}
