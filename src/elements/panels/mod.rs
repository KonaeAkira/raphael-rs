/// Item spacing applied inside each left-column select panel (recipe, food, potion). Kept
/// in one place so the panels' layout and the table-height math can't drift apart.
pub(crate) const PANEL_ITEM_SPACING: egui::Vec2 = egui::Vec2::new(8.0, 3.0);

mod macro_view;
pub use macro_view::{MacroView, MacroViewConfig};

mod simulator;
pub use simulator::Simulator;

mod recipe_select;
pub use recipe_select::{
    RecipeSelect, SearchDomain as RecipeSearchDomain, recipe_table_min_height,
};

mod consumable_select;
pub use consumable_select::{FoodSelect, PotionSelect, consumable_table_min_height};

mod stats_edit;
pub use stats_edit::StatsEdit;

mod saved_rotations;
pub use saved_rotations::{
    Rotation, SavedRotationsConfig, SavedRotationsData, SavedRotationsWidget,
};

#[cfg(any(debug_assertions, feature = "dev-panel"))]
mod render_info;
#[cfg(any(debug_assertions, feature = "dev-panel"))]
pub use render_info::RenderInfo;
