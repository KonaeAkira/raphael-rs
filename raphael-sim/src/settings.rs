use crate::{Action, ActionImpl};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Settings {
    pub max_cp: u16,
    pub max_durability: u16,
    pub max_progress: u16,
    pub max_quality: u16,
    pub base_progress: u16,
    pub base_quality: u16,
    pub job_level: u8,
    pub allowed_actions: ActionMask,
    pub adversarial: bool,
    /// If `backload_progress` is set, after using any action that increases Progress, the simulator will forbid the use of actions that directly increase Quality.
    pub backload_progress: bool,
}

impl Settings {
    pub fn is_action_allowed<ACTION: ActionImpl>(&self) -> bool {
        self.job_level >= ACTION::LEVEL_REQUIREMENT
            && self.allowed_actions.has_mask(ACTION::ACTION_MASK)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionMask {
    mask: u64,
}

impl ActionMask {
    pub const fn from_bits(mask: u64) -> Self {
        Self { mask }
    }

    pub const fn to_bits(self) -> u64 {
        self.mask
    }

    pub const fn none() -> Self {
        Self { mask: 0 }
    }

    pub const fn all() -> Self {
        Self { mask: u64::MAX }
    }

    /// Excludes TrainedEye and specialist actions
    pub const fn regular() -> Self {
        Self::all()
            .remove(Action::TrainedEye)
            .remove(Action::HeartAndSoul)
            .remove(Action::QuickInnovation)
    }

    pub const fn has(self, action: Action) -> bool {
        (self.mask & (1 << action as u64)) != 0
    }

    pub const fn has_mask(self, other: Self) -> bool {
        (self.mask & other.mask) == other.mask
    }

    #[must_use]
    pub const fn add(self, action: Action) -> Self {
        let bit = 1 << (action as u64);
        Self {
            mask: self.mask | bit,
        }
    }

    #[must_use]
    pub const fn remove(self, action: Action) -> Self {
        let bit = 1 << (action as u64);
        Self {
            mask: (self.mask | bit) ^ bit,
        }
    }

    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self {
            mask: self.mask | other.mask,
        }
    }

    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        Self {
            mask: self.mask & other.mask,
        }
    }

    #[must_use]
    pub const fn minus(self, other: Self) -> Self {
        Self {
            mask: self.mask & (!other.mask),
        }
    }
}
