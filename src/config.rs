use std::num::NonZeroUsize;

use raphael_data::{
    CrafterStats, CustomRecipeOverrides, Ingredient, LEVEL_ADJUST_TABLE, Locale,
    RECIPE_TO_STELLAR_MISSION_LINKS, Recipe, STELLAR_MISSIONS,
};
use raphael_sim::Settings;
use raphael_translations::t;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QualitySource {
    HqMaterialList([u8; 6]),
    Value(u16),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AppConfig {
    pub zoom_percentage: u16,
    pub num_threads: Option<NonZeroUsize>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            zoom_percentage: 100,
            num_threads: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RecipeSource {
    Normal {
        id: u32,
        data: Recipe,
    },
    Custom {
        data: Recipe,
        overrides: CustomRecipeOverrides,
    },
}

impl RecipeSource {
    pub fn into_custom(self, job_level: u8, settings: Settings) -> Self {
        match self {
            Self::Normal { mut data, .. } => {
                data.item_id = 0;

                if data.max_level_scaling != 0 {
                    let job_level = std::cmp::min(data.max_level_scaling, job_level);
                    data.recipe_level = LEVEL_ADJUST_TABLE[job_level as usize];
                }

                data.req_craftsmanship = 0;
                data.req_control = 0;
                data.max_level_scaling = 0;
                data.material_factor = 0;
                data.ingredients = [Ingredient::default(); 6];

                Self::Custom {
                    data,
                    overrides: CustomRecipeOverrides {
                        max_progress_override: settings.max_progress,
                        max_quality_override: settings.max_quality,
                        max_durability_override: settings.max_durability,
                        base_progress_override: None,
                        base_quality_override: None,
                    },
                }
            }
            Self::Custom { .. } => self,
        }
    }

    // TODO: figure out if this heuristic works (seems like it doesn't; see Oizys EX+ sequential missions)
    // or determine how to properly calculate the number of safely usable charges for gold rating
    pub fn default_stellar_steady_hand_charges(&self) -> u8 {
        match self {
            Self::Normal { id, .. } => {
                RECIPE_TO_STELLAR_MISSION_LINKS
                    .get(*id)
                    .map_or(0, |stellar_mission_id| {
                        u8::from(
                            STELLAR_MISSIONS[*stellar_mission_id].stellar_steady_hand_charges > 0,
                        )
                    })
            }
            Self::Custom { .. } => 0,
        }
    }

    pub fn max_stellar_steady_hand_charges(&self) -> u8 {
        match self {
            Self::Normal { id, .. } => {
                RECIPE_TO_STELLAR_MISSION_LINKS
                    .get(*id)
                    .map_or(0, |stellar_mission_id| {
                        STELLAR_MISSIONS[*stellar_mission_id].stellar_steady_hand_charges
                    })
            }
            Self::Custom { .. } => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RecipeConfiguration {
    pub recipe_source: RecipeSource,
    pub quality_source: QualitySource,
}

impl Default for RecipeConfiguration {
    fn default() -> Self {
        Self {
            recipe_source: raphael_data::RECIPES
                .entries()
                .map(|(id, data)| RecipeSource::Normal { id, data: *data })
                .next()
                .unwrap(),
            quality_source: QualitySource::HqMaterialList([0; 6]),
        }
    }
}

impl RecipeConfiguration {
    pub fn recipe(&self) -> &Recipe {
        match &self.recipe_source {
            RecipeSource::Normal { data, .. } | RecipeSource::Custom { data, .. } => data,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct CrafterConfig {
    pub selected_job: u8,
    pub crafter_stats: [CrafterStats; 8],
}

impl CrafterConfig {
    pub fn active_stats(&self) -> &CrafterStats {
        &self.crafter_stats[self.selected_job as usize]
    }

    pub fn active_stats_mut(&mut self) -> &mut CrafterStats {
        &mut self.crafter_stats[self.selected_job as usize]
    }
}

impl Default for CrafterConfig {
    fn default() -> Self {
        Self {
            selected_job: 1,
            crafter_stats: Default::default(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityTarget {
    Zero,
    Half,
    CollectableT1,
    CollectableT2,
    CollectableT3,
    #[default]
    Full,
    Custom(u16),
}

impl QualityTarget {
    pub fn get_target(self, max_quality: u16) -> u16 {
        match self {
            Self::Zero => 0,
            Self::Half => max_quality / 2,
            Self::CollectableT1 => (max_quality as u32 * 55 / 100) as u16,
            Self::CollectableT2 => (max_quality as u32 * 75 / 100) as u16,
            Self::CollectableT3 => (max_quality as u32 * 95 / 100) as u16,
            Self::Full => max_quality,
            Self::Custom(quality) => quality,
        }
    }

    pub fn display(self, locale: Locale) -> &'static str {
        match self {
            Self::Zero => t!(locale, "0% quality"),
            Self::Half => t!(locale, "50% quality"),
            Self::CollectableT1 => t!(locale, "55% quality"),
            Self::CollectableT2 => t!(locale, "75% quality"),
            Self::CollectableT3 => t!(locale, "95% quality"),
            Self::Full => t!(locale, "100% quality"),
            Self::Custom(_) => t!(locale, "Custom"),
        }
    }
}
