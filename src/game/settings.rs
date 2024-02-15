use crate::game::units::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Settings {
    pub max_cp: CP,
    pub max_durability: Durability,
    pub max_progress: Progress,
    pub max_quality: Quality,
}
