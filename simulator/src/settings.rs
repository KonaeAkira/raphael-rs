#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

use crate::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

    pub fn from_level(level: u8) -> Self {
        let mut result = Self::none();
        for action in ALL_ACTIONS {
            if action.level_requirement() <= level {
                result = result.add(*action);
            }
        }
        result
    }

    pub const fn has(self, action: Action) -> bool {
        (self.mask & (1 << action as u64)) != 0
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
    Action::ComboStandardTouch,
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
    Action::ComboAdvancedTouch,
    Action::HeartAndSoul,
    Action::PrudentSynthesis,
    Action::TrainedFinesse,
    Action::ComboRefinedTouch,
    Action::ImmaculateMend,
    Action::TrainedPerfection,
    Action::QuickInnovation,
];
