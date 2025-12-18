use std::num::NonZeroUsize;

use raphael_data::{CrafterStats, CustomRecipeOverrides, Locale, Recipe};
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CustomRecipeOverridesConfiguration {
    pub use_custom_recipe: bool,
    pub custom_recipe_overrides: CustomRecipeOverrides,
    pub use_base_increase_overrides: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RecipeConfiguration {
    pub recipe: Recipe,
    pub quality_source: QualitySource,
}

impl Default for RecipeConfiguration {
    fn default() -> Self {
        Self {
            recipe: *raphael_data::RECIPES.values().next().unwrap(),
            quality_source: QualitySource::HqMaterialList([0; 6]),
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
