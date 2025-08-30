use crate::SheetData;

#[derive(Debug)]
pub struct StellarMission {
    pub id: u32,
    pub job_id: u32,
    pub recipe_ids: Vec<u32>,
}

impl SheetData for StellarMission {
    const SHEET: &'static str = "WKSMissionUnit";
    const REQUIRED_FIELDS: &[&str] = &["ClassJobCategory", "WKSMissionRecipe"];

    fn row_id(&self) -> u32 {
        self.id
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        let job_ids: Vec<u32> = fields["ClassJobCategory"]
            .members()
            .filter_map(|class_job_category| {
                let id = class_job_category["value"].as_u32()?;
                if id >= 9 && id < 17 {
                    Some(id - 9)
                } else {
                    None
                }
            })
            .collect();
        assert!(job_ids.len() <= 1);
        let recipe_ids = fields["WKSMissionRecipe"]["fields"]["Recipe"]
            .members()
            .filter_map(|recipe| recipe["value"].as_u32().filter(|recipe_id| *recipe_id > 0))
            .collect();
        Some(Self {
            id: value["row_id"].as_u32().unwrap(),
            job_id: *job_ids.first()?,
            recipe_ids,
        })
    }
}

impl std::fmt::Display for StellarMission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StellarMission {{ ")?;
        write!(f, "job_id: {}, ", self.job_id)?;
        write!(f, "recipe_ids: &[")?;
        for recipe_id in self.recipe_ids.iter() {
            write!(f, "{}, ", recipe_id)?;
        }
        write!(f, "], ")?;
        write!(f, "}}")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StellarMissionName {
    pub id: u32,
    pub name: String,
}

impl SheetData for StellarMissionName {
    const SHEET: &'static str = "WKSMissionUnit";
    const REQUIRED_FIELDS: &[&str] = &["Name"];

    fn row_id(&self) -> u32 {
        self.id
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        Some(Self {
            id: value["row_id"].as_u32().unwrap(),
            name: fields["Name"]
                .as_str()
                .unwrap()
                .replace('­', "") // FR
                .replace(' ', " "), // No-break space used by FR, copied as normal space from in-game chat
        })
    }
}
