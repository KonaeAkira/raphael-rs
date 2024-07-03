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

    pub fn craftsmanship(&self) -> u32 {
        self.crafter_stats[self.selected_job].craftsmanship
    }

    pub fn craftsmanship_mut(&mut self) -> &mut u32 {
        &mut self.crafter_stats[self.selected_job].craftsmanship
    }

    pub fn control(&self) -> u32 {
        self.crafter_stats[self.selected_job].control
    }

    pub fn control_mut(&mut self) -> &mut u32 {
        &mut self.crafter_stats[self.selected_job].control
    }

    pub fn cp(&self) -> u32 {
        self.crafter_stats[self.selected_job].cp
    }

    pub fn cp_mut(&mut self) -> &mut u32 {
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
