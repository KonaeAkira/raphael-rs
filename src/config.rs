use game_data::CrafterStats;
use serde::{Deserialize, Serialize};

pub const JOB_NAMES: [&'static str; 8] = ["CRP", "BSM", "ARM", "GSM", "LTW", "WVR", "ALC", "CUL"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub struct CrafterConfig {
    pub selected_job: usize,
    pub crafter_stats: [CrafterStats; 8],
}

impl CrafterConfig {
    pub fn stats(&self) -> CrafterStats {
        self.crafter_stats[self.selected_job]
    }

    pub fn craftsmanship(&self) -> u16 {
        self.crafter_stats[self.selected_job].craftsmanship
    }

    pub fn craftsmanship_mut(&mut self) -> &mut u16 {
        &mut self.crafter_stats[self.selected_job].craftsmanship
    }

    pub fn control(&self) -> u16 {
        self.crafter_stats[self.selected_job].control
    }

    pub fn control_mut(&mut self) -> &mut u16 {
        &mut self.crafter_stats[self.selected_job].control
    }

    pub fn cp(&self) -> u16 {
        self.crafter_stats[self.selected_job].cp
    }

    pub fn cp_mut(&mut self) -> &mut u16 {
        &mut self.crafter_stats[self.selected_job].cp
    }

    pub fn level(&self) -> u8 {
        self.crafter_stats[self.selected_job].level
    }

    pub fn level_mut(&mut self) -> &mut u8 {
        &mut self.crafter_stats[self.selected_job].level
    }

    pub fn manipulation(&self) -> bool {
        self.crafter_stats[self.selected_job].manipulation
    }

    pub fn manipulation_mut(&mut self) -> &mut bool {
        &mut self.crafter_stats[self.selected_job].manipulation
    }
}

impl Default for CrafterConfig {
    fn default() -> Self {
        Self {
            selected_job: 1,
            crafter_stats: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityTarget {
    Zero,
    CollectableT1,
    CollectableT2,
    CollectableT3,
    Full,
    Custom(u16),
}

impl QualityTarget {
    pub fn get_target(self, max_quality: u16) -> u16 {
        match self {
            Self::Zero => 0,
            Self::CollectableT1 => (max_quality as f64 * 0.55).ceil() as u16,
            Self::CollectableT2 => (max_quality as f64 * 0.75).ceil() as u16,
            Self::CollectableT3 => (max_quality as f64 * 0.95).ceil() as u16,
            Self::Full => max_quality,
            Self::Custom(quality) => quality,
        }
    }
}

impl std::fmt::Display for QualityTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Zero => "0% quality",
                Self::CollectableT1 => "55% quality",
                Self::CollectableT2 => "75% quality",
                Self::CollectableT3 => "95% quality",
                Self::Full => "100% quality",
                Self::Custom(_) => "Custom",
            }
        )
    }
}
