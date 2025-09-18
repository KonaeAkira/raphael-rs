use raphael_data::{Consumable, CrafterStats, Locale};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    config::{
        AppConfig, CrafterConfig, CustomRecipeOverridesConfiguration, QualitySource, QualityTarget,
        RecipeConfiguration,
    },
    widgets::{MacroViewConfig, SavedRotationsConfig, SavedRotationsData},
};

fn load<T: DeserializeOwned>(cc: &eframe::CreationContext<'_>, key: &'static str, default: T) -> T {
    match cc.storage {
        Some(storage) => eframe::get_value(storage, key).unwrap_or(default),
        None => default,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SolverConfig {
    pub quality_target: QualityTarget,
    pub backload_progress: bool,
    pub adversarial: bool,
}

pub struct AppContext {
    pub locale: Locale,
    pub app_config: AppConfig,
    pub recipe_config: RecipeConfiguration,
    pub custom_recipe_overrides_config: CustomRecipeOverridesConfiguration,
    pub selected_food: Option<Consumable>,
    pub selected_potion: Option<Consumable>,
    pub crafter_config: CrafterConfig,
    pub solver_config: SolverConfig,
    pub macro_view_config: MacroViewConfig,
    pub saved_rotations_config: SavedRotationsConfig,
    pub saved_rotations_data: SavedRotationsData,
}

impl AppContext {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            locale: load(cc, "LOCALE", Locale::EN),
            app_config: load(cc, "APP_CONFIG", AppConfig::default()),
            recipe_config: load(cc, "RECIPE_CONFIG", RecipeConfiguration::default()),
            custom_recipe_overrides_config: load(
                cc,
                "CUSTOM_RECIPE_OVERRIDES_CONFIG",
                CustomRecipeOverridesConfiguration::default(),
            ),
            selected_food: load(cc, "SELECTED_FOOD", None),
            selected_potion: load(cc, "SELECTED_POTION", None),
            crafter_config: load(cc, "CRAFTER_CONFIG", CrafterConfig::default()),
            solver_config: load(cc, "SOLVER_CONFIG", SolverConfig::default()),
            macro_view_config: load(cc, "MACRO_VIEW_CONFIG", MacroViewConfig::default()),
            saved_rotations_config: load(
                cc,
                "SAVED_ROTATIONS_CONFIG",
                SavedRotationsConfig::default(),
            ),
            saved_rotations_data: load(cc, "SAVED_ROTATIONS", SavedRotationsData::default()),
        }
    }

    pub fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "LOCALE", &self.locale);
        eframe::set_value(storage, "APP_CONFIG", &self.app_config);
        eframe::set_value(storage, "RECIPE_CONFIG", &self.recipe_config);
        eframe::set_value(
            storage,
            "CUSTOM_RECIPE_OVERRIDES_CONFIG",
            &self.custom_recipe_overrides_config,
        );
        eframe::set_value(storage, "SELECTED_FOOD", &self.selected_food);
        eframe::set_value(storage, "SELECTED_POTION", &self.selected_potion);
        eframe::set_value(storage, "CRAFTER_CONFIG", &self.crafter_config);
        eframe::set_value(storage, "SOLVER_CONFIG", &self.solver_config);
        eframe::set_value(storage, "MACRO_VIEW_CONFIG", &self.macro_view_config);
        eframe::set_value(
            storage,
            "SAVED_ROTATIONS_CONFIG",
            &self.saved_rotations_config,
        );
        eframe::set_value(storage, "SAVED_ROTATIONS", &self.saved_rotations_data);
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
                recipe_config.recipe,
                hq_materials,
            ),
            QualitySource::Value(quality) => quality,
        }
    }

    pub fn game_settings(&self) -> raphael_sim::Settings {
        let Self {
            recipe_config,
            custom_recipe_overrides_config,
            selected_food: food,
            selected_potion: potion,
            crafter_config,
            solver_config,
            ..
        } = self;
        let custom_recipe_overrides = match custom_recipe_overrides_config.use_custom_recipe {
            true => Some(custom_recipe_overrides_config.custom_recipe_overrides),
            false => None,
        };
        let mut game_settings = raphael_data::get_game_settings(
            recipe_config.recipe,
            custom_recipe_overrides,
            *crafter_config.active_stats(),
            *food,
            *potion,
        );
        game_settings.adversarial = solver_config.adversarial;
        game_settings.backload_progress = solver_config.backload_progress;
        game_settings
    }

    pub fn selected_job(&self) -> u8 {
        self.crafter_config.selected_job
    }

    pub fn active_stats(&self) -> &CrafterStats {
        self.crafter_config.active_stats()
    }

    pub fn active_stats_mut(&mut self) -> &mut CrafterStats {
        self.crafter_config.active_stats_mut()
    }
}
