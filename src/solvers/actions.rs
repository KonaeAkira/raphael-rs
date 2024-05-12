use crate::game::Action;

#[derive(Debug, Clone, Copy)]
pub struct ActionMask {
    mask: u64,
}

impl ActionMask {
    pub const fn new() -> Self {
        Self { mask: 0 }
    }

    pub const fn add(self, action: Action) -> Self {
        Self {
            mask: self.mask | (1 << (action as u64)),
        }
    }

    pub const fn union(self, other: Self) -> Self {
        Self {
            mask: self.mask | other.mask,
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
            let mut action_mask = ActionMask::new();
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
    Action::TricksOfTheTrade,
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
    Action::FocusedSynthesis,
    Action::FocusedTouch,
    Action::Reflect,
    Action::PreparatoryTouch,
    Action::Groundwork,
    Action::DelicateSynthesis,
    Action::IntensiveSynthesis,
    Action::AdvancedTouch,
    Action::ComboAdvancedTouch,
    Action::PrudentSynthesis,
    Action::TrainedFinesse,
];

pub const PROGRESS_ACTIONS: ActionMask = action_mask!(
    Action::BasicSynthesis,
    Action::Observe,
    Action::Veneration,
    Action::MuscleMemory,
    Action::CarefulSynthesis,
    Action::FocusedSynthesis,
    Action::Groundwork,
    // Action::IntensiveSynthesis,
    Action::PrudentSynthesis
);

pub const QUALITY_ACTIONS: ActionMask = action_mask!(
    Action::BasicTouch,
    Action::Observe,
    Action::StandardTouch,
    Action::ComboStandardTouch,
    Action::GreatStrides,
    Action::Innovation,
    Action::ByregotsBlessing,
    // Action::PreciseTouch,
    Action::PrudentTouch,
    Action::FocusedTouch,
    Action::Reflect,
    Action::PreparatoryTouch,
    Action::AdvancedTouch,
    Action::ComboAdvancedTouch,
    Action::TrainedFinesse
);

pub const MIXED_ACTIONS: ActionMask = action_mask!(Action::DelicateSynthesis);

pub const DURABILITY_ACTIONS: ActionMask = action_mask!(
    Action::MasterMend,
    Action::WasteNot,
    Action::WasteNot2,
    Action::Manipulation
);
