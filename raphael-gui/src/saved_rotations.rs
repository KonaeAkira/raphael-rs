use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

use raphael_data::{CrafterStats, CustomRecipeOverrides, Locale, Recipe};
use raphael_sim::*;
use raphael_translations::t;
use serde::{Deserialize, Serialize};

use crate::{
    config::{QualitySource, RecipeConfiguration, RecipeSource},
    context::{AppContext, SolverConfig},
    solve::{RunningSolveInfo, SolveParameters},
};

fn generate_unique_rotation_id() -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    web_time::Instant::now().hash(&mut hasher);
    hasher.finish()
}

// Kept for compatibility with old data using the derived deserialization
#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CustomRecipeOverridesConfiguration {
    pub custom_recipe_overrides: CustomRecipeOverrides,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RecipeInfo {
    NormalRecipe(u32),
    CustomRecipe(Recipe, CustomRecipeOverridesConfiguration),
}

impl From<&RecipeSource> for RecipeInfo {
    fn from(recipe_source: &RecipeSource) -> Self {
        match recipe_source {
            RecipeSource::Normal { id, .. } => Self::NormalRecipe(*id),
            RecipeSource::Custom { data, overrides } => Self::CustomRecipe(
                *data,
                CustomRecipeOverridesConfiguration {
                    custom_recipe_overrides: *overrides,
                },
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolveInfo {
    pub game_settings: Settings,
    pub initial_quality: u16,
    pub solver_config: SolverConfig,
}

impl SolveInfo {
    pub fn new(
        game_settings: &Settings,
        initial_quality: u16,
        solver_config: &SolverConfig,
    ) -> Self {
        Self {
            game_settings: *game_settings,
            initial_quality,
            solver_config: *solver_config,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rotation {
    pub unique_id: u64,
    pub name: String,
    pub solver: String,
    pub actions: Vec<Action>,
    #[serde(default)]
    pub recipe_info: Option<RecipeInfo>,
    #[serde(default)]
    pub solve_info: Option<SolveInfo>,
    pub food: Option<(u32, bool)>,
    pub potion: Option<(u32, bool)>,
    pub crafter_stats: CrafterStats,
}

impl Rotation {
    pub fn new(info: &RunningSolveInfo, actions: Vec<Action>, locale: Locale) -> Self {
        let RunningSolveInfo {
            solve_params:
                SolveParameters {
                    settings: game_settings,
                    initial_quality,
                    solver_config,
                },
            recipe_config,
            food,
            potion,
            stats,
            ..
        } = info;
        let name = raphael_data::get_item_name(recipe_config.recipe().item_id, false, locale)
            .unwrap_or(t!(locale, "Unknown item").to_owned());
        let solver_params = format!(
            "Raphael v{}{}{}",
            env!("CARGO_PKG_VERSION"),
            match solver_config.backload_progress {
                true => " +backload",
                false => "",
            },
            match solver_config.adversarial {
                true => " +adversarial",
                false => "",
            },
        );
        Self {
            unique_id: generate_unique_rotation_id(),
            name,
            solver: solver_params,
            actions,
            recipe_info: Some(RecipeInfo::from(&recipe_config.recipe_source)),
            solve_info: Some(SolveInfo::new(
                game_settings,
                *initial_quality,
                solver_config,
            )),
            food: food.map(|consumable| (consumable.item_id, consumable.hq)),
            potion: potion.map(|consumable| (consumable.item_id, consumable.hq)),
            crafter_stats: *stats,
        }
    }
}

impl Clone for Rotation {
    fn clone(&self) -> Self {
        Self {
            unique_id: generate_unique_rotation_id(),
            name: self.name.clone(),
            solver: self.solver.clone(),
            actions: self.actions.clone(),
            recipe_info: self.recipe_info.clone(),
            solve_info: self.solve_info.clone(),
            food: self.food,
            potion: self.potion,
            crafter_stats: self.crafter_stats,
        }
    }
}

impl PartialEq for Rotation {
    fn eq(&self, other: &Self) -> bool {
        // unique_id & name are skipped
        self.solver == other.solver
            && self.actions == other.actions
            && self.recipe_info == other.recipe_info
            && self.solve_info == other.solve_info
            && self.food == other.food
            && self.potion == other.potion
            && self.crafter_stats == other.crafter_stats
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LoadOperation {
    #[default]
    Rotation,
    RotationRecipe,
    RotationRecipeConsumables,
}

struct LoadOperationDisplay {
    load_operation: LoadOperation,
    locale: Locale,
}

impl LoadOperation {
    pub fn display(self, locale: Locale) -> impl std::fmt::Display {
        LoadOperationDisplay {
            load_operation: self,
            locale,
        }
    }
}

impl std::fmt::Display for LoadOperationDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let locale = self.locale;
        let output_str = match self.load_operation {
            LoadOperation::Rotation => t!(locale, "Load rotation"),
            LoadOperation::RotationRecipe => t!(locale, "Load rotation & recipe"),
            LoadOperation::RotationRecipeConsumables => {
                t!(locale, "Load rotation, recipe & consumables")
            }
        };
        write!(f, "{}", output_str)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedRotationsConfig {
    pub load_from_saved_rotations: bool,
    pub default_load_operation: LoadOperation,
    pub max_history_size: usize,
}

impl Default for SavedRotationsConfig {
    fn default() -> Self {
        Self {
            load_from_saved_rotations: false,
            default_load_operation: LoadOperation::Rotation,
            max_history_size: 50,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SavedRotationsData {
    pub pinned: Vec<Rotation>,
    pub solve_history: VecDeque<Rotation>,
}

impl SavedRotationsData {
    pub fn add_solved_rotation(&mut self, rotation: Rotation, config: &SavedRotationsConfig) {
        if let Some(index) = self
            .solve_history
            .iter()
            .position(|saved_rotation| *saved_rotation == rotation)
        {
            self.solve_history.remove(index);
        }

        self.solve_history.push_front(rotation);
        while self.solve_history.len() > config.max_history_size {
            self.solve_history.pop_back();
        }
    }

    pub fn find_solved_rotation(
        &self,
        game_settings: &Settings,
        initial_quality: u16,
        solver_config: &SolverConfig,
    ) -> Option<Vec<Action>> {
        let solve_info = SolveInfo::new(game_settings, initial_quality, solver_config);
        let find_and_map_rotation = |rotation: &Rotation| {
            if let (Some(saved_solver_version), Some(saved_solve_info)) =
                (rotation.solver.split(' ').nth(1), &rotation.solve_info)
                && saved_solver_version == format!("v{}", env!("CARGO_PKG_VERSION"))
                && *saved_solve_info == solve_info
            {
                Some(rotation.actions.clone())
            } else {
                None
            }
        };
        let history_search_result = self.solve_history.iter().find_map(find_and_map_rotation);
        if history_search_result.is_some() {
            history_search_result
        } else {
            self.pinned.iter().find_map(find_and_map_rotation)
        }
    }
}

impl Rotation {
    pub fn duration(&self) -> u8 {
        self.actions.iter().map(|action| action.time_cost()).sum()
    }

    pub fn recipe(&self) -> Option<&Recipe> {
        self.recipe_info
            .as_ref()
            .and_then(|recipe_info| match recipe_info {
                RecipeInfo::NormalRecipe(recipe_id) => raphael_data::RECIPES.get(*recipe_id),
                RecipeInfo::CustomRecipe(recipe, _) => Some(recipe),
            })
    }
}

impl AppContext {
    /// Loads the recipe that was used to generate the rotation.
    pub fn load_rotation_recipe(&mut self, rotation: &Rotation) {
        let Some(recipe_info) = &rotation.recipe_info else {
            return;
        };
        match recipe_info {
            RecipeInfo::NormalRecipe(recipe_id) => {
                if let Some(recipe) = raphael_data::RECIPES.get(*recipe_id) {
                    self.recipe_config = RecipeConfiguration {
                        recipe_source: RecipeSource::Normal {
                            id: *recipe_id,
                            data: *recipe,
                        },
                        quality_source: QualitySource::HqMaterialList([0; 6]),
                    };
                    self.crafter_config.selected_job = recipe.job_id;
                    self.solver_config.stellar_steady_hand_charges = 0;
                    self.search_state.recipe.show_custom_recipe_select = false;
                } else {
                    log::debug!("Unable to find recipe with recipe_id={:?}", recipe_id);
                }
            }
            RecipeInfo::CustomRecipe(recipe, custom_recipe_overrides_config) => {
                self.recipe_config = RecipeConfiguration {
                    recipe_source: RecipeSource::Custom {
                        data: *recipe,
                        overrides: custom_recipe_overrides_config.custom_recipe_overrides,
                    },
                    quality_source: QualitySource::Value(0),
                };
                self.crafter_config.selected_job = recipe.job_id;
                self.solver_config.stellar_steady_hand_charges = 0;
                self.search_state.recipe.show_custom_recipe_select = true;
            }
        }
    }

    pub fn load_rotation_consumables(&mut self, rotation: &Rotation) {
        self.selected_food = rotation.food.and_then(|(item_id, hq)| {
            raphael_data::MEALS
                .iter()
                .find(|food| food.item_id == item_id && food.hq == hq)
                .copied()
        });
        self.selected_potion = rotation.potion.and_then(|(item_id, hq)| {
            raphael_data::POTIONS
                .iter()
                .find(|potion| potion.item_id == item_id && potion.hq == hq)
                .copied()
        });
    }
}
