use crate::SheetData;

#[derive(Debug)]
pub enum TemporaryAction {
    None,
    Unsupported,
    StellarSteadyHand,
}

impl From<u32> for TemporaryAction {
    fn from(id: u32) -> Self {
        match id {
            46843 => Self::StellarSteadyHand,
            0 => Self::None,
            _ => Self::Unsupported,
        }
    }
}

#[derive(Debug)]
pub struct StellarMission {
    pub id: u32,
    pub job_id: u16,
    pub temporary_action: TemporaryAction,
    pub temporary_action_charges: u8,
    pub recipe_ids: Vec<u32>,
}

impl StellarMission {
    fn stellar_steady_hand_charges(&self) -> u8 {
        match self.temporary_action {
            TemporaryAction::StellarSteadyHand => self.temporary_action_charges,
            _ => 0,
        }
    }
}

impl SheetData for StellarMission {
    const SHEET: &'static str = "WKSMissionUnit";
    const REQUIRED_FIELDS: &[&str] = &[
        "ClassJobCategory@as(raw)",
        "WKSMissionRecipe.Recipe@as(raw)",
        "MissionToDo[].TemporaryAction@as(raw)",
        "MissionToDo[].Unknown14@as(raw)",
    ];

    fn row_id(&self) -> u32 {
        self.id
    }

    fn from_json(value: &json::JsonValue) -> Option<Self> {
        let fields = &value["fields"];
        let job_ids: Vec<u16> = fields["ClassJobCategory@as(raw)"]
            .members()
            .filter_map(|class_job_category| {
                let id = class_job_category.as_u16()?;
                if (9..17).contains(&id) {
                    Some(id - 9)
                } else {
                    None
                }
            })
            .collect();
        assert!(job_ids.len() <= 1);

        // `MissionToDo` is an array. Currently only the first two elements are used and the second one only for the delivery step for Red Alert missions.
        let (temporary_action, temporary_action_charges) = fields["MissionToDo"]
            .members()
            .map(|member| {
                (
                    TemporaryAction::from(
                        member["fields"]["TemporaryAction@as(raw)"]
                            .as_u32()
                            .unwrap(),
                    ),
                    member["fields"]["Unknown14@as(raw)"].as_u8().unwrap(),
                )
            })
            .next()
            .unwrap();

        let recipe_ids = fields["WKSMissionRecipe"]["fields"]["Recipe@as(raw)"]
            .members()
            .filter_map(|recipe| recipe.as_u32().filter(|recipe_id| *recipe_id > 0))
            .collect();
        Some(Self {
            id: value["row_id"].as_u32().unwrap(),
            job_id: *job_ids.first()?,
            temporary_action,
            temporary_action_charges,
            recipe_ids,
        })
    }
}

impl std::fmt::Display for StellarMission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StellarMission {{ ")?;
        write!(f, "job_id: {}, ", self.job_id)?;
        write!(
            f,
            "stellar_steady_hand_charges: {}, ",
            self.stellar_steady_hand_charges()
        )?;
        write!(f, "recipe_ids: &[")?;
        for recipe_id in &self.recipe_ids {
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
                .replace('\u{AD}', "") // FR
                .replace('\u{A0}', " "), // No-break space used by FR, copied as normal space from in-game chat
        })
    }
}
