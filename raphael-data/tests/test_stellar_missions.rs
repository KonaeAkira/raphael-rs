use expect_test::expect;
use raphael_data::*;

#[derive(Debug, Clone)]
#[allow(dead_code)] // False positive (used by expect_test::expect)
struct DetailedItemInfo {
    item_id: u32,
    item_name: String,
}

impl DetailedItemInfo {
    pub fn from_item_id(item_id: u32) -> Self {
        Self {
            item_id,
            item_name: get_item_name(item_id, false, Locale::EN).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // False positive (used by expect_test::expect)
struct DetailedRecipeInfo {
    recipe_id: u32,
    resulting_item: DetailedItemInfo,
    progress: u32,
    quality: u32,
    durability: u16,
}

impl DetailedRecipeInfo {
    pub fn from_recipe_id(recipe_id: u32) -> Self {
        let recipe = RECIPES.get(recipe_id).unwrap();
        let rlvl_entry = RLVLS[usize::from(recipe.recipe_level)];
        Self {
            recipe_id,
            resulting_item: DetailedItemInfo::from_item_id(recipe.item_id),
            progress: rlvl_entry.max_progress * recipe.progress_factor / 100,
            quality: rlvl_entry.max_quality * recipe.quality_factor / 100,
            durability: rlvl_entry.max_durability * recipe.durability_factor / 100,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // False positive (used by expect_test::expect)
struct DetailedMissionInfo {
    job_id: u8,
    recipes: Vec<DetailedRecipeInfo>,
}

impl DetailedMissionInfo {
    pub fn from_mission_id(mission_id: u32) -> Self {
        let mission = STELLAR_MISSIONS.get(mission_id).unwrap();
        Self {
            job_id: mission.job_id,
            recipes: mission
                .recipe_ids
                .iter()
                .copied()
                .map(DetailedRecipeInfo::from_recipe_id)
                .collect(),
        }
    }
}

#[test]
fn gathering_miscellany() {
    let matching_mission_ids = find_stellar_missions("Gathering Miscellany", Locale::EN);
    assert_eq!(matching_mission_ids.len(), 1);

    let mission_id = matching_mission_ids[0];
    let mission_details = DetailedMissionInfo::from_mission_id(mission_id);
    let expected_details = expect![[r#"
        DetailedMissionInfo {
            job_id: 0,
            recipes: [
                DetailedRecipeInfo {
                    recipe_id: 36167,
                    resulting_item: DetailedItemInfo {
                        item_id: 48277,
                        item_name: "Survey Bracelet",
                    },
                    progress: 4422,
                    quality: 7080,
                    durability: 80,
                },
            ],
        }
    "#]];
    expected_details.assert_debug_eq(&mission_details);
}

#[test]
fn meteoric_material_test_processing() {
    let matching_mission_ids =
        find_stellar_missions("Meteoric Material Test Processing", Locale::EN);
    assert_eq!(matching_mission_ids.len(), 3); // BSM, ARM, GSM

    let mission_id = matching_mission_ids[0];
    let mission_details = DetailedMissionInfo::from_mission_id(mission_id);
    let expected_details = expect![[r#"
        DetailedMissionInfo {
            job_id: 1,
            recipes: [
                DetailedRecipeInfo {
                    recipe_id: 36259,
                    resulting_item: DetailedItemInfo {
                        item_id: 48244,
                        item_name: "Cosmotized Lunar Adamantite Ingot \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
                DetailedRecipeInfo {
                    recipe_id: 36260,
                    resulting_item: DetailedItemInfo {
                        item_id: 48245,
                        item_name: "Cosmotized Chondrite Ingot \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
                DetailedRecipeInfo {
                    recipe_id: 36261,
                    resulting_item: DetailedItemInfo {
                        item_id: 48246,
                        item_name: "Cosmotized Ilmenite Plate \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
            ],
        }
    "#]];
    expected_details.assert_debug_eq(&mission_details);

    let mission_id = matching_mission_ids[1];
    let mission_details = DetailedMissionInfo::from_mission_id(mission_id);
    let expected_details = expect![[r#"
        DetailedMissionInfo {
            job_id: 2,
            recipes: [
                DetailedRecipeInfo {
                    recipe_id: 36322,
                    resulting_item: DetailedItemInfo {
                        item_id: 48244,
                        item_name: "Cosmotized Lunar Adamantite Ingot \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
                DetailedRecipeInfo {
                    recipe_id: 36323,
                    resulting_item: DetailedItemInfo {
                        item_id: 48245,
                        item_name: "Cosmotized Chondrite Ingot \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
                DetailedRecipeInfo {
                    recipe_id: 36324,
                    resulting_item: DetailedItemInfo {
                        item_id: 48246,
                        item_name: "Cosmotized Ilmenite Plate \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
            ],
        }
    "#]];
    expected_details.assert_debug_eq(&mission_details);

    let mission_id = matching_mission_ids[2];
    let mission_details = DetailedMissionInfo::from_mission_id(mission_id);
    let expected_details = expect![[r#"
        DetailedMissionInfo {
            job_id: 3,
            recipes: [
                DetailedRecipeInfo {
                    recipe_id: 36385,
                    resulting_item: DetailedItemInfo {
                        item_id: 48244,
                        item_name: "Cosmotized Lunar Adamantite Ingot \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
                DetailedRecipeInfo {
                    recipe_id: 36386,
                    resulting_item: DetailedItemInfo {
                        item_id: 48245,
                        item_name: "Cosmotized Chondrite Ingot \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
                DetailedRecipeInfo {
                    recipe_id: 36387,
                    resulting_item: DetailedItemInfo {
                        item_id: 48246,
                        item_name: "Cosmotized Ilmenite Plate \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15000,
                    durability: 70,
                },
            ],
        }
    "#]];
    expected_details.assert_debug_eq(&mission_details);
}

#[test]
fn ex_natural_remedy_inspection_ii() {
    let matching_mission_ids =
        find_stellar_missions("EX: Natural Remedy Inspection II", Locale::EN);
    assert_eq!(matching_mission_ids.len(), 1);

    let mission_id = matching_mission_ids[0];
    let mission_details = DetailedMissionInfo::from_mission_id(mission_id);
    let expected_details = expect![[r#"
        DetailedMissionInfo {
            job_id: 6,
            recipes: [
                DetailedRecipeInfo {
                    recipe_id: 36588,
                    resulting_item: DetailedItemInfo {
                        item_id: 48574,
                        item_name: "Survey Tincture \u{e03d}",
                    },
                    progress: 7500,
                    quality: 15400,
                    durability: 75,
                },
            ],
        }
    "#]];
    expected_details.assert_debug_eq(&mission_details);
}
