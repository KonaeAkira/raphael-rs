use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct CrafterStats {
    #[serde(default)]
    pub craftsmanship: u16,
    #[serde(default)]
    pub control: u16,
    #[serde(default)]
    pub cp: u16,
    #[serde(default)]
    pub level: u8,
    #[serde(default)]
    pub manipulation: bool,
    #[serde(default)]
    pub quick_innovation: bool,
}

impl Default for CrafterStats {
    fn default() -> Self {
        Self {
            craftsmanship: 4900,
            control: 4800,
            cp: 620,
            level: 100,
            manipulation: true,
            quick_innovation: false,
        }
    }
}
