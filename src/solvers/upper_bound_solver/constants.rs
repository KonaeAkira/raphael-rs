use crate::game::{units::CP, Action};

// cost per effect stack
pub const WASTE_NOT_COST: CP = Action::WasteNot2.base_cp_cost() / 8;
pub const MANIPULATION_COST: CP = Action::Manipulation.base_cp_cost() / 8;

// cost for 5 duration
pub const DURABILITY_COST: CP = Action::Manipulation.base_cp_cost() / 8;
