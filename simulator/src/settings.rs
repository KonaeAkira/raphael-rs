#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Settings {
    pub max_cp: i16,
    pub max_durability: i8,
    pub max_progress: u16,
    pub max_quality: u16,
    pub base_progress: u16,
    pub base_quality: u16,
    pub job_level: u8,
    pub allowed_actions: ActionMask,
    pub adversarial: bool,
}

impl Settings {
    pub fn is_action_allowed<ACTION: ActionImpl>(&self) -> bool {
        self.job_level >= ACTION::LEVEL_REQUIREMENT
            && self.allowed_actions.has_mask(ACTION::ACTION_MASK)
    }
}

use crate::{Action, ActionImpl};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActionMask {
    mask: u64,
}

impl ActionMask {
    pub const fn none() -> Self {
        Self { mask: 0 }
    }

    pub const fn all() -> Self {
        Self { mask: u64::MAX }
    }

    pub const fn has(self, action: Action) -> bool {
        (self.mask & (1 << action as u64)) != 0
    }

    pub const fn has_mask(self, other: Self) -> bool {
        (self.mask & other.mask) == other.mask
    }

    pub const fn add(self, action: Action) -> Self {
        let bit = 1 << (action as u64);
        Self {
            mask: self.mask | bit,
        }
    }

    pub const fn remove(self, action: Action) -> Self {
        let bit = 1 << (action as u64);
        Self {
            mask: (self.mask | bit) ^ bit,
        }
    }

    pub const fn union(self, other: Self) -> Self {
        Self {
            mask: self.mask | other.mask,
        }
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self {
            mask: self.mask & other.mask,
        }
    }

    pub const fn minus(self, other: Self) -> Self {
        Self {
            mask: self.mask & (!other.mask),
        }
    }

    pub fn actions(self) -> Box<[Action]> {
        ALL_ACTIONS
            .iter()
            .copied()
            .filter(|action| ((self.mask >> *action as u64) & 1) != 0)
            .collect()
    }

    pub fn actions_iter(self) -> impl Iterator<Item = Action> {
        ALL_ACTIONS
            .iter()
            .copied()
            .filter(move |action| ((self.mask >> *action as u64) & 1) != 0)
    }
}

#[macro_export]
macro_rules! action_mask {
    ( $( $x:expr ),* ) => {
        {
            let mut action_mask = ActionMask::none();
            $(
                action_mask = action_mask.add($x);
            )*
            action_mask
        }
    };
}

const ALL_ACTIONS: &[Action] = &[
    Action::BasicSynthesis,
    Action::BasicTouch,
    Action::MasterMend,
    Action::Observe,
    Action::WasteNot,
    Action::Veneration,
    Action::StandardTouch,
    Action::GreatStrides,
    Action::Innovation,
    Action::WasteNot2,
    Action::ByregotsBlessing,
    Action::PreciseTouch,
    Action::MuscleMemory,
    Action::CarefulSynthesis,
    Action::Manipulation,
    Action::PrudentTouch,
    Action::TrainedEye,
    Action::Reflect,
    Action::PreparatoryTouch,
    Action::Groundwork,
    Action::DelicateSynthesis,
    Action::IntensiveSynthesis,
    Action::AdvancedTouch,
    Action::HeartAndSoul,
    Action::PrudentSynthesis,
    Action::TrainedFinesse,
    Action::RefinedTouch,
    Action::ImmaculateMend,
    Action::TrainedPerfection,
    Action::QuickInnovation,
];
