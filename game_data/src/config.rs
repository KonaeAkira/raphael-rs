use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct CrafterStats {
    pub craftsmanship: u16,
    pub control: u16,
    pub cp: u16,
    pub level: u8,
    pub manipulation: bool,
}

impl Default for CrafterStats {
    fn default() -> Self {
        Self {
            craftsmanship: 3858,
            control: 4057,
            cp: 687,
            level: 90,
            manipulation: true,
        }
    }
}
