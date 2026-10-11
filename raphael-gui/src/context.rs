use raphael_data::{Consumable, CosmicExplorationZone, CrafterStats, Locale};
use raphael_translations::t;
use serde::{Deserialize, Serialize};

use crate::{
    config::{AppConfig, CrafterConfig, QualitySource, QualityTarget, RecipeConfiguration},
    macros::MacroViewConfig,
    saved_rotations::{SavedRotationsConfig, SavedRotationsData},
    storage,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecipeSearchDomain {
    #[default]
    Recipes,
    StellarMissions,
}

impl RecipeSearchDomain {
    pub fn display(self, locale: Locale) -> &'static str {
        match self {
            Self::Recipes => t!(locale, "Recipes"),
            Self::StellarMissions => t!(locale, "Missions"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchState {
    pub recipe: RecipeSearchState,
    pub food: GenericSearchState,
    pub potion: GenericSearchState,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecipeSearchFilters {
    pub active_job_only: bool,
    pub cosmic_exploration_zone: Option<CosmicExplorationZone>,
}

impl RecipeSearchFilters {
    pub fn construct_recipe_filter(
        &self,
        crafter_config: &CrafterConfig,
    ) -> raphael_data::RecipeFilters {
        raphael_data::RecipeFilters {
            job_id: self.active_job_only.then_some(crafter_config.selected_job),
        }
    }

    pub fn construct_stellar_mission_filter(
        &self,
        crafter_config: &CrafterConfig,
    ) -> raphael_data::StellarMissionFilters {
        raphael_data::StellarMissionFilters {
            job_id: self.active_job_only.then_some(crafter_config.selected_job),
            zone: self.cosmic_exploration_zone,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecipeSearchState {
    pub show_custom_recipe_select: bool,

    pub search_domain: RecipeSearchDomain,
    pub search_text: String,

    #[serde(default)]
    pub filters: RecipeSearchFilters,
}

impl RecipeSearchState {
    pub fn filters_active(&self) -> bool {
        match self.search_domain {
            RecipeSearchDomain::Recipes => self.filters.active_job_only,
            RecipeSearchDomain::StellarMissions => {
                self.filters.active_job_only || self.filters.cosmic_exploration_zone.is_some()
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct GenericSearchState {
    pub search_text: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolverConfig {
    pub quality_target: QualityTarget,
    pub backload_progress: bool,
    pub adversarial: bool,
    #[serde(default)]
    pub must_reach_target_quality: bool,
    #[serde(default)]
    pub stellar_steady_hand_charges: u8,
}

pub struct AppContext {
    pub locale: Locale,
    pub app_config: AppConfig,
    pub search_state: SearchState,
    pub recipe_config: RecipeConfiguration,
    pub selected_food: Option<Consumable>,
    pub selected_potion: Option<Consumable>,
    pub crafter_config: CrafterConfig,
    pub solver_config: SolverConfig,
    pub macro_view_config: MacroViewConfig,
    pub saved_rotations_config: SavedRotationsConfig,
    pub saved_rotations_data: SavedRotationsData,
}

/// Keys under which each part of the [`AppContext`] is persisted.
/// These match the keys used by the previous (egui-based) versions of the app.
pub mod keys {
    pub const LOCALE: &str = "LOCALE";
    pub const APP_CONFIG: &str = "APP_CONFIG";
    pub const SEARCH_STATE: &str = "SEARCH_STATE";
    pub const RECIPE_CONFIG: &str = "RECIPE_CONFIG";
    pub const SELECTED_FOOD: &str = "SELECTED_FOOD";
    pub const SELECTED_POTION: &str = "SELECTED_POTION";
    pub const CRAFTER_CONFIG: &str = "CRAFTER_CONFIG";
    pub const SOLVER_CONFIG: &str = "SOLVER_CONFIG";
    pub const MACRO_VIEW_CONFIG: &str = "MACRO_VIEW_CONFIG";
    pub const SAVED_ROTATIONS_CONFIG: &str = "SAVED_ROTATIONS_CONFIG";
    pub const SAVED_ROTATIONS: &str = "SAVED_ROTATIONS";
}

impl AppContext {
    pub fn load() -> Self {
        Self {
            locale: storage::load(keys::LOCALE).unwrap_or(Locale::EN),
            app_config: storage::load_or_default(keys::APP_CONFIG),
            search_state: storage::load_or_default(keys::SEARCH_STATE),
            recipe_config: storage::load_or_default(keys::RECIPE_CONFIG),
            selected_food: storage::load_or_default(keys::SELECTED_FOOD),
            selected_potion: storage::load_or_default(keys::SELECTED_POTION),
            crafter_config: storage::load_or_default(keys::CRAFTER_CONFIG),
            solver_config: storage::load_or_default(keys::SOLVER_CONFIG),
            macro_view_config: storage::load_or_default(keys::MACRO_VIEW_CONFIG),
            saved_rotations_config: storage::load_or_default(keys::SAVED_ROTATIONS_CONFIG),
            saved_rotations_data: storage::load_or_default(keys::SAVED_ROTATIONS),
        }
    }

    pub fn save(&self) {
        storage::store(keys::LOCALE, &self.locale);
        storage::store(keys::APP_CONFIG, &self.app_config);
        storage::store(keys::SEARCH_STATE, &self.search_state);
        storage::store(keys::RECIPE_CONFIG, &self.recipe_config);
        storage::store(keys::SELECTED_FOOD, &self.selected_food);
        storage::store(keys::SELECTED_POTION, &self.selected_potion);
        storage::store(keys::CRAFTER_CONFIG, &self.crafter_config);
        storage::store(keys::SOLVER_CONFIG, &self.solver_config);
        storage::store(keys::MACRO_VIEW_CONFIG, &self.macro_view_config);
        storage::store(keys::SAVED_ROTATIONS_CONFIG, &self.saved_rotations_config);
        storage::store(keys::SAVED_ROTATIONS, &self.saved_rotations_data);
    }

    /// Solver config with settings that are unavailable for the current recipe turned off.
    /// Returns `None` if the current solver config is already valid.
    pub fn sanitized_solver_config(&self) -> Option<SolverConfig> {
        let mut solver_config = self.solver_config;
        if self.recipe_config.recipe().is_expert {
            solver_config.adversarial = false;
        }
        let max_charges = self
            .recipe_config
            .recipe_source
            .max_stellar_steady_hand_charges();
        solver_config.stellar_steady_hand_charges =
            solver_config.stellar_steady_hand_charges.min(max_charges);
        (solver_config != self.solver_config).then_some(solver_config)
    }

    pub fn initial_quality(&self) -> u16 {
        let Self {
            recipe_config,
            crafter_config,
            ..
        } = self;
        match recipe_config.quality_source {
            QualitySource::HqMaterialList(hq_materials) => raphael_data::get_initial_quality(
                *crafter_config.active_stats(),
                *recipe_config.recipe(),
                hq_materials,
            ),
            QualitySource::Value(quality) => quality,
        }
    }

    pub fn game_settings(&self) -> raphael_sim::Settings {
        let Self {
            recipe_config,
            selected_food: food,
            selected_potion: potion,
            crafter_config,
            solver_config,
            ..
        } = self;
        let (recipe, custom_recipe_overrides) = match recipe_config.recipe_source {
            crate::config::RecipeSource::Normal { data, .. } => (data, None),
            crate::config::RecipeSource::Custom { data, overrides } => (data, Some(overrides)),
        };
        let mut game_settings = raphael_data::get_game_settings(
            recipe,
            custom_recipe_overrides,
            *crafter_config.active_stats(),
            *food,
            *potion,
        );
        game_settings.adversarial = solver_config.adversarial;
        game_settings.backload_progress = solver_config.backload_progress;
        game_settings.stellar_steady_hand_charges = solver_config.stellar_steady_hand_charges;
        game_settings
    }

    pub fn active_stats(&self) -> &CrafterStats {
        self.crafter_config.active_stats()
    }

    pub fn active_stats_mut(&mut self) -> &mut CrafterStats {
        self.crafter_config.active_stats_mut()
    }
}
