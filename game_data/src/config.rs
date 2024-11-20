#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CrafterStats {
    #[cfg_attr(feature = "serde", serde(default))]
    pub craftsmanship: u16,
    #[cfg_attr(feature = "serde", serde(default))]
    pub control: u16,
    #[cfg_attr(feature = "serde", serde(default))]
    pub cp: u16,
    #[cfg_attr(feature = "serde", serde(default))]
    pub level: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub manipulation: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub heart_and_soul: bool,
    #[cfg_attr(feature = "serde", serde(default))]
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
            heart_and_soul: false,
            quick_innovation: false,
        }
    }
}
