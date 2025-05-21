use raphael_data::Consumable;

use crate::{
    app::SolverConfig,
    config::{
        CrafterConfig, CustomRecipeOverridesConfiguration, QualitySource, RecipeConfiguration,
    },
};

pub fn get_initial_quality(
    recipe_config: &RecipeConfiguration,
    crafter_config: &CrafterConfig,
) -> u16 {
    match recipe_config.quality_source {
        QualitySource::HqMaterialList(hq_materials) => raphael_data::get_initial_quality(
            *crafter_config.active_stats(),
            recipe_config.recipe,
            hq_materials,
        ),
        QualitySource::Value(quality) => quality,
    }
}

pub fn get_game_settings(
    recipe_config: &RecipeConfiguration,
    custom_recipe_overrides_config: &CustomRecipeOverridesConfiguration,
    solver_config: &SolverConfig,
    crafter_config: &CrafterConfig,
    selected_food: Option<Consumable>,
    selected_potion: Option<Consumable>,
) -> raphael_sim::Settings {
    let mut game_settings = raphael_data::get_game_settings(
        recipe_config.recipe,
        match custom_recipe_overrides_config.use_custom_recipe {
            true => Some(custom_recipe_overrides_config.custom_recipe_overrides),
            false => None,
        },
        *crafter_config.active_stats(),
        selected_food,
        selected_potion,
    );

    game_settings.adversarial = solver_config.adversarial;
    game_settings.backload_progress = solver_config.backload_progress;
    game_settings
}
